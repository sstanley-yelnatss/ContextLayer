import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { createWorkspace, initDatabase, listWorkspaces } from "../api";
import { TEMPLATES, type Workspace, type WorkspaceTemplate } from "../types";

export default function WorkspaceListPage() {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [goal, setGoal] = useState("");
  const [template, setTemplate] = useState<WorkspaceTemplate>("blank");

  async function load() {
    setLoading(true);
    setError("");
    try {
      await initDatabase();
      setWorkspaces(await listWorkspaces());
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    load();
  }, []);

  async function handleCreate(e: React.FormEvent) {
    e.preventDefault();
    try {
      await createWorkspace(name, goal, template);
      setName("");
      setGoal("");
      setTemplate("blank");
      setShowForm(false);
      setWorkspaces(await listWorkspaces());
    } catch (err) {
      setError(String(err));
    }
  }

  return (
    <div className="mx-auto max-w-3xl px-6 py-10">
      <header className="mb-10">
        <h1 className="text-3xl font-semibold tracking-tight text-zinc-50">
          ContextLayer
        </h1>
        <p className="mt-2 text-zinc-400">
          Reasoning graph — workspaces, hypotheses, tests, evidence, conclusions
        </p>
      </header>

      {error && (
        <p className="mb-4 rounded-lg border border-red-900/50 bg-red-950/40 px-4 py-3 text-sm text-red-300">
          {error}
        </p>
      )}

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
              {TEMPLATES.map((t) => (
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
      ) : workspaces.length === 0 ? (
        <p className="text-zinc-500">No workspaces yet.</p>
      ) : (
        <ul className="space-y-3">
          {workspaces.map((ws) => (
            <li key={ws.id}>
              <Link
                to={`/workspace/${ws.id}`}
                className="block cursor-pointer rounded-xl border border-zinc-800 bg-zinc-900/40 px-5 py-4 transition hover:border-zinc-600 hover:bg-zinc-900/70"
              >
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <h3 className="font-medium text-zinc-100">{ws.name}</h3>
                    <p className="mt-1 text-sm text-zinc-400 line-clamp-2">
                      {ws.goal}
                    </p>
                  </div>
                  <span className="shrink-0 rounded-full bg-zinc-800 px-2.5 py-0.5 text-xs text-zinc-400">
                    {ws.template.replace("_", " ")}
                  </span>
                </div>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
