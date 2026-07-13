import { invoke } from "@tauri-apps/api/core";
import type { BlockEntry, Workspace, WorkspaceHygieneReport } from "./types";

export type CaptureSource = "cursor" | "claude";

export type CaptureCandidate = {
  cursor_project: string;
  transcript_path: string;
  label: string;
  modified_secs_ago: number;
  source?: CaptureSource;
};

function normalizeCaptureSource(source?: string): CaptureSource | undefined {
  if (source === "cursor" || source === "claude") {
    return source;
  }
  return undefined;
}

export function normalizeCaptureCandidates(
  candidates: Array<{
    cursor_project: string;
    transcript_path: string;
    label: string;
    modified_secs_ago: number;
    source?: string;
  }>,
): CaptureCandidate[] {
  return candidates.map((c) => ({
    ...c,
    source: normalizeCaptureSource(c.source),
  }));
}

export async function getBundledToolPaths(): Promise<{
  install_dir: string | null;
  recorder: string | null;
  mcp: string | null;
  trace: string | null;
  capture_watcher_running: boolean;
  mcp_json: Record<string, unknown> | null;
}> {
  return invoke("get_bundled_tool_paths");
}

export async function initDatabase(): Promise<string> {
  return invoke<string>("init_database");
}

export async function listWorkspaces(includeArchived = false): Promise<Workspace[]> {
  return invoke<Workspace[]>("list_workspaces", { includeArchived });
}

export async function setWorkspaceArchived(id: string, archived: boolean): Promise<Workspace> {
  return invoke("set_workspace_archived", { id, archived });
}

export async function createWorkspace(
  name: string,
  goal: string,
  template: string,
): Promise<Workspace> {
  return invoke("create_workspace", { name, goal, template });
}

export async function updateWorkspace(args: {
  id: string;
  name: string;
  goal: string;
  template: string;
}): Promise<Workspace> {
  return invoke("update_workspace", {
    id: args.id,
    name: args.name,
    goal: args.goal,
    template: args.template,
  });
}

export async function fetchWorkspaceHygiene(
  workspaceId: string,
): Promise<WorkspaceHygieneReport> {
  return invoke("fetch_workspace_hygiene", { workspaceId });
}

export async function fetchBlocks(
  workspaceId: string,
  ascending: boolean,
  fresh = false,
): Promise<BlockEntry[]> {
  return invoke("fetch_blocks", { workspaceId, ascending, fresh });
}

export async function saveBlock(args: {
  workspaceId: string;
  blockId?: string;
  blockTitle?: string;
  title?: string;
  hypothesisText?: string;
  actionText?: string;
  evidenceText?: string;
  evidenceSource?: string;
  conclusionText?: string;
  conclusionOutcome?: string;
  conclusionTag?: string;
  confidenceLevel?: string;
  beliefState?: string;
  systemTag?: string;
  userTag?: string;
  linkToBlockIds?: string[];
}): Promise<BlockEntry> {
  return invoke("save_block", {
    workspaceId: args.workspaceId,
    blockId: args.blockId ?? null,
    blockTitle: args.blockTitle ?? null,
    title: args.title ?? null,
    hypothesisText: args.hypothesisText ?? null,
    actionText: args.actionText ?? null,
    evidenceText: args.evidenceText ?? null,
    evidenceSource: args.evidenceSource ?? null,
    conclusionText: args.conclusionText ?? null,
    conclusionOutcome: args.conclusionOutcome ?? null,
    conclusionTag: args.conclusionTag ?? null,
    confidenceLevel: args.confidenceLevel ?? null,
    beliefState: args.beliefState ?? null,
    systemTag: args.systemTag ?? null,
    userTag: args.userTag ?? null,
    linkToBlockIds: args.linkToBlockIds ?? [],
  });
}

export async function softDeleteBlock(blockId: string) {
  return invoke("soft_delete_block", { blockId });
}

export async function listBlocksForPicker(
  workspaceId: string,
): Promise<[string, string][]> {
  return invoke("list_blocks_for_picker", { workspaceId });
}

export async function addBlockLink(
  workspaceId: string,
  fromBlockId: string,
  toBlockId: string,
) {
  return invoke("add_block_link", { workspaceId, fromBlockId, toBlockId });
}

export async function exportWorkspaceSummary(
  workspaceId: string,
): Promise<string> {
  return invoke("export_workspace_summary", { workspaceId });
}

export const TRACE_LOG_SLICE_OPTIONS = [
  { value: "past_25", label: "Past 25" },
  { value: "past_50", label: "Past 50" },
  { value: "past_75", label: "Past 75" },
  { value: "past_100", label: "Past 100" },
  { value: "first_25", label: "First 25" },
  { value: "first_50", label: "First 50" },
  { value: "first_75", label: "First 75" },
  { value: "first_100", label: "First 100" },
  {
    value: "since_last_capture_start",
    label: "Since last capture start",
    requiresCaptureBoundary: true,
  },
] as const;

export type TraceLogSlice = (typeof TRACE_LOG_SLICE_OPTIONS)[number]["value"];

export async function exportPrReasoning(
  workspaceId: string,
  blockIds: string[],
  options?: {
    branch?: string;
    prNumber?: string;
    gitSha?: string;
    /** Legacy: false omits entire trace section */
    includeTrace?: boolean;
    includeTraceCheckpoints?: boolean;
    includeTraceLog?: boolean;
    includeTraceBranchLogs?: boolean;
    /** past_25|50|75|100, first_25|50|75|100, since_last_capture_start */
    traceLogSlice?: string;
  },
): Promise<string> {
  return invoke("export_pr_reasoning", {
    workspaceId,
    blockIds,
    branch: options?.branch ?? null,
    prNumber: options?.prNumber ?? null,
    gitSha: options?.gitSha ?? null,
    includeTrace: options?.includeTrace ?? null,
    includeTraceCheckpoints: options?.includeTraceCheckpoints ?? true,
    includeTraceLog: options?.includeTraceLog ?? false,
    includeTraceBranchLogs: options?.includeTraceBranchLogs ?? false,
    traceLogSlice: options?.traceLogSlice ?? "past_50",
  });
}

export async function startCapture(
  workspaceId: string,
  options?: {
    label?: string;
    cursorProject?: string;
    transcriptPath?: string;
    rememberScope?: boolean;
  },
) {
  return invoke<{
    status: "started" | "needs_picker" | "no_candidates";
    session?: { id: string; cursor_project?: string; transcript_path?: string };
    scope_label?: string;
    baselined_transcript_files?: number;
    candidates?: CaptureCandidate[];
    hint?: string;
    capture_watcher_running?: boolean;
  }>("start_capture_cmd", {
    workspaceId,
    label: options?.label ?? null,
    cursorProject: options?.cursorProject ?? null,
    transcriptPath: options?.transcriptPath ?? null,
    rememberScope: options?.rememberScope ?? false,
  });
}

export async function listCaptureCandidates() {
  const result = await invoke<{
    candidates: Array<{
      cursor_project: string;
      transcript_path: string;
      label: string;
      modified_secs_ago: number;
      source?: string;
    }>;
  }>("list_capture_candidates_cmd");
  return { candidates: normalizeCaptureCandidates(result.candidates) };
}

export async function stopCapture(workspaceId: string) {
  return invoke("stop_capture_cmd", { workspaceId });
}

export async function captureStatus(workspaceId: string): Promise<{
  active_session: {
    id: string;
    workspace_id: string;
    cursor_project?: string;
    transcript_path?: string;
  } | null;
  summary: { message_count: number; commit_count: number };
  session_message_count?: number;
  scope_label?: string | null;
  capture_log_boundary_available?: boolean;
}> {
  return invoke("capture_status_cmd", { workspaceId });
}

export async function commitTraceCheckpoint(args: {
  workspaceId: string;
  intent: string;
  note: string;
  rejectedPaths?: string[];
  gitSha?: string;
  blockIds?: string[];
}): Promise<unknown> {
  return invoke("commit_trace_checkpoint", {
    workspaceId: args.workspaceId,
    intent: args.intent,
    note: args.note,
    rejectedPaths: args.rejectedPaths ?? [],
    gitSha: args.gitSha ?? null,
    blockIds: args.blockIds ?? [],
  });
}
