import { useCallback, useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  captureStatus,
  commitTraceCheckpoint,
  exportPrReasoning,
  fetchBlocks,
  fetchWorkspaceHygiene,
  listWorkspaces,
  normalizeCaptureCandidates,
  startCapture,
  stopCapture,
  updateWorkspace,
  type CaptureCandidate,
} from "../api";
import BlockPanel from "../components/BlockPanel";
import CapturePickerDialog from "../components/CapturePickerDialog";
import CheckpointDialog, {
  type CheckpointFormValues,
} from "../components/CheckpointDialog";
import HygienePanel from "../components/HygienePanel";
import PromptDialog from "../components/PromptDialog";
import { useToast } from "../components/Toast";
import {
  beliefStateLabel,
  hygieneCategoryLabel,
  hypothesisFieldLabel,
  placeholdersForTemplate,
  systemTagLabel,
  type BeliefState,
  type BlockEntry,
  type BlockSystemTag,
  type Workspace,
  type WorkspaceHygieneReport,
} from "../types";

const BELIEF_COLORS: Record<BeliefState, string> = {
  open: "bg-zinc-800 text-zinc-300 border-zinc-700",
  leaning_true: "bg-violet-900/60 text-violet-200 border-violet-800",
  leaning_false: "bg-orange-900/50 text-orange-200 border-orange-800",
  confirmed: "bg-emerald-900/60 text-emerald-200 border-emerald-800",
  rejected: "bg-red-900/50 text-red-300 border-red-800",
};

function blockPreview(block: BlockEntry): string {
  if (block.title.trim()) {
    return block.title;
  }
  return (
    block.hypothesis?.text ??
    block.action?.text ??
    block.evidence?.text ??
    block.conclusion?.text ??
    "(empty block)"
  );
}

/** Detect MCP / external edits when panel is open. */
function blockRevision(block: BlockEntry): string {
  return JSON.stringify({
    updated_at: block.updated_at,
    title: block.title,
    hypothesis: block.hypothesis?.text ?? "",
    action: block.action?.text ?? "",
    evidence: block.evidence?.text ?? "",
    evidence_source: block.evidence?.source ?? "",
    conclusion: block.conclusion?.text ?? "",
    belief_state: block.belief_state,
    system_tag: block.system_tag,
    user_tag: block.user_tag ?? "",
    linked_block_ids: block.linked_block_ids ?? [],
  });
}

export default function TimelinePage() {
  const { workspaceId } = useParams<{ workspaceId: string }>();
  const { showToast } = useToast();
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [allBlocks, setAllBlocks] = useState<BlockEntry[]>([]);
  const [hygiene, setHygiene] = useState<WorkspaceHygieneReport | null>(null);
  const [hygieneLoading, setHygieneLoading] = useState(true);
  const [ascending, setAscending] = useState(false);
  const [beliefFilter, setBeliefFilter] = useState<BeliefState | "">("");
  const [tagFilter, setTagFilter] = useState<BlockSystemTag | "">("");
  const [hygieneCategory, setHygieneCategory] = useState<string | null>(null);
  const [selected, setSelected] = useState<BlockEntry | null>(null);
  const [panelOpen, setPanelOpen] = useState(false);
  const [editingGoal, setEditingGoal] = useState(false);
  const [goalDraft, setGoalDraft] = useState("");
  const [goalSaving, setGoalSaving] = useState(false);
  const [prExportMode, setPrExportMode] = useState(false);
  const [selectedForPr, setSelectedForPr] = useState<Set<string>>(new Set());
  const [captureActive, setCaptureActive] = useState(false);
  const [captureScopeLabel, setCaptureScopeLabel] = useState<string | null>(null);
  const [captureMessageCount, setCaptureMessageCount] = useState(0);
  const [capturePickerOpen, setCapturePickerOpen] = useState(false);
  const [captureCandidates, setCaptureCandidates] = useState<CaptureCandidate[]>([]);
  const [rememberCaptureScope, setRememberCaptureScope] = useState(true);
  const captureEmptyWarned = useRef(false);
  const [includeTraceCheckpointsInPr, setIncludeTraceCheckpointsInPr] = useState(true);
  const [includeTraceLogInPr, setIncludeTraceLogInPr] = useState(false);
  const [includeTraceBranchLogsInPr, setIncludeTraceBranchLogsInPr] = useState(false);
  const [prExportDialogOpen, setPrExportDialogOpen] = useState(false);
  const [checkpointDialogOpen, setCheckpointDialogOpen] = useState(false);

  const load = useCallback(async (options?: { silent?: boolean }) => {
    if (!workspaceId) return;
    const silent = options?.silent ?? false;
    if (!silent) {
      setHygieneLoading(true);
    }
    try {
      const workspaces = await listWorkspaces();
      setWorkspace(workspaces.find((w) => w.id === workspaceId) ?? null);
      const [list, report, cap] = await Promise.all([
        fetchBlocks(workspaceId, ascending, silent),
        fetchWorkspaceHygiene(workspaceId),
        captureStatus(workspaceId).catch(() => null),
      ]);
      setAllBlocks(list);
      setHygiene(report);
      setCaptureActive(Boolean(cap?.active_session));
      setCaptureScopeLabel(cap?.scope_label ?? null);
      setCaptureMessageCount(cap?.session_message_count ?? 0);
    } catch (e) {
      if (!silent) showToast({ message: String(e), kind: "error" });
    } finally {
      if (!silent) setHygieneLoading(false);
    }
  }, [workspaceId, ascending, showToast]);

  useEffect(() => {
    load();
  }, [load]);

  useEffect(() => {
    if (!captureActive || captureMessageCount > 0) {
      captureEmptyWarned.current = false;
      return;
    }
    const timer = window.setTimeout(() => {
      if (captureActive && captureMessageCount === 0 && !captureEmptyWarned.current) {
        captureEmptyWarned.current = true;
        showToast({
          message:
            "Capture is on but no messages yet. Chat in the scoped thread after Start capture.",
          kind: "error",
        });
      }
    }, 30_000);
    return () => window.clearTimeout(timer);
  }, [captureActive, captureMessageCount, showToast]);

  // MCP and other writers update the same DB — refresh while viewing timeline.
  useEffect(() => {
    if (!workspaceId) return;

    const refresh = () => load({ silent: true });

    const onFocus = () => refresh();
    const onVisibility = () => {
      if (document.visibilityState === "visible") refresh();
    };

    window.addEventListener("focus", onFocus);
    document.addEventListener("visibilitychange", onVisibility);
    const interval = window.setInterval(refresh, 2000);

    return () => {
      window.removeEventListener("focus", onFocus);
      document.removeEventListener("visibilitychange", onVisibility);
      window.clearInterval(interval);
    };
  }, [workspaceId, load]);

  // Keep open block panel in sync when MCP edits the same block.
  useEffect(() => {
    if (!panelOpen || !selected) return;
    const fresh = allBlocks.find((b) => b.id === selected.id);
    if (!fresh) {
      setPanelOpen(false);
      setSelected(null);
      return;
    }
    if (blockRevision(fresh) !== blockRevision(selected)) {
      setSelected(fresh);
    }
  }, [allBlocks, panelOpen, selected]);

  let blocks = allBlocks;
  if (beliefFilter) {
    blocks = blocks.filter((b) => b.belief_state === beliefFilter);
  }
  if (tagFilter) {
    blocks = blocks.filter((b) => b.system_tag === tagFilter);
  }
  if (hygieneCategory && hygiene) {
    const key = hygieneCategory as keyof Pick<
      WorkspaceHygieneReport,
      "orphans" | "stale" | "still_open" | "dead_ends" | "decisions"
    >;
    const items = hygiene[key];
    const ids = new Set(items.map((x) => x.block_id));
    blocks = blocks.filter((b) => ids.has(b.id));
  }

  function togglePrBlock(blockId: string) {
    setSelectedForPr((prev) => {
      const next = new Set(prev);
      if (next.has(blockId)) next.delete(blockId);
      else next.add(blockId);
      return next;
    });
  }

  function selectAllForPr() {
    setSelectedForPr(new Set(blocks.map((b) => b.id)));
  }

  function clearPrSelection() {
    setSelectedForPr(new Set());
  }

  function handleExportPr() {
    if (!workspaceId) return;
    if (selectedForPr.size === 0) {
      showToast({ message: "Select at least one block for PR export", kind: "error" });
      return;
    }
    setPrExportDialogOpen(true);
  }

  async function confirmPrExport(prNumberInput: string) {
    if (!workspaceId) return;
    setPrExportDialogOpen(false);
    const prNumber = prNumberInput || undefined;
    try {
      const md = await exportPrReasoning(workspaceId, [...selectedForPr], {
        includeTraceCheckpoints: includeTraceCheckpointsInPr,
        includeTraceLog: includeTraceLogInPr,
        includeTraceBranchLogs: includeTraceBranchLogsInPr,
        prNumber,
      });
      await writeText(md);
      showToast(
        `PR reasoning export copied (${selectedForPr.size} block${selectedForPr.size === 1 ? "" : "s"})`,
      );
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }

  async function beginCapture(options?: {
    cursorProject?: string;
    transcriptPath?: string;
    rememberScope?: boolean;
  }) {
    if (!workspaceId) return;
    const result = await startCapture(workspaceId, {
      cursorProject: options?.cursorProject,
      transcriptPath: options?.transcriptPath,
      rememberScope: options?.rememberScope ?? rememberCaptureScope,
    });
    if (result.status === "needs_picker") {
      setCaptureCandidates(normalizeCaptureCandidates(result.candidates ?? []));
      setCapturePickerOpen(true);
      return;
    }
    if (result.status === "no_candidates") {
      showToast({
        message: result.hint ?? "No recent chats found. Send a message, then try again.",
        kind: "error",
      });
      return;
    }
    setCaptureActive(true);
    setCaptureScopeLabel(result.scope_label ?? null);
    captureEmptyWarned.current = false;
    const scope = result.scope_label ? ` (${result.scope_label})` : "";
    showToast(`Live capture started${scope}`);
  }

  async function handleStartCapture() {
    if (!workspaceId) return;
    try {
      await beginCapture();
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }

  async function handlePickerSelect(candidate: CaptureCandidate) {
    if (!workspaceId) return;
    setCapturePickerOpen(false);
    try {
      await beginCapture({
        cursorProject: candidate.cursor_project,
        transcriptPath: candidate.transcript_path,
        rememberScope: rememberCaptureScope,
      });
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }

  async function handleStopCapture() {
    if (!workspaceId) return;
    try {
      await stopCapture(workspaceId);
      setCaptureActive(false);
      setCaptureScopeLabel(null);
      setCaptureMessageCount(0);
      showToast("Live capture stopped");
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }

  async function confirmCheckpoint(values: CheckpointFormValues) {
    if (!workspaceId) return;
    setCheckpointDialogOpen(false);
    try {
      await commitTraceCheckpoint({
        workspaceId,
        intent: values.intent,
        note: values.note,
        rejectedPaths: values.rejectedPaths,
        blockIds: prExportMode ? [...selectedForPr] : [],
      });
      showToast("Trace checkpoint committed");
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }

  function handleCheckpoint() {
    if (!workspaceId) return;
    setCheckpointDialogOpen(true);
  }

  function openCreate() {
    if (!workspace) {
      showToast({ message: "Workspace still loading. Try again in a moment.", kind: "error" });
      return;
    }
    setSelected(null);
    setPanelOpen(true);
  }

  function openDetail(block: BlockEntry) {
    setSelected(block);
    setPanelOpen(true);
  }

  function openBlockById(blockId: string) {
    const block = allBlocks.find((b) => b.id === blockId);
    if (block) openDetail(block);
  }

  function startGoalEdit() {
    if (!workspace) return;
    setGoalDraft(workspace.goal);
    setEditingGoal(true);
  }

  async function saveGoal() {
    if (!workspace) return;
    setGoalSaving(true);
    try {
      const updated = await updateWorkspace({
        id: workspace.id,
        name: workspace.name,
        goal: goalDraft.trim(),
        template: workspace.template,
      });
      setWorkspace(updated);
      setEditingGoal(false);
      showToast("Goal updated");
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    } finally {
      setGoalSaving(false);
    }
  }

  function cancelGoalEdit() {
    setEditingGoal(false);
    setGoalDraft("");
  }

  if (!workspaceId) return null;

  return (
    <div className="flex min-h-screen">
      <PromptDialog
        open={prExportDialogOpen}
        title="Export for PR"
        message="Optional. Adds PR metadata to the export header."
        label="PR number (optional)"
        placeholder="e.g. 42"
        confirmLabel="Copy export"
        onConfirm={confirmPrExport}
        onCancel={() => setPrExportDialogOpen(false)}
      />
      <CheckpointDialog
        open={checkpointDialogOpen}
        onConfirm={confirmCheckpoint}
        onCancel={() => setCheckpointDialogOpen(false)}
      />
      <CapturePickerDialog
        open={capturePickerOpen}
        candidates={captureCandidates}
        rememberScope={rememberCaptureScope}
        onRememberScopeChange={setRememberCaptureScope}
        onSelect={(c) => void handlePickerSelect(c)}
        onCancel={() => setCapturePickerOpen(false)}
      />

      <div className="min-w-0 flex-1 px-6 py-8">
        <div className="mb-6">
          <Link
            to="/"
            className="group inline-flex cursor-pointer items-center gap-2 text-sm font-medium text-zinc-500 transition hover:text-zinc-300"
          >
            <span
              className="text-2xl leading-none text-zinc-400 transition group-hover:text-zinc-200"
              aria-hidden
            >
              ←
            </span>
            Workspaces
          </Link>
        </div>

        <header className="mb-8">
          <h1 className="text-2xl font-semibold text-zinc-50">
            {workspace?.name ?? "Workspace"}
          </h1>
          {workspace && (
            <div className="mt-2 max-w-2xl">
              {editingGoal ? (
                <div className="space-y-2">
                  <textarea
                    value={goalDraft}
                    onChange={(e) => setGoalDraft(e.target.value)}
                    rows={3}
                    autoFocus
                    className="w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
                    placeholder={
                      workspace
                        ? placeholdersForTemplate(workspace.template).goal
                        : "What change or PR are you reasoning through?"
                    }
                  />
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={saveGoal}
                      disabled={goalSaving || !goalDraft.trim()}
                      className="cursor-pointer rounded-lg bg-emerald-600 px-3 py-1.5 text-sm text-white hover:bg-emerald-500 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      {goalSaving ? "Saving…" : "Save goal"}
                    </button>
                    <button
                      type="button"
                      onClick={cancelGoalEdit}
                      className="cursor-pointer rounded-lg border border-zinc-700 px-3 py-1.5 text-sm text-zinc-400 hover:border-zinc-500"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <div className="flex items-start gap-2">
                  {workspace.goal ? (
                    <p className="text-sm text-zinc-400">{workspace.goal}</p>
                  ) : (
                    <p className="text-sm italic text-zinc-600">No goal set</p>
                  )}
                  <button
                    type="button"
                    onClick={startGoalEdit}
                    title="Edit goal"
                    aria-label="Edit goal"
                    className="mt-0.5 shrink-0 cursor-pointer rounded p-1 text-zinc-500 hover:bg-zinc-800 hover:text-zinc-300"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      viewBox="0 0 20 20"
                      fill="currentColor"
                      className="h-4 w-4"
                      aria-hidden
                    >
                      <path d="m2.695 14.763-1.262 3.154a.5.5 0 0 0 .65.65l3.155-1.262a4 4 0 0 0 1.343-.885L17.5 5.5a2.121 2.121 0 0 0-3-3L3.58 13.42a4 4 0 0 0-.885 1.343Z" />
                    </svg>
                  </button>
                </div>
              )}
            </div>
          )}
          {captureActive && (
            <p className="mt-2 text-sm text-rose-200/90">
              Capturing
              {captureScopeLabel ? `: ${captureScopeLabel}` : ""}
              {captureMessageCount > 0
                ? ` · ${captureMessageCount} new message${captureMessageCount === 1 ? "" : "s"}`
                : " · waiting for new messages"}
            </p>
          )}
        </header>

        {hygieneCategory && (
          <p className="mb-4 text-sm text-orange-300">
            Filtering timeline by hygiene: {hygieneCategoryLabel(hygieneCategory)}
            <button
              type="button"
              onClick={() => setHygieneCategory(null)}
              className="ml-2 cursor-pointer underline hover:text-orange-200"
            >
              Clear
            </button>
          </p>
        )}

        <div className="mb-6 flex flex-wrap items-center gap-2.5">
          <select
            value={beliefFilter}
            onChange={(e) => setBeliefFilter(e.target.value as BeliefState | "")}
            className="select-filter min-w-[9.5rem] cursor-pointer"
          >
            <option value="">All beliefs</option>
            <option value="open">Open</option>
            <option value="leaning_true">Leaning True</option>
            <option value="leaning_false">Leaning False</option>
            <option value="confirmed">Confirmed</option>
            <option value="rejected">Rejected</option>
          </select>
          <select
            value={tagFilter}
            onChange={(e) => setTagFilter(e.target.value as BlockSystemTag | "")}
            className="select-filter min-w-[9.5rem] cursor-pointer"
          >
            <option value="">All tags</option>
            <option value="needs_review">Needs Review</option>
            <option value="ruled_out">Ruled Out</option>
            <option value="reportable">Reportable</option>
            <option value="reasoning_debt">Reasoning Debt</option>
            <option value="stale">Stale</option>
          </select>
          <button
            type="button"
            onClick={() => setAscending(!ascending)}
            className="cursor-pointer rounded-lg border border-zinc-700 px-3.5 py-2 text-sm text-zinc-300 hover:border-zinc-500 hover:text-zinc-100"
          >
            {ascending ? "Oldest first" : "Newest first"}
          </button>
          <button
            type="button"
            onClick={() => {
              setPrExportMode((v) => !v);
              if (prExportMode) clearPrSelection();
            }}
            className={`cursor-pointer rounded-lg border px-3.5 py-2 text-sm ${
              prExportMode
                ? "border-violet-600 bg-violet-950/40 text-violet-200"
                : "border-zinc-700 text-zinc-300 hover:border-zinc-500 hover:text-zinc-100"
            }`}
          >
            {prExportMode ? "PR export on" : "PR export"}
          </button>
          {prExportMode && (
            <>
              <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-400">
                <input
                  type="checkbox"
                  checked={includeTraceCheckpointsInPr}
                  onChange={(e) => setIncludeTraceCheckpointsInPr(e.target.checked)}
                  className="rounded border-zinc-600"
                />
                Session trace: checkpoints
              </label>
              <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-400">
                <input
                  type="checkbox"
                  checked={includeTraceLogInPr}
                  onChange={(e) => setIncludeTraceLogInPr(e.target.checked)}
                  className="rounded border-zinc-600"
                />
                Session trace: raw log
              </label>
              <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-400">
                <input
                  type="checkbox"
                  checked={includeTraceBranchLogsInPr}
                  onChange={(e) => setIncludeTraceBranchLogsInPr(e.target.checked)}
                  className="rounded border-zinc-600"
                />
                Session trace: branch logs
              </label>
              <button
                type="button"
                onClick={selectAllForPr}
                className="cursor-pointer rounded-lg border border-zinc-700 px-3 py-2 text-sm text-zinc-400 hover:border-zinc-500"
              >
                Select all
              </button>
              <button
                type="button"
                onClick={clearPrSelection}
                disabled={selectedForPr.size === 0}
                className="cursor-pointer rounded-lg border border-zinc-700 px-3 py-2 text-sm text-zinc-400 hover:border-zinc-500 disabled:cursor-not-allowed disabled:opacity-40"
              >
                Clear
              </button>
              <button
                type="button"
                onClick={handleExportPr}
                disabled={selectedForPr.size === 0}
                className="cursor-pointer rounded-lg bg-violet-600 px-3.5 py-2 text-sm font-medium text-white hover:bg-violet-500 disabled:cursor-not-allowed disabled:opacity-50"
              >
                Export for PR ({selectedForPr.size})
              </button>
            </>
          )}
          <button
            type="button"
            onClick={captureActive ? handleStopCapture : handleStartCapture}
            className={`cursor-pointer rounded-lg border px-3.5 py-2 text-sm ${
              captureActive
                ? "border-rose-800/80 bg-rose-950/30 text-rose-100 hover:border-rose-600"
                : "border-zinc-700 text-zinc-300 hover:border-zinc-500"
            }`}
          >
            {captureActive ? "Stop capture" : "Start capture"}
          </button>
          <button
            type="button"
            onClick={handleCheckpoint}
            className="cursor-pointer rounded-lg border border-amber-800/80 bg-amber-950/30 px-3.5 py-2 text-sm text-amber-100 hover:border-amber-600"
          >
            Checkpoint
          </button>
          <Link
            to="/help"
            className="inline-flex cursor-pointer items-center rounded-lg border border-zinc-700 px-3.5 py-2 text-sm text-zinc-300 hover:border-zinc-500 hover:text-zinc-100"
          >
            Help
          </Link>
        </div>

        <div className="mb-4">
          <button
            type="button"
            onClick={openCreate}
            className="cursor-pointer rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
          >
            + Add block
          </button>
        </div>

        <ul className="space-y-3">
          {blocks.length === 0 ? (
            <li className="text-sm text-zinc-500">No blocks match this view.</li>
          ) : (
            blocks.map((block) => (
              <li key={block.id} className="flex items-stretch gap-2">
                {prExportMode && (
                  <label className="flex cursor-pointer items-center px-1">
                    <input
                      type="checkbox"
                      checked={selectedForPr.has(block.id)}
                      onChange={() => togglePrBlock(block.id)}
                      className="h-4 w-4 cursor-pointer rounded border-zinc-600 bg-zinc-900 text-violet-500"
                      aria-label={`Select ${block.title || "block"} for PR export`}
                    />
                  </label>
                )}
                <button
                  type="button"
                  onClick={() => openDetail(block)}
                  className="min-w-0 flex-1 cursor-pointer rounded-xl border border-zinc-800 bg-zinc-900/40 px-4 py-3 text-left transition hover:border-zinc-600"
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <span
                      className={`rounded-full border px-2 py-0.5 text-xs ${BELIEF_COLORS[block.belief_state]}`}
                    >
                      {beliefStateLabel(block.belief_state)}
                    </span>
                    {block.system_tag !== "none" && (
                      <span className="rounded-full bg-sky-950/60 px-2 py-0.5 text-xs text-sky-200">
                        {systemTagLabel(block.system_tag)}
                      </span>
                    )}
                    {block.user_tag && (
                      <span className="rounded-full bg-zinc-800 px-2 py-0.5 text-xs text-zinc-400">
                        {block.user_tag}
                      </span>
                    )}
                    {block.incomplete && (
                      <span className="rounded-full bg-orange-950/60 px-2 py-0.5 text-xs text-orange-300">
                        Incomplete
                      </span>
                    )}
                    {block.linked_block_ids.length > 0 && (
                      <span className="text-xs text-zinc-500">
                        → {block.linked_block_ids.length} link
                        {block.linked_block_ids.length > 1 ? "s" : ""}
                      </span>
                    )}
                    <span className="ml-auto text-xs text-zinc-600">
                      {new Date(block.updated_at).toLocaleString()}
                    </span>
                  </div>

                  <p className="mt-2 text-sm font-medium text-zinc-100 line-clamp-2">
                    {block.title || blockPreview(block)}
                  </p>
                  {block.title && block.hypothesis && (
                    <p className="mt-1 text-xs text-zinc-500 line-clamp-1">
                      {block.hypothesis.text}
                    </p>
                  )}

                  <div className="mt-2 flex flex-wrap gap-2">
                    {block.hypothesis && (
                      <span className="rounded border border-violet-900/50 px-2 py-0.5 text-xs font-medium text-violet-300">
                        {workspace ? hypothesisFieldLabel(workspace.template) : "Hypothesis"}
                      </span>
                    )}
                    {block.action && (
                      <span className="rounded border border-sky-900/50 px-2 py-0.5 text-xs font-medium text-sky-300">
                        Action
                      </span>
                    )}
                    {block.evidence && (
                      <span className="rounded border border-amber-900/50 px-2 py-0.5 text-xs font-medium text-amber-300">
                        Evidence
                      </span>
                    )}
                    {block.conclusion && (
                      <span className="rounded border border-emerald-900/50 px-2 py-0.5 text-xs font-medium text-emerald-300">
                        Conclusion
                      </span>
                    )}
                  </div>
                </button>
              </li>
            ))
          )}
        </ul>
      </div>

      <HygienePanel
        report={hygiene}
        loading={hygieneLoading}
        activeCategory={hygieneCategory}
        onSelectCategory={(cat) => {
          setHygieneCategory(cat);
        }}
        onSelectBlock={openBlockById}
      />

      {panelOpen && workspace && (
        <BlockPanel
          key={selected ? `${selected.id}:${blockRevision(selected)}` : "new"}
          workspace={workspace}
          block={selected}
          onClose={() => {
            setPanelOpen(false);
            setSelected(null);
          }}
          onSaved={() => {
            setPanelOpen(false);
            setSelected(null);
            load();
          }}
        />
      )}

    </div>
  );
}
