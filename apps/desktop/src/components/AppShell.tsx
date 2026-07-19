import { useCallback, useEffect, useState } from "react";
import { Link, Outlet, useLocation, useNavigate, useParams } from "react-router-dom";
import { openUrl } from "@tauri-apps/plugin-opener";
import { MessageSquarePlus, Plus } from "lucide-react";
import { fetchWorkspaceHygiene, initDatabase, listWorkspaces } from "../api";
import type { AppShellOutletContext } from "../shellContext";
import type { Workspace, WorkspaceHygieneReport } from "../types";

const DESIGN_PARTNER_ISSUE_URL =
  "https://github.com/sstanley-yelnatss/ContextLayer/issues/new?template=design-partner-feedback.yml";

function truncateName(name: string, max = 22): string {
  if (name.length <= max) return name;
  return `${name.slice(0, max - 1)}…`;
}

export default function AppShell() {
  const location = useLocation();
  const navigate = useNavigate();
  const { workspaceId: routeWorkspaceId } = useParams<{ workspaceId: string }>();
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [hygiene, setHygiene] = useState<WorkspaceHygieneReport | null>(null);

  const activeWorkspaceId =
    routeWorkspaceId ??
    (location.pathname.startsWith("/workspace/")
      ? location.pathname.split("/")[2]
      : undefined);

  const refreshWorkspaces = useCallback(async () => {
    await initDatabase();
    setWorkspaces(await listWorkspaces(false));
  }, []);

  useEffect(() => {
    void refreshWorkspaces();
  }, [refreshWorkspaces, location.pathname]);

  useEffect(() => {
    if (!activeWorkspaceId) {
      setHygiene(null);
      return;
    }
    let cancelled = false;
    fetchWorkspaceHygiene(activeWorkspaceId)
      .then((report) => {
        if (!cancelled) setHygiene(report);
      })
      .catch(() => {
        if (!cancelled) setHygiene(null);
      });
    return () => {
      cancelled = true;
    };
  }, [activeWorkspaceId]);

  const openNewWorkspace = useCallback(() => {
    navigate("/", { state: { openCreate: true } });
  }, [navigate]);

  const outletContext: AppShellOutletContext = {
    refreshWorkspaces,
    openNewWorkspace,
  };

  const openLoops = hygiene?.summary.still_open ?? 0;
  const reasoningDebt = hygiene?.summary.reasoning_debt ?? 0;
  const onHelp = location.pathname === "/help";

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <aside
        className="flex w-52 shrink-0 flex-col overflow-hidden border-r border-sidebar-border bg-sidebar"
      >
        <div className="flex items-center gap-2.5 border-b border-sidebar-border px-4 py-3.5">
          <div
            className="flex h-[18px] w-[18px] shrink-0 items-center justify-center rounded-[3px]"
            style={{ background: "rgba(34,211,238,0.15)" }}
          >
            <div className="h-[9px] w-[9px] rounded-[2px] bg-accent" />
          </div>
          <span className="text-[13px] font-semibold tracking-tight text-foreground">
            ContextLayer
          </span>
        </div>

        <div className="flex-1 overflow-y-auto px-2 pb-2 pt-4">
          <p className="font-mono-ui mb-1.5 px-2 text-[12px] font-medium uppercase tracking-widest text-muted-foreground">
            Workspaces
          </p>
          <div className="space-y-0.5">
            {workspaces.map((ws) => {
              const active = ws.id === activeWorkspaceId;
              return (
                <Link
                  key={ws.id}
                  to={`/workspace/${ws.id}`}
                  title={ws.name}
                  className={`font-mono-ui block w-full truncate rounded-[3px] px-2 py-1.5 text-left text-[12.5px] transition-colors ${
                    active
                      ? "bg-[rgba(34,211,238,0.08)] text-accent"
                      : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  {truncateName(ws.name)}
                </Link>
              );
            })}
            {workspaces.length === 0 && (
              <p className="px-2 py-1 text-[11px] text-muted-foreground">No workspaces yet</p>
            )}
          </div>

          {activeWorkspaceId && hygiene && (openLoops > 0 || reasoningDebt > 0) && (
            <>
              <p className="font-mono-ui mb-1.5 mt-6 px-2 text-[12px] font-medium uppercase tracking-widest text-muted-foreground">
                Hygiene
              </p>
              <div className="space-y-0.5 px-1">
                {openLoops > 0 && (
                  <div className="flex items-center gap-1.5 py-0.5">
                    <span
                      className="h-1.5 w-1.5 shrink-0 rounded-full"
                      style={{ background: "var(--belief-open)" }}
                    />
                    <span
                      className="font-mono-ui text-[11px]"
                      style={{ color: "var(--belief-open)" }}
                    >
                      {openLoops} open loop{openLoops !== 1 ? "s" : ""}
                    </span>
                  </div>
                )}
                {reasoningDebt > 0 && (
                  <div className="flex items-center gap-1.5 py-0.5">
                    <span
                      className="h-1.5 w-1.5 shrink-0 rounded-full"
                      style={{ background: "var(--hygiene-warn)" }}
                    />
                    <span
                      className="font-mono-ui text-[11px]"
                      style={{ color: "var(--hygiene-warn)" }}
                    >
                      {reasoningDebt} reasoning debt
                    </span>
                  </div>
                )}
              </div>
            </>
          )}
        </div>

        <div className="space-y-0.5 border-t border-sidebar-border px-3 py-3">
          <Link
            to="/help"
            className={`block w-full rounded-[3px] px-2 py-1.5 text-[13px] transition-colors ${
              onHelp
                ? "bg-[rgba(255,255,255,0.06)] text-foreground"
                : "text-muted-foreground hover:text-foreground"
            }`}
          >
            Help
          </Link>
          <button
            type="button"
            onClick={() => {
              void openUrl(DESIGN_PARTNER_ISSUE_URL);
            }}
            className="flex w-full items-center gap-2 rounded-[3px] px-2 py-1.5 text-[13px] text-muted-foreground transition-colors hover:text-foreground"
          >
            <MessageSquarePlus size={13} />
            <span>Report feedback</span>
          </button>
          <button
            type="button"
            onClick={openNewWorkspace}
            className="flex w-full items-center gap-2 rounded-[3px] px-2 py-1.5 text-[13px] text-muted-foreground transition-colors hover:text-foreground"
          >
            <Plus size={13} />
            <span>New workspace</span>
          </button>
        </div>
      </aside>

      <main className="min-w-0 flex-1 overflow-hidden">
        <Outlet context={outletContext} />
      </main>
    </div>
  );
}
