import { invoke } from "@tauri-apps/api/core";
import type { BlockEntry, Workspace, WorkspaceHygieneReport } from "./types";

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
  });
}

export async function startCapture(workspaceId: string, label?: string) {
  return invoke("start_capture_cmd", { workspaceId, label: label ?? null });
}

export async function stopCapture(workspaceId: string) {
  return invoke("stop_capture_cmd", { workspaceId });
}

export async function captureStatus(workspaceId: string): Promise<{
  active_session: { id: string; workspace_id: string } | null;
  summary: { message_count: number; commit_count: number };
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
