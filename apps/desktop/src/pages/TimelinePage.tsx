import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  exportWorkspaceSummary,
  fetchBlocks,
  fetchWorkspaceHygiene,
  listWorkspaces,
} from "../api";
import BlockPanel from "../components/BlockPanel";
import HygienePanel from "../components/HygienePanel";
import {
  beliefStateLabel,
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
  return (
    block.hypothesis?.text ??
    block.action?.text ??
    block.evidence?.text ??
    block.conclusion?.text ??
    "(empty block)"
  );
}

export default function TimelinePage() {
  const { workspaceId } = useParams<{ workspaceId: string }>();
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
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    if (!workspaceId) return;
    setError("");
    setHygieneLoading(true);
    try {
      const workspaces = await listWorkspaces();
      setWorkspace(workspaces.find((w) => w.id === workspaceId) ?? null);
      const [list, report] = await Promise.all([
        fetchBlocks(workspaceId, ascending),
        fetchWorkspaceHygiene(workspaceId),
      ]);
      setAllBlocks(list);
      setHygiene(report);
    } catch (e) {
      setError(String(e));
    } finally {
      setHygieneLoading(false);
    }
  }, [workspaceId, ascending]);

  useEffect(() => {
    load();
  }, [load]);

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

  async function handleExport() {
    if (!workspaceId) return;
    try {
      const md = await exportWorkspaceSummary(workspaceId);
      await writeText(md);
      setStatus("Summary copied to clipboard");
    } catch (e) {
      setError(String(e));
    }
  }

  function openCreate() {
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

  if (!workspaceId) return null;

  return (
    <div className="flex min-h-screen">
      <div className="min-w-0 flex-1 px-6 py-8">
        <div className="mb-6 flex flex-wrap items-center gap-3">
          <Link
            to="/"
            className="cursor-pointer text-sm text-zinc-500 hover:text-zinc-300"
          >
            ← Workspaces
          </Link>
        </div>

        <header className="mb-8">
          <h1 className="text-2xl font-semibold text-zinc-50">
            {workspace?.name ?? "Workspace"}
          </h1>
          {workspace?.goal && (
            <p className="mt-2 max-w-2xl text-sm text-zinc-400">{workspace.goal}</p>
          )}
        </header>

        {error && (
          <p className="mb-4 rounded-lg border border-red-900/50 bg-red-950/40 px-4 py-3 text-sm text-red-300">
            {error}
          </p>
        )}
        {status && (
          <p className="mb-4 rounded-lg border border-emerald-900/50 bg-emerald-950/30 px-4 py-3 text-sm text-emerald-300">
            {status}
          </p>
        )}

        {hygieneCategory && (
          <p className="mb-4 text-xs text-orange-300">
            Filtering timeline by hygiene: {hygieneCategory.replace("_", " ")}
            <button
              type="button"
              onClick={() => setHygieneCategory(null)}
              className="ml-2 cursor-pointer underline hover:text-orange-200"
            >
              Clear
            </button>
          </p>
        )}

        <div className="mb-6 flex flex-wrap items-center gap-2">
          <select
            value={beliefFilter}
            onChange={(e) => setBeliefFilter(e.target.value as BeliefState | "")}
            className="cursor-pointer rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-1.5 text-xs text-zinc-300"
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
            className="cursor-pointer rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-1.5 text-xs text-zinc-300"
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
            className="cursor-pointer rounded-lg border border-zinc-700 px-3 py-1 text-xs text-zinc-400 hover:border-zinc-500"
          >
            {ascending ? "Oldest first" : "Newest first"}
          </button>
          <button
            type="button"
            onClick={handleExport}
            className="ml-auto cursor-pointer rounded-lg border border-zinc-700 px-3 py-1.5 text-xs text-zinc-300 hover:border-zinc-500"
          >
            Copy summary
          </button>
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
              <li key={block.id}>
                <button
                  type="button"
                  onClick={() => openDetail(block)}
                  className="w-full cursor-pointer rounded-xl border border-zinc-800 bg-zinc-900/40 px-4 py-3 text-left transition hover:border-zinc-600"
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
                        incomplete
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
                    {blockPreview(block)}
                  </p>

                  <div className="mt-2 flex flex-wrap gap-1.5">
                    {block.hypothesis && (
                      <span className="rounded border border-violet-900/50 px-1.5 py-0.5 text-[10px] text-violet-300">
                        Hypothesis
                      </span>
                    )}
                    {block.action && (
                      <span className="rounded border border-sky-900/50 px-1.5 py-0.5 text-[10px] text-sky-300">
                        Action
                      </span>
                    )}
                    {block.evidence && (
                      <span className="rounded border border-amber-900/50 px-1.5 py-0.5 text-[10px] text-amber-300">
                        Evidence
                      </span>
                    )}
                    {block.conclusion && (
                      <span className="rounded border border-emerald-900/50 px-1.5 py-0.5 text-[10px] text-emerald-300">
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

      {panelOpen && workspace && (
        <BlockPanel
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

      <HygienePanel
        report={hygiene}
        loading={hygieneLoading}
        activeCategory={hygieneCategory}
        onSelectCategory={(cat) => {
          setHygieneCategory(cat);
        }}
        onSelectBlock={openBlockById}
      />
    </div>
  );
}
