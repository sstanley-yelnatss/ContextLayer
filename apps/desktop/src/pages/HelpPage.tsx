import type { ReactNode } from "react";
import { Link } from "react-router-dom";

function Section({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
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
          Day-to-day commands while you work. Full install, MCP wiring, and troubleshooting
          live in the repo{" "}
          <a
            href="https://github.com/sstanley-yelnatss/ContextLayer/blob/develop/docs/COMMANDS-CHEATSHEET.md"
            className="text-violet-400 underline underline-offset-2 hover:text-violet-300"
            target="_blank"
            rel="noreferrer"
          >
            docs folder
          </a>
          .
        </p>
      </header>

      <Section title="Desktop workflow">
        <ul className="list-inside list-disc space-y-1.5">
          <li>Create a workspace, log blocks on the timeline (assumption → action → evidence → conclusion).</li>
          <li>Use the hygiene panel to catch orphans, stale threads, and dead ends.</li>
          <li>
            <strong className="text-zinc-300">PR export:</strong> turn on PR export mode, select blocks,
            optionally include session trace (checkpoints and/or raw log), copy, paste into GitHub.
          </li>
          <li>
            <strong className="text-zinc-300">Capture (optional):</strong> adds raw Cursor chat to a session
            log for PR trace. Blocks on the timeline work without it. When you want capture: turn it on with{" "}
            <strong className="text-zinc-300">Start capture</strong> in the toolbar (same as{" "}
            <Cmd>start</Cmd> in the CLI), leave <Cmd>contextlayer-recorder watch</Cmd> running in a
            terminal while you chat, then stop capture when done.
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
            <strong className="text-zinc-300">Session trace: checkpoints</strong> (on by default) — includes
            decision moments you committed with the <strong className="text-zinc-300">Checkpoint</strong>{" "}
            button: intent, note, rejected paths, and which log lines that decision covers. This is the
            structured “what we decided and when” trail. Use this for most PRs; reviewers get the arc without
            wading through chat.
          </li>
          <li>
            <strong className="text-zinc-300">Session trace: raw log</strong> (off by default) — includes a
            capped slice of ingested Cursor chat (recent messages since capture started, truncated per
            message). Noisier, but useful when someone needs to see what was actually said in the session,
            not just your blocks and checkpoints.
          </li>
        </ul>
        <p className="text-zinc-500">Quick picks:</p>
        <ul className="list-inside list-disc space-y-1.5">
          <li>
            <strong className="text-zinc-300">Checkpoints only</strong> — default; best for normal PR review.
          </li>
          <li>
            <strong className="text-zinc-300">Both</strong> — when the reviewer may ask “what did the agent
            say?” and blocks alone are not enough.
          </li>
          <li>
            <strong className="text-zinc-300">Neither</strong> — blocks-only reasoning receipt; no capture
            appendix (fine if you never turned capture on).
          </li>
          <li>
            <strong className="text-zinc-300">Raw log only</strong> — rare; checkpoints are usually more
            useful than chat alone.
          </li>
        </ul>
      </Section>

      <Section title="Recorder CLI">
        <p>
          The recorder pulls <strong className="text-zinc-300">raw Cursor chat</strong> into a session log
          (optional PR trace). Your reasoning blocks live in the app database and do not need capture.
        </p>
        <p>
          Capture uses two steps on purpose. <strong className="text-zinc-300">Start</strong> turns capture
          on (one command or toolbar click — then you get your prompt back).{" "}
          <strong className="text-zinc-300">Watch</strong> is a separate terminal window you leave open;
          it copies new Cursor chat into the log while capture is on. You need both for automatic capture;
          skip both if you only want structured blocks.
        </p>
        <p>Build once from repo root:</p>
        <Pre>{`cargo build -p contextlayer-recorder --release`}</Pre>
        <p>Typical session (use your workspace name):</p>
        <Pre>{`contextlayer-recorder list-workspaces
contextlayer-recorder bind-repo --path C:\\path\\to\\repo --workspace "My workspace"
contextlayer-recorder start --workspace "My workspace"
contextlayer-recorder watch
contextlayer-recorder stop --workspace "My workspace"`}</Pre>
        <p className="text-zinc-500">What each command does, in order:</p>
        <ul className="space-y-2">
          <li>
            <Cmd>list-workspaces</Cmd> — print workspace names and IDs so you can copy the exact title
            for other commands.
          </li>
          <li>
            <Cmd>bind-repo</Cmd> — map a repo folder to a workspace (once per project). No recording yet.
          </li>
          <li>
            <Cmd>start</Cmd> — turn capture <strong className="text-zinc-300">on</strong> for this workspace.
            The command finishes right away and returns you to the prompt; it does not stay running. From
            that moment, only new Cursor chat counts (nothing before is pulled in). Same as{" "}
            <strong className="text-zinc-300">Start capture</strong> in the toolbar or{" "}
            <Cmd>start_capture</Cmd> in MCP. It does not read transcript files — run{" "}
            <Cmd>watch</Cmd> for that.
          </li>
          <li>
            <Cmd>watch</Cmd> — leave this running in a terminal (Ctrl+C when done). It checks Cursor
            transcript files every few seconds and copies new chat into the session log, but only while
            capture is on (<Cmd>start</Cmd> or Start capture). If capture is off, watch still runs but
            writes nothing.
          </li>
          <li>
            <Cmd>stop</Cmd> — turn capture <strong className="text-zinc-300">off</strong> for this
            workspace. Finishes right away like <Cmd>start</Cmd>. If watch is still running, it keeps
            polling but stops writing until you start again.
          </li>
        </ul>
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
            <Cmd>branch_capture_session</Cmd> / <Cmd>merge_capture_branch</Cmd> — fork chat threads
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
            Recorder ingests nothing → capture off? Run Start capture or <Cmd>start</Cmd>. Capture on but
            still empty? Run <Cmd>watch</Cmd> in a terminal and confirm <Cmd>bind-repo</Cmd> matches your
            repo.
          </li>
          <li>Wrong workspace in MCP → use exact workspace title from this app.</li>
          <li>MCP stale after rebuild → disable MCP in Cursor, rebuild, re-enable.</li>
        </ul>
      </Section>
    </div>
  );
}
