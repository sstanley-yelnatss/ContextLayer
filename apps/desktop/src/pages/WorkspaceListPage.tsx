import { useEffect, useMemo, useState } from "react";
import { Link, useLocation, useOutletContext } from "react-router-dom";
import ConfirmDialog from "../components/ConfirmDialog";
import { useToast } from "../components/Toast";
import type { AppShellOutletContext } from "../shellContext";
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
  const location = useLocation();
  const { refreshWorkspaces } = useOutletContext<AppShellOutletContext>();
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(
    Boolean((location.state as { openCreate?: boolean } | null)?.openCreate),
  );
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
      await refreshWorkspaces();
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
      await refreshWorkspaces();
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

  useEffect(() => {
    if ((location.state as { openCreate?: boolean } | null)?.openCreate) {
      setShowForm(true);
    }
  }, [location.state]);

  return (
    <div className="h-full overflow-y-auto">
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
        <h1 className="text-xl font-semibold tracking-tight text-foreground">Workspaces</h1>
        <p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted-foreground">
          Local investigations for serious questions. Each workspace tracks what you
          assume, try, observe, and conclude — then exports a reasoning receipt for PR
          review.
        </p>
      </header>

      <div className="mb-4 flex flex-wrap items-center gap-3">
        <input
          type="search"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search workspaces…"
          className="min-w-[12rem] flex-1 rounded-[3px] border border-border bg-input-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground"
        />
        <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
          <input
            type="checkbox"
            checked={showArchived}
            onChange={(e) => setShowArchived(e.target.checked)}
            className="rounded border-border accent-[var(--accent)]"
          />
          Show archived
        </label>
      </div>

      <div className="mb-6 flex items-center justify-between">
        <h2 className="text-sm font-medium text-muted-foreground">Your list</h2>
        <button
          type="button"
          onClick={() => setShowForm(!showForm)}
          className="cursor-pointer rounded-[3px] border border-border bg-[rgba(255,255,255,0.06)] px-4 py-2 text-sm font-medium text-foreground hover:bg-[rgba(255,255,255,0.09)]"
        >
          {showForm ? "Cancel" : "New workspace"}
        </button>
      </div>

      {showForm && (
        <form
          onSubmit={handleCreate}
          className="cl-surface-card mb-8 p-5"
        >
          <label className="cl-label">
            Name
            <input
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="cl-input"
            />
          </label>
          <label className="cl-label mt-4">
            Goal (required)
            <textarea
              required
              value={goal}
              onChange={(e) => setGoal(e.target.value)}
              rows={3}
              placeholder={goalPlaceholder}
              className="cl-input"
            />
          </label>
          <label className="cl-label mt-4">
            Template
            <select
              value={template}
              onChange={(e) => setTemplate(e.target.value as WorkspaceTemplate)}
              className="select-filter mt-1 w-full py-2"
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
            className="mt-4 cursor-pointer rounded-[3px] border border-border bg-[rgba(255,255,255,0.08)] px-4 py-2 text-sm font-medium text-foreground hover:bg-[rgba(255,255,255,0.12)]"
          >
            Create
          </button>
        </form>
      )}

      {loading ? (
        <p className="text-muted-foreground">Loading…</p>
      ) : filteredWorkspaces.length === 0 ? (
        <p className="text-muted-foreground">
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
                  className={`cl-surface-card block min-w-0 flex-1 px-5 py-4 transition-colors hover:bg-[#161619] ${
                    isArchived ? "opacity-70" : ""
                  }`}
                >
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <h3 className="font-mono-ui text-[13px] font-medium text-foreground">
                        {ws.name}
                      </h3>
                      <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">
                        {ws.goal}
                      </p>
                    </div>
                    <span className="font-mono-ui shrink-0 whitespace-nowrap rounded-[3px] border border-border bg-[rgba(255,255,255,0.03)] px-2 py-0.5 text-[10px] text-muted-foreground">
                      {templateLabel(ws.template)}
                    </span>
                  </div>
                </Link>
                <button
                  type="button"
                  onClick={() => setArchiveConfirm(ws)}
                  className="cl-btn-ghost shrink-0 self-center px-3 py-2 text-xs"
                >
                  {isArchived ? "Restore" : "Archive"}
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
    </div>
  );
}
