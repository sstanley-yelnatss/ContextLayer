import { useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import ConfirmDialog from "../components/ConfirmDialog";
import { useToast } from "../components/Toast";
import {
  createWorkspace,
  initDatabase,
  listWorkspaces,
  setWorkspaceArchived,
} from "../api";
import {
  CREATE_WORKSPACE_TEMPLATES,
  placeholdersForTemplate,
  templateLabel,
  type Workspace,
  type WorkspaceTemplate,
} from "../types";

export default function WorkspaceListPage() {
  const { showToast } = useToast();
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [name, setName] = useState("");
  const [goal, setGoal] = useState("");
  const [template, setTemplate] = useState<WorkspaceTemplate>("agent_devops");
  const [archiveConfirm, setArchiveConfirm] = useState<Workspace | null>(null);

  async function load(includeArchived = showArchived) {
    setLoading(true);
    try {
      await initDatabase();
      setWorkspaces(await listWorkspaces(includeArchived));
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load(showArchived);
  }, [showArchived]);

  const filteredWorkspaces = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return workspaces;
    return workspaces.filter(
      (ws) =>
        ws.name.toLowerCase().includes(q) ||
        ws.goal.toLowerCase().includes(q) ||
        templateLabel(ws.template).toLowerCase().includes(q),
    );
  }, [workspaces, searchQuery]);

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    const trimmedName = name.trim();
    try {
      await createWorkspace(trimmedName, goal, template);
      setName("");
      setGoal("");
      setTemplate("agent_devops");
      setShowForm(false);
      showToast(`Workspace "${trimmedName}" created`);
      await load(showArchived);
    } catch (err) {
      showToast({ message: String(err), kind: "error" });
    }
  }

  async function confirmArchiveToggle() {
    if (!archiveConfirm) return;
    const ws = archiveConfirm;
    const archived = !ws.archived_at;
    setArchiveConfirm(null);
    try {
      await setWorkspaceArchived(ws.id, archived);
      await load(showArchived);
      showToast(
        archived
          ? `Archived "${ws.name}". Turn on Show archived to restore it.`
          : `Restored "${ws.name}" to your workspace list.`,
      );
    } catch (err) {
      showToast({ message: String(err), kind: "error" });
    }
  }

  const goalPlaceholder = placeholdersForTemplate(template).goal;
  const pendingArchive = archiveConfirm;
  const archiving = pendingArchive ? !pendingArchive.archived_at : false;

  return (
    <div className="mx-auto max-w-3xl px-6 py-10">
      <ConfirmDialog
        open={Boolean(pendingArchive)}
        title={archiving ? "Archive workspace?" : "Restore workspace?"}
        message={
          archiving
            ? `"${pendingArchive?.name}" will move off the main list. You can restore it anytime from Show archived.`
            : `"${pendingArchive?.name}" will show up in your main workspace list again.`
        }
        confirmLabel={archiving ? "Archive" : "Restore"}
        onConfirm={confirmArchiveToggle}
        onCancel={() => setArchiveConfirm(null)}
      />

      <header className="mb-10">
        <div className="flex items-start justify-between gap-4">
          <h1 className="text-3xl font-semibold tracking-tight text-zinc-50">
            ContextLayer
          </h1>
          <Link
            to="/help"
            className="shrink-0 cursor-pointer rounded-lg border border-zinc-700 px-3 py-1.5 text-sm text-zinc-400 hover:border-zinc-500 hover:text-zinc-200"
          >
            Help
          </Link>
        </div>
        <p className="mt-2 max-w-2xl text-base leading-relaxed text-zinc-400">
          Local workspaces for AI change governance. Each timeline tracks what you
          assume, what you tried, what you observed, and what you decided, then
          exports a reasoning receipt for PR review.
        </p>
        <p className="mt-3 max-w-2xl text-base leading-relaxed text-zinc-400">
          Optional capture records AI chat while you work in Cursor. Checkpoints
          mark decision moments, and you can attach session trace to the same PR
          export. The health panel surfaces open loops, stale threads, and dead
          ends.
        </p>
      </header>

      <div className="mb-4 flex flex-wrap items-center gap-3">
        <input
          type="search"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search workspaces…"
          className="min-w-[12rem] flex-1 rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-600"
        />
        <label className="flex cursor-pointer items-center gap-2 text-sm text-zinc-400">
          <input
            type="checkbox"
            checked={showArchived}
            onChange={(e) => setShowArchived(e.target.checked)}
            className="rounded border-zinc-600"
          />
          Show archived
        </label>
      </div>

      <div className="mb-6 flex items-center justify-between">
        <h2 className="text-lg font-medium text-zinc-200">Workspaces</h2>
        <button
          type="button"
          onClick={() => setShowForm(!showForm)}
          className="cursor-pointer rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
        >
          {showForm ? "Cancel" : "New workspace"}
        </button>
      </div>

      {showForm && (
        <form
          onSubmit={handleCreate}
          className="mb-8 rounded-xl border border-zinc-800 bg-zinc-900/60 p-5"
        >
          <label className="block text-sm text-zinc-400">
            Name
            <input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-zinc-100"
            />
          </label>
          <label className="mt-4 block text-sm text-zinc-400">
            Goal (required)
            <textarea
              required
              value={goal}
              onChange={(e) => setGoal(e.target.value)}
              rows={3}
              placeholder={goalPlaceholder}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-zinc-100"
            />
          </label>
          <label className="mt-4 block text-sm text-zinc-400">
            Template
            <select
              value={template}
              onChange={(e) => setTemplate(e.target.value as WorkspaceTemplate)}
              className="mt-1 w-full rounded-lg border border-zinc-700 bg-zinc-950 px-3 py-2 text-zinc-100"
            >
              {CREATE_WORKSPACE_TEMPLATES.map((t) => (
                <option key={t.value} value={t.value}>
                  {t.label}
                </option>
              ))}
            </select>
          </label>
          <button
            type="submit"
            className="mt-4 cursor-pointer rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
          >
            Create
          </button>
        </form>
      )}

      {loading ? (
        <p className="text-zinc-500">Loading…</p>
      ) : filteredWorkspaces.length === 0 ? (
        <p className="text-zinc-500">
          {searchQuery.trim()
            ? "No workspaces match your search."
            : showArchived
              ? "No archived workspaces."
              : "No workspaces yet."}
        </p>
      ) : (
        <ul className="space-y-3">
          {filteredWorkspaces.map((ws) => {
            const isArchived = Boolean(ws.archived_at);
            return (
              <li key={ws.id} className="flex items-stretch gap-2">
                <Link
                  to={`/workspace/${ws.id}`}
                  className={`block min-w-0 flex-1 cursor-pointer rounded-xl border px-5 py-4 transition hover:bg-zinc-900/70 ${
                    isArchived
                      ? "border-zinc-800/60 bg-zinc-900/20 opacity-70"
                      : "border-zinc-800 bg-zinc-900/40 hover:border-zinc-600"
                  }`}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <h3 className="font-medium text-zinc-100">{ws.name}</h3>
                      <p className="mt-1 text-sm text-zinc-400 line-clamp-2">
                        {ws.goal}
                      </p>
                    </div>
                    <span className="shrink-0 whitespace-nowrap rounded-full bg-zinc-800 px-2.5 py-0.5 text-xs text-zinc-400">
                      {templateLabel(ws.template)}
                    </span>
                  </div>
                </Link>
                <button
                  type="button"
                  onClick={() => setArchiveConfirm(ws)}
                  className="shrink-0 cursor-pointer self-center rounded-lg border border-zinc-700 px-3 py-2 text-xs text-zinc-400 hover:border-zinc-500 hover:text-zinc-200"
                >
                  {isArchived ? "Restore" : "Archive"}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
