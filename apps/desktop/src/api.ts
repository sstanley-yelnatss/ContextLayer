import { invoke } from "@tauri-apps/api/core";
import type { BlockEntry, Workspace, WorkspaceHygieneReport } from "./types";

export async function initDatabase(): Promise<string> {
  return invoke<string>("init_database");
}

export async function listWorkspaces(): Promise<Workspace[]> {
  return invoke<Workspace[]>("list_workspaces");
}

export async function createWorkspace(
  name: string,
  goal: string,
  template: string,
): Promise<Workspace> {
  return invoke("create_workspace", { name, goal, template });
}

export async function fetchWorkspaceHygiene(
  workspaceId: string,
): Promise<WorkspaceHygieneReport> {
  return invoke("fetch_workspace_hygiene", { workspaceId });
}

export async function fetchBlocks(
  workspaceId: string,
  ascending: boolean,
): Promise<BlockEntry[]> {
  return invoke("fetch_blocks", { workspaceId, ascending });
}

export async function saveBlock(args: {
  workspaceId: string;
  blockId?: string;
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
