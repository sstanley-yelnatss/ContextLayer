import { useCallback, useEffect, useState } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Link } from "react-router-dom";
import { getBundledToolPaths } from "../api";
import { useToast } from "../components/Toast";

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="mb-10">
      <h2 className="mb-3 text-lg font-medium text-zinc-200">{title}</h2>
      <div className="space-y-3 text-sm leading-relaxed text-zinc-400">{children}</div>
    </section>
  );
}

function Cmd({ children }: { children: string }) {
  return (
    <code className="rounded bg-zinc-800/80 px-1.5 py-0.5 font-mono text-xs text-zinc-200">
      {children}
    </code>
  );
}

function Pre({ children }: { children: string }) {
  return (
    <pre className="overflow-x-auto rounded-lg border border-zinc-800 bg-zinc-950 p-3 font-mono text-xs leading-relaxed text-zinc-300">
      {children}
    </pre>
  );
}

export default function HelpPage() {
  const { showToast } = useToast();
  const [tools, setTools] = useState<Awaited<ReturnType<typeof getBundledToolPaths>> | null>(
    null,
  );

  useEffect(() => {
    getBundledToolPaths()
      .then(setTools)
      .catch(() => setTools(null));
  }, []);

  const mcpSnippet = tools?.mcp_json
    ? JSON.stringify(tools.mcp_json, null, 2)
    : null;

  const copyMcpJson = useCallback(async () => {
    if (!mcpSnippet) {
      showToast({ message: "MCP path not found — reinstall or run from a release build.", kind: "error" });
      return;
    }
    try {
      await writeText(mcpSnippet);
      showToast("MCP config copied — paste into Cursor Settings → MCP");
    } catch (e) {
      showToast({ message: String(e), kind: "error" });
    }
  }, [mcpSnippet, showToast]);

  return (
    <div className="mx-auto max-w-3xl px-6 py-10">
      <div className="mb-8">
        <Link
          to="/"
          className="group inline-flex cursor-pointer items-center gap-2 text-sm font-medium text-zinc-500 transition hover:text-zinc-300"
        >
          <span className="text-xl leading-none" aria-hidden>
            ←
          </span>
          Workspaces
        </Link>
      </div>

      <header className="mb-10">
        <h1 className="text-3xl font-semibold tracking-tight text-zinc-50">Help</h1>
        <p className="mt-2 max-w-2xl text-base text-zinc-400">
          Day-to-day commands while you work. The Windows installer ships the desktop app plus CLI
          tools in the same folder. Repo docs have the full reference.
        </p>
      </header>

      <Section title="Bundled tools (installer)">
        <p>
          After install, these executables sit next to <strong className="text-zinc-300">ContextLayer.exe</strong>.
          You do not need Rust or a separate build for normal use.
        </p>
        {tools?.install_dir ? (
          <Pre>{`Install folder:\n${tools.install_dir}\n\ncontextlayer-recorder.exe\ncontextlayer-mcp.exe\ncontextlayer-trace.exe`}</Pre>
        ) : (
          <p className="text-zinc-500">Loading install paths…</p>
        )}
        <ul className="list-inside list-disc space-y-1.5">
          <li>
            <strong className="text-zinc-300">Recorder</strong> — optional terminal commands (
            <Cmd>bind-repo</Cmd>, <Cmd>list-workspaces</Cmd>, etc.). Live ingest runs inside the app
            when you click Start capture.
          </li>
          <li>
            <strong className="text-zinc-300">MCP server</strong> — wire into Cursor (snippet below).
          </li>
          <li>
            <strong className="text-zinc-300">Trace CLI</strong> — PR trace CI in repos that use{" "}
            <Cmd>.contextlayer/rules.yml</Cmd>.
          </li>
        </ul>
      </Section>

      <Section title="Cursor MCP setup">
        <p>
          Copy the snippet, open Cursor → Settings → MCP, and paste it into your config (or merge the{" "}
          <Cmd>contextlayer</Cmd> block into an existing <Cmd>mcpServers</Cmd> object).
        </p>
        {mcpSnippet ? (
          <>
            <Pre>{mcpSnippet}</Pre>
            <button
              type="button"
              onClick={() => void copyMcpJson()}
              className="cursor-pointer rounded-lg border border-violet-700/60 bg-violet-950/40 px-4 py-2 text-sm font-medium text-violet-200 transition hover:border-violet-600 hover:bg-violet-900/40"
            >
              Copy MCP config
            </button>
          </>
        ) : (
          <p className="text-zinc-500">
            MCP path unavailable in dev until sidecars are built (
            <Cmd>npm run desktop:sidecars</Cmd> from repo root).
          </p>
        )}
        <p className="text-zinc-500">
          MCP opens and closes the capture gate like the toolbar.{" "}
          <strong className="text-zinc-300">Ingest</strong> needs either Start capture in this app (recommended)
          or <Cmd>contextlayer-recorder watch</Cmd> in a terminal while the gate is open.
        </p>
      </Section>

      <Section title="Desktop workflow">
        <ul className="list-inside list-disc space-y-1.5">
          <li>Create a workspace, log blocks on the timeline (assumption → action → evidence → conclusion).</li>
          <li>Use the hygiene panel to catch orphans, stale threads, and dead ends.</li>
          <li>
            <strong className="text-zinc-300">PR export:</strong> turn on PR export mode, select blocks,
            optionally include session trace (checkpoints and/or raw log), copy, paste into GitHub.
          </li>
          <li>
            <strong className="text-zinc-300">Capture (optional):</strong> click{" "}
            <strong className="text-zinc-300">Start capture</strong> in the toolbar. The app polls Cursor
            transcripts automatically while capture is on (no separate terminal). Open your repo in Cursor
            from a git folder so auto-bind can map the project. Stop capture when done.
          </li>
        </ul>
      </Section>

      <Section title="PR export — session trace options">
        <p>
          In <strong className="text-zinc-300">PR export</strong> mode, two checkboxes control an optional
          appendix at the bottom of the export. They only matter if you ran capture and have session data;
          otherwise the export is your selected blocks only.
        </p>
        <ul className="space-y-3">
          <li>
            <strong className="text-zinc-300">Session trace: checkpoints</strong> (on by default) — decision
            moments from the <strong className="text-zinc-300">Checkpoint</strong> button.
          </li>
          <li>
            <strong className="text-zinc-300">Session trace: raw log</strong> (off by default) — capped Cursor
            chat since capture started.
          </li>
        </ul>
      </Section>

      <Section title="When you need the recorder CLI">
        <p>
          <strong className="text-zinc-300">Normal use:</strong> you do not need the CLI.{" "}
          <strong className="text-zinc-300">Start capture</strong> in the app opens the session and
          ingests Cursor chat while the app is open.
        </p>
        <p className="text-zinc-500">Use the bundled CLI when:</p>
        <ul className="list-inside list-disc space-y-1.5">
          <li>
            <strong className="text-zinc-300">MCP-only, app closed</strong> —{" "}
            <Cmd>start_capture</Cmd> in Cursor opens the gate but does not poll transcripts. Run{" "}
            <Cmd>contextlayer-recorder watch</Cmd> in a terminal, or open the app and Start capture.
          </li>
          <li>
            <strong className="text-zinc-300">Custom binding</strong> — auto-bind only uses git root.
            Use <Cmd>bind-repo</Cmd> for a specific path, or{" "}
            <Cmd>--cursor-project</Cmd> on <Cmd>start</Cmd> to limit which Cursor project ingests.
          </li>
          <li>
            <strong className="text-zinc-300">Import old chat</strong> —{" "}
            <Cmd>import --file …</Cmd> backfills a transcript file (onboarding; not live capture).
          </li>
          <li>
            <strong className="text-zinc-300">Scripts / debugging</strong> —{" "}
            <Cmd>status</Cmd>, <Cmd>list-bindings</Cmd>, <Cmd>once</Cmd> (single poll + stats), branch/merge
            from terminal.
          </li>
          <li>
            <strong className="text-zinc-300">Terminal-only workflow</strong> — control{" "}
            <Cmd>start</Cmd> / <Cmd>stop</Cmd> / <Cmd>watch</Cmd> without opening the GUI (still need{" "}
            <Cmd>watch</Cmd> for ingest unless the desktop app is also running capture).
          </li>
        </ul>
        <p className="text-zinc-500">
          Duplicating <Cmd>start</Cmd> + <Cmd>watch</Cmd> in a terminal while the app already has capture
          on is redundant but harmless (same log; one poll loop is enough).
        </p>
      </Section>

      <Section title="Recorder CLI (reference)">
        <p>
          Same binary as in the install folder. Examples below; skip entirely if you use Start capture in
          the app.
        </p>
        <Pre>{`contextlayer-recorder list-workspaces
contextlayer-recorder bind-repo --path C:\\path\\to\\repo --workspace "My workspace"
contextlayer-recorder start --workspace "My workspace"
contextlayer-recorder watch
contextlayer-recorder stop --workspace "My workspace"`}</Pre>
      </Section>

      <Section title="MCP — read first">
        <ul className="space-y-2">
          <li>
            <Cmd>list_workspaces</Cmd> — pick a workspace
          </li>
          <li>
            <Cmd>get_workspace_index</Cmd> — titles, belief, flags (no bodies)
          </li>
          <li>
            <Cmd>get_block</Cmd> — one full block by id or title
          </li>
          <li>
            <Cmd>get_workspace_hygiene</Cmd> — before suggesting next tests
          </li>
        </ul>
      </Section>

      <Section title="MCP — write & export">
        <ul className="space-y-2">
          <li>
            <Cmd>save_block</Cmd> — primary write path; partial updates OK; target by title
          </li>
          <li>
            <Cmd>export_blocks</Cmd> — PR markdown for selected blocks (same as desktop export)
          </li>
          <li>
            <Cmd>compile_agent_context</Cmd> — full agent handoff packet
          </li>
          <li>
            <Cmd>start_capture</Cmd> / <Cmd>stop_capture</Cmd> / <Cmd>capture_status</Cmd>
          </li>
          <li>
            <Cmd>commit_checkpoint</Cmd> — decision moment; slices session log
          </li>
          <li>
            <Cmd>branch_capture_session</Cmd> / <Cmd>merge_capture_branch</Cmd> — fork a tangent
            thread within an active capture (same ingested log; branching still works)
          </li>
        </ul>
      </Section>

      <Section title="Where data lives">
        <ul className="space-y-1.5 font-mono text-xs text-zinc-300">
          <li>%USERPROFILE%\.contextlayer\graph.db</li>
          <li>%USERPROFILE%\.contextlayer\capture\&lt;workspace_id&gt;\log.jsonl</li>
          <li>%USERPROFILE%\.contextlayer\capture\&lt;workspace_id&gt;\commits.jsonl</li>
        </ul>
      </Section>

      <Section title="Quick fixes">
        <ul className="list-inside list-disc space-y-1.5">
          <li>
            Capture on but log empty → work in Cursor on a git repo; Start capture auto-binds git root
            when possible. Otherwise run <Cmd>bind-repo</Cmd> once.
          </li>
          <li>Wrong workspace in MCP → use exact workspace title from this app.</li>
          <li>MCP stale after update → disable MCP in Cursor, re-copy config from Help, re-enable.</li>
        </ul>
      </Section>
    </div>
  );
}
