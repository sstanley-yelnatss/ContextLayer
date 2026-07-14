import { useCallback, useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Camera, Download, Pencil, Plus, Zap } from "lucide-react";
import {
  captureStatus,
  commitTraceCheckpoint,
  exportPrReasoning,
  fetchBlocks,
  fetchSessionGraph,
  fetchWorkspaceHygiene,
  listWorkspaces,
  normalizeCaptureCandidates,
  startCapture,
  stopCapture,
  TRACE_LOG_SLICE_OPTIONS,
  updateWorkspace,
  type CaptureCandidate,
  type TraceLogSlice,
} from "../api";
import BlockPanel from "../components/BlockPanel";
import CapturePickerDialog from "../components/CapturePickerDialog";
import CheckpointDialog, {
  type CheckpointFormValues,
} from "../components/CheckpointDialog";
import HygienePanel from "../components/HygienePanel";
import PromptDialog from "../components/PromptDialog";
import SessionGraphDetailPanel from "../components/SessionGraphDetailPanel";
import SessionGraphView from "../components/SessionGraphView";
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
  type SessionGraph,
  type SessionGraphRow,
  type Workspace,
  type WorkspaceHygieneReport,
} from "../types";

const BELIEF_BADGE_CLASS: Record<BeliefState, string> = {
  open: "cl-belief-open",
  leaning_true: "cl-belief-leaning-true",
  leaning_false: "cl-belief-leaning-false",
  confirmed: "cl-belief-confirmed",
  rejected: "cl-belief-rejected",
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
  const [traceLogSliceInPr, setTraceLogSliceInPr] = useState<TraceLogSlice>("past_50");
  const [captureLogBoundaryAvailable, setCaptureLogBoundaryAvailable] = useState(false);
  const [includeTraceBranchLogsInPr, setIncludeTraceBranchLogsInPr] = useState(false);
  const [prExportDialogOpen, setPrExportDialogOpen] = useState(false);
  const [checkpointDialogOpen, setCheckpointDialogOpen] = useState(false);
  const [workspaceView, setWorkspaceView] = useState<"timeline" | "session">("timeline");
  const [sessionGraph, setSessionGraph] = useState<SessionGraph | null>(null);
  const [sessionLoading, setSessionLoading] = useState(false);
  const [selectedSessionRow, setSelectedSessionRow] = useState<SessionGraphRow | null>(null);

  const loadSessionGraph = useCallback(async () => {
    if (!workspaceId) return;
    setSessionLoading(true);
    try {
      const graph = await fetchSessionGraph(workspaceId);
      setSessionGraph(graph);
      setSelectedSessionRow((prev) =>
        prev ? (graph.rows.find((r) => r.id === prev.id) ?? null) : null,
      );
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    } finally {
      setSessionLoading(false);
    }
  }, [workspaceId, showToast]);

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
      setCaptureLogBoundaryAvailable(Boolean(cap?.capture_log_boundary_available));
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
    if (workspaceView !== "session" || !workspaceId) return;
    void loadSessionGraph();
  }, [workspaceView, workspaceId, loadSessionGraph]);

  useEffect(() => {
    if (workspaceView !== "session") {
      setSelectedSessionRow(null);
    }
  }, [workspaceView]);

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

  useEffect(() => {
    if (
      traceLogSliceInPr === "since_last_capture_start" &&
      !captureLogBoundaryAvailable
    ) {
      setTraceLogSliceInPr("past_50");
    }
  }, [captureLogBoundaryAvailable, traceLogSliceInPr]);

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
        traceLogSlice: traceLogSliceInPr,
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
    if (workspaceView === "session") void loadSessionGraph();
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
      if (workspaceView === "session") void loadSessionGraph();
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
      if (workspaceView === "session") void loadSessionGraph();
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
    <div className="flex h-full min-h-0">
      <PromptDialog
        open={prExportDialogOpen}
        title="Export for PR"
        message="Optional. Adds PR metadata to the export header."
        label="PR number (optional)"
        placeholder="e.g. 42"
        confirmLabel="Copy export"
        onConfirm={confirmPrExport}
        onCancel={() => setPrExportDialogOpen(false)}
      >
        <div className="mt-4 space-y-2 border-t border-border pt-4">
          <p className="font-mono-ui text-[10px] font-medium uppercase tracking-widest text-muted-foreground">
            Session trace
          </p>
          <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              checked={includeTraceCheckpointsInPr}
              onChange={(e) => setIncludeTraceCheckpointsInPr(e.target.checked)}
              className="rounded border-border"
            />
            Include checkpoints
          </label>
          <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              checked={includeTraceLogInPr}
              onChange={(e) => setIncludeTraceLogInPr(e.target.checked)}
              className="rounded border-border"
            />
            Include raw log
          </label>
          {includeTraceLogInPr && (
            <label className="flex items-center gap-2 text-sm text-muted-foreground">
              <span className="shrink-0">Log slice</span>
              <select
                value={traceLogSliceInPr}
                onChange={(e) => setTraceLogSliceInPr(e.target.value as TraceLogSlice)}
                className="select-filter min-w-0 flex-1 py-1.5 text-xs"
              >
                {TRACE_LOG_SLICE_OPTIONS.map((opt) => {
                  const disabled =
                    "requiresCaptureBoundary" in opt &&
                    opt.requiresCaptureBoundary &&
                    !captureLogBoundaryAvailable;
                  return (
                    <option key={opt.value} value={opt.value} disabled={disabled}>
                      {opt.label}
                      {disabled ? " (capture first)" : ""}
                    </option>
                  );
                })}
              </select>
            </label>
          )}
          <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
            <input
              type="checkbox"
              checked={includeTraceBranchLogsInPr}
              onChange={(e) => setIncludeTraceBranchLogsInPr(e.target.checked)}
              className="rounded border-border"
            />
            Include branch logs
          </label>
        </div>
      </PromptDialog>
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

      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <header className="shrink-0 border-b border-border px-6 pb-4 pt-5">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0 flex-1">
              <h1 className="font-mono-ui truncate text-[17px] font-semibold tracking-tight text-foreground">
                {workspace?.name ?? "Workspace"}
              </h1>
              {workspace && (
                <div className="mt-1 max-w-2xl">
                  {editingGoal ? (
                    <div className="space-y-2">
                      <textarea
                        value={goalDraft}
                        onChange={(e) => setGoalDraft(e.target.value)}
                        rows={3}
                        autoFocus
                        className="w-full rounded-[3px] border border-border bg-input-background px-3 py-2 text-sm text-foreground"
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
                          className="cl-btn-export disabled:cursor-not-allowed disabled:opacity-50"
                        >
                          {goalSaving ? "Saving…" : "Save goal"}
                        </button>
                        <button type="button" onClick={cancelGoalEdit} className="cl-btn-ghost">
                          Cancel
                        </button>
                      </div>
                    </div>
                  ) : (
                    <div className="flex items-start gap-2">
                      {workspace.goal ? (
                        <p className="text-[12px] text-muted-foreground">{workspace.goal}</p>
                      ) : (
                        <p className="text-[12px] italic text-muted-foreground/70">No goal set</p>
                      )}
                      <button
                        type="button"
                        onClick={startGoalEdit}
                        title="Edit goal"
                        aria-label="Edit goal"
                        className="mt-0.5 shrink-0 rounded-[3px] p-1 text-muted-foreground transition-colors hover:bg-[rgba(255,255,255,0.06)] hover:text-foreground"
                      >
                        <Pencil size={12} aria-hidden />
                      </button>
                    </div>
                  )}
                </div>
              )}
              {captureActive && (
                <p className="mt-2 font-mono-ui text-[11px]" style={{ color: "var(--hygiene-warn)" }}>
                  Capturing
                  {captureScopeLabel ? `: ${captureScopeLabel}` : ""}
                  {captureMessageCount > 0
                    ? ` · ${captureMessageCount} new message${captureMessageCount === 1 ? "" : "s"}`
                    : " · waiting for new messages"}
                </p>
              )}
            </div>

            <div className="mt-1 flex shrink-0 items-center gap-2">
              <button type="button" onClick={handleCheckpoint} className="cl-btn-ghost cl-btn-toolbar">
                <Camera size={13} />
                Checkpoint
              </button>
              <button
                type="button"
                onClick={captureActive ? handleStopCapture : handleStartCapture}
                className={`cl-btn-toolbar ${captureActive ? "cl-btn-capture-active" : "cl-btn-accent"}`}
              >
                <Zap size={13} />
                {captureActive ? "Stop capture" : "Start capture"}
              </button>
              <button
                type="button"
                onClick={() => {
                  setPrExportMode((v) => !v);
                  if (prExportMode) clearPrSelection();
                }}
                className={`cl-btn-toolbar ${prExportMode ? "cl-btn-export ring-1 ring-[rgba(52,211,153,0.35)]" : "cl-btn-export"}`}
              >
                <Download size={13} />
                {prExportMode ? "Export mode on" : "Export PR"}
              </button>
            </div>
          </div>

          <div className="mt-4 flex items-center gap-1 rounded-[3px] border border-border bg-[rgba(255,255,255,0.02)] p-0.5 w-fit">
            <button
              type="button"
              onClick={() => setWorkspaceView("timeline")}
              className={`rounded-[2px] px-3 py-1.5 text-[12px] font-medium transition-colors ${
                workspaceView === "timeline"
                  ? "bg-[rgba(255,255,255,0.08)] text-foreground"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Timeline
            </button>
            <button
              type="button"
              onClick={() => setWorkspaceView("session")}
              className={`rounded-[2px] px-3 py-1.5 text-[12px] font-medium transition-colors ${
                workspaceView === "session"
                  ? "bg-[rgba(255,255,255,0.08)] text-foreground"
                  : "text-muted-foreground hover:text-foreground"
              }`}
            >
              Session
            </button>
          </div>

          {workspaceView === "timeline" && (
          <>
          <div className="mt-4 flex flex-wrap items-center gap-2">
            <select
              value={beliefFilter}
              onChange={(e) => setBeliefFilter(e.target.value as BeliefState | "")}
              className="select-filter min-w-[9.5rem]"
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
              className="select-filter min-w-[9.5rem]"
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
              className="cl-btn-ghost"
            >
              {ascending ? "Oldest first" : "Newest first"}
            </button>
          </div>

          {hygieneCategory && (
            <p className="mt-3 text-sm" style={{ color: "var(--hygiene-warn)" }}>
              Filtering timeline by hygiene: {hygieneCategoryLabel(hygieneCategory)}
              <button
                type="button"
                onClick={() => setHygieneCategory(null)}
                className="ml-2 underline hover:opacity-80"
              >
                Clear
              </button>
            </p>
          )}

          {prExportMode && (
            <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-border pt-3">
              <span className="font-mono-ui text-[10px] uppercase tracking-widest text-muted-foreground">
                PR export · {selectedForPr.size} selected
              </span>
              <button type="button" onClick={selectAllForPr} className="cl-btn-ghost">
                Select all
              </button>
              <button
                type="button"
                onClick={clearPrSelection}
                disabled={selectedForPr.size === 0}
                className="cl-btn-ghost disabled:cursor-not-allowed disabled:opacity-40"
              >
                Clear
              </button>
              <button
                type="button"
                onClick={handleExportPr}
                disabled={selectedForPr.size === 0}
                className="cl-btn-export disabled:cursor-not-allowed disabled:opacity-50"
              >
                Copy export ({selectedForPr.size})
              </button>
            </div>
          )}
          </>
          )}
        </header>

        {workspaceView === "timeline" ? (
        <div className="flex-1 overflow-y-auto px-6 py-5">
          <button
            type="button"
            onClick={openCreate}
            className="mb-3 flex w-full items-center gap-2 rounded-[3px] border border-dashed border-border px-3 py-[9px] text-[12px] text-muted-foreground transition-colors hover:border-[rgba(255,255,255,0.18)] hover:text-foreground"
          >
            <Plus size={12} />
            <span>Add block</span>
          </button>

          <ul className="space-y-2">
            {blocks.length === 0 ? (
              <li className="text-sm text-muted-foreground">No blocks match this view.</li>
            ) : (
              blocks.map((block) => (
                <li key={block.id} className="flex items-stretch gap-2">
                  {prExportMode && (
                    <label className="flex cursor-pointer items-center px-1">
                      <input
                        type="checkbox"
                        checked={selectedForPr.has(block.id)}
                        onChange={() => togglePrBlock(block.id)}
                        className="h-4 w-4 cursor-pointer rounded border-border bg-input-background accent-[var(--belief-confirmed)]"
                        aria-label={`Select ${block.title || "block"} for PR export`}
                      />
                    </label>
                  )}
                  <button
                    type="button"
                    onClick={() => openDetail(block)}
                    className="cl-surface-card min-w-0 flex-1 px-4 py-3 text-left transition-colors hover:bg-[#161619]"
                  >
                    <div className="flex flex-wrap items-center gap-2">
                      <span
                        className={`cl-belief-badge ${BELIEF_BADGE_CLASS[block.belief_state]}`}
                      >
                        {beliefStateLabel(block.belief_state)}
                      </span>
                      {block.system_tag !== "none" && (
                        <span
                          className="font-mono-ui rounded-[3px] border px-2 py-0.5 text-[10px] uppercase tracking-wide"
                          style={{
                            background: "rgba(251, 113, 133, 0.08)",
                            borderColor: "rgba(251, 113, 133, 0.2)",
                            color: "var(--hygiene-warn)",
                          }}
                        >
                          {systemTagLabel(block.system_tag)}
                        </span>
                      )}
                      {block.user_tag && (
                        <span className="font-mono-ui rounded-[3px] border border-border bg-[rgba(255,255,255,0.03)] px-2 py-0.5 text-[10px] text-muted-foreground">
                          {block.user_tag}
                        </span>
                      )}
                      {block.incomplete && (
                        <span
                          className="font-mono-ui rounded-[3px] border px-2 py-0.5 text-[10px] uppercase tracking-wide"
                          style={{
                            background: "rgba(251, 146, 60, 0.08)",
                            borderColor: "rgba(251, 146, 60, 0.2)",
                            color: "var(--belief-leaning-false)",
                          }}
                        >
                          Incomplete
                        </span>
                      )}
                      {block.linked_block_ids.length > 0 && (
                        <span className="font-mono-ui text-[10px] text-muted-foreground">
                          → {block.linked_block_ids.length} link
                          {block.linked_block_ids.length > 1 ? "s" : ""}
                        </span>
                      )}
                      <span className="font-mono-ui ml-auto text-[10px] text-muted-foreground/80">
                        {new Date(block.updated_at).toLocaleString()}
                      </span>
                    </div>

                    <p className="mt-2 line-clamp-2 text-sm font-medium text-foreground">
                      {block.title || blockPreview(block)}
                    </p>
                    {block.title && block.hypothesis && (
                      <p className="mt-1 line-clamp-1 text-xs text-muted-foreground">
                        {block.hypothesis.text}
                      </p>
                    )}

                    <div className="mt-2 flex flex-wrap gap-1.5">
                      {block.hypothesis && (
                        <span className="cl-field-label">
                          {workspace ? hypothesisFieldLabel(workspace.template) : "Hypothesis"}
                        </span>
                      )}
                      {block.action && <span className="cl-field-label">Action</span>}
                      {block.evidence && <span className="cl-field-label">Evidence</span>}
                      {block.conclusion && <span className="cl-field-label">Conclusion</span>}
                    </div>
                  </button>
                </li>
              ))
            )}
          </ul>
        </div>
        ) : (
        <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
          {sessionLoading && !sessionGraph ? (
            <p className="px-6 py-8 text-sm text-muted-foreground">Loading session graph…</p>
          ) : sessionGraph ? (
            <SessionGraphView
              graph={sessionGraph}
              selectedRowId={selectedSessionRow?.id ?? null}
              onSelectRow={setSelectedSessionRow}
              onStartCapture={handleStartCapture}
            />
          ) : (
            <p className="px-6 py-8 text-sm text-muted-foreground">Could not load session graph.</p>
          )}
        </div>
        )}
      </div>

      {workspaceView === "timeline" ? (
      <HygienePanel
        report={hygiene}
        loading={hygieneLoading}
        activeCategory={hygieneCategory}
        onSelectCategory={(cat) => {
          setHygieneCategory(cat);
        }}
        onSelectBlock={openBlockById}
      />
      ) : selectedSessionRow && workspaceId ? (
      <SessionGraphDetailPanel
        workspaceId={workspaceId}
        row={selectedSessionRow}
        lane={sessionGraph?.lanes.find((l) => l.id === selectedSessionRow.lane)}
        onClose={() => setSelectedSessionRow(null)}
        onOpenBlock={openBlockById}
      />
      ) : null}

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
