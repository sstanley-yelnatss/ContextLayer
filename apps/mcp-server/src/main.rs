//! ContextLayer minimal MCP — stdio read/write lane to ~/.contextlayer/graph.db
//! No text normalization; records user/agent wording as provided.

use std::path::PathBuf;

use contextlayer_db::{default_db_path, DbError, GraphStore, SaveBlockInput};
use contextlayer_export::compile_workspace_summary_markdown;
use contextlayer_export::compile_agent_context_markdown;
use contextlayer_export::{compile_pr_export_markdown_with_options, PrExportOptions};
use contextlayer_export::build_workspace_index;
use contextlayer_export::resolve_pr_block_ids;
use contextlayer_export::resolve_agent_block_ids;
use contextlayer_export::import_transcript;
use contextlayer_trace::{
    build_context_summary, compile_pr_trace_appendix_with_options, create_capture_branch,
    find_checkpoint, list_branches_for_workspace, merge_capture_branch as merge_branch_record,
    CaptureStore, PrTraceAppendixOptions, TraceStore,
};
use rmcp::{
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt,
};
use serde::Deserialize;
use serde_json::json;

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string()),
    )]))
}

fn err_msg(msg: String) -> McpError {
    McpError::internal_error(msg, None)
}

#[derive(Clone)]
struct ContextLayerMcp {
    db_path: PathBuf,
    tool_router: ToolRouter<Self>,
}

impl ContextLayerMcp {
    fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            tool_router: Self::tool_router(),
        }
    }

    fn with_store<F, T>(&self, f: F) -> Result<T, McpError>
    where
        F: FnOnce(&GraphStore) -> Result<T, DbError>,
    {
        let store = GraphStore::open(&self.db_path).map_err(|e| err_msg(e.to_string()))?;
        f(&store).map_err(|e| err_msg(e.to_string()))
    }

    fn with_trace<F, T>(&self, f: F) -> Result<T, McpError>
    where
        F: FnOnce(&TraceStore) -> Result<T, String>,
    {
        let store = TraceStore::default_open().map_err(err_msg)?;
        f(&store).map_err(err_msg)
    }

    fn with_capture<F, T>(&self, f: F) -> Result<T, McpError>
    where
        F: FnOnce(&contextlayer_trace::CaptureStore) -> Result<T, String>,
    {
        let store = contextlayer_trace::CaptureStore::default_open().map_err(err_msg)?;
        f(&store).map_err(err_msg)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WorkspaceIdArgs {
    /// Workspace UUID from list_workspaces
    workspace_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateWorkspaceArgs {
    name: String,
    goal: String,
    /// blank | agent_devops | security_hunt | product_research | decision_strategy
    #[serde(default = "default_template")]
    template: String,
}

fn default_template() -> String {
    "agent_devops".to_string()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TextNodeArgs {
    workspace_id: String,
    /// Recorded verbatim — no normalization
    text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CreateEvidenceArgs {
    workspace_id: String,
    text: String,
    source: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SaveConclusionArgs {
    workspace_id: String,
    text: String,
    /// confirmed | rejected | uncertain | refined
    outcome: String,
    /// none | pivot | act | ignore | defer
    #[serde(default = "default_tag")]
    tag: String,
    confidence: Option<f64>,
    hypothesis_ids: Vec<String>,
    evidence_ids: Vec<String>,
}

fn default_tag() -> String {
    "none".to_string()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AddLinkArgs {
    workspace_id: String,
    from_type: String,
    from_id: String,
    to_type: String,
    to_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SaveBlockArgs {
    workspace_id: String,
    /// Block UUID — use when known.
    block_id: Option<String>,
    /// Human title within the workspace (case-insensitive). Use instead of block_id when the user names a block.
    block_title: Option<String>,
    /// Short name for this block (unique per workspace). Set on create; optional on update.
    title: Option<String>,
    hypothesis_text: Option<String>,
    action_text: Option<String>,
    evidence_text: Option<String>,
    evidence_source: Option<String>,
    conclusion_text: Option<String>,
    conclusion_outcome: Option<String>,
    conclusion_tag: Option<String>,
    /// low | medium | high
    confidence_level: Option<String>,
    /// open | leaning_true | leaning_false | confirmed | rejected
    belief_state: Option<String>,
    /// none | needs_review | ruled_out | reportable | reasoning_debt | stale
    system_tag: Option<String>,
    user_tag: Option<String>,
    link_to_block_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeleteBlockArgs {
    workspace_id: String,
    block_id: Option<String>,
    /// Case-insensitive title match when block_id omitted.
    block_title: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UpdateWorkspaceArgs {
    workspace_id: String,
    /// Omit to keep current name.
    name: Option<String>,
    /// Omit to keep current goal.
    goal: Option<String>,
    /// blank | agent_devops | security_hunt | product_research | decision_strategy — omit to keep current.
    template: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetBlockArgs {
    workspace_id: String,
    block_id: Option<String>,
    /// Case-insensitive title match when block_id omitted.
    block_title: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RemoveLinkArgs {
    /// Node link UUID from list_links.
    link_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RemoveBlockLinkArgs {
    /// Block-to-block link UUID from list_block_links.
    link_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ExportBlocksArgs {
    workspace_id: String,
    /// Block UUIDs from list_blocks — use block_ids or block_titles (or both). Optional for compile_agent_context only.
    #[serde(default)]
    block_ids: Vec<String>,
    /// Case-insensitive block titles — alternative to block_ids.
    #[serde(default)]
    block_titles: Vec<String>,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    pr_number: Option<String>,
    #[serde(default)]
    git_sha: Option<String>,
    /// Include decision checkpoints in session trace (default true).
    #[serde(default = "default_true")]
    include_trace_checkpoints: bool,
    /// Include raw session log (first N messages since capture start; default false).
    #[serde(default)]
    include_trace_log: bool,
    /// Legacy master switch — when false, omits entire session trace section.
    #[serde(default)]
    include_trace: Option<bool>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ImportSessionArgs {
    workspace_name: String,
    /// Pasted Cursor / chat transcript
    transcript: String,
    #[serde(default)]
    goal: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AppendTraceEventArgs {
    workspace_id: String,
    /// e.g. mcp_log | tool_call | user_note
    event_type: String,
    summary: String,
    #[serde(default)]
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CommitCheckpointArgs {
    workspace_id: String,
    intent: String,
    note: String,
    #[serde(default)]
    rejected_paths: Vec<String>,
    #[serde(default)]
    git_sha: Option<String>,
    #[serde(default)]
    block_ids: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ContextLogArgs {
    workspace_id: String,
    #[serde(default)]
    from_seq: Option<u64>,
    #[serde(default = "default_log_limit")]
    limit: usize,
}

fn default_log_limit() -> usize {
    50
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ContextCommitsArgs {
    workspace_id: String,
    #[serde(default = "default_commit_limit")]
    limit: usize,
}

fn default_commit_limit() -> usize {
    10
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BranchCaptureArgs {
    /// Workspace UUID or exact name (must have active capture on main)
    workspace: String,
    /// Short label for this tangent (e.g. experiment, alt-auth) — becomes folder slug
    label: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MergeBranchArgs {
    branch_id: String,
    /// confirmed | rejected
    outcome: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CheckpointLookupArgs {
    workspace_id: String,
    /// Checkpoint UUID prefix or git_sha prefix
    id_or_sha: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BindCaptureArgs {
    workspace_id: String,
    /// Cursor sanitized project folder name under ~/.cursor/projects/
    cursor_project: String,
    /// Optional absolute repo path — also registers repo_paths binding
    repo_path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct StartCaptureArgs {
    /// Workspace UUID or exact name from list_workspaces / desktop
    workspace: String,
    /// Limit ingest to this Cursor project folder key (recommended)
    cursor_project: Option<String>,
    /// Limit ingest to one transcript file (absolute path) — e.g. current chat only
    transcript_path: Option<String>,
    /// Optional human label for this investigation
    label: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct StopCaptureArgs {
    /// Workspace UUID or exact name
    workspace: String,
}

#[tool_router]
impl ContextLayerMcp {
    #[tool(
        description = "List all ContextLayer workspaces (id, name, goal, template). Call before logging if workspace is unknown."
    )]
    fn list_workspaces(&self) -> Result<CallToolResult, McpError> {
        let list = self.with_store(|store| store.list_workspaces(false))?;
        ok_json(json!({ "workspaces": list }))
    }

    #[tool(
        description = "Read compiled reasoning state for a workspace — linked hypotheses, actions, evidence, conclusions, and draft/unlinked nodes. Call before suggesting retests."
    )]
    fn get_workspace_summary(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let markdown = self.with_store(|store| {
            compile_workspace_summary_markdown(store, &workspace_id)
                .map_err(DbError::InvalidInput)
        })?;
        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }

    #[tool(
        description = "Export selected reasoning blocks as PR-ready markdown. Pass block_ids and/or block_titles (case-insensitive). Chronological order in export."
    )]
    fn export_blocks(
        &self,
        Parameters(args): Parameters<ExportBlocksArgs>,
    ) -> Result<CallToolResult, McpError> {
        let trace_appendix = if args.include_trace == Some(false) {
            None
        } else {
            let capture = CaptureStore::default_open().map_err(err_msg)?;
            let opts = PrTraceAppendixOptions {
                include_checkpoints: args.include_trace_checkpoints,
                include_log: args.include_trace_log,
                ..PrTraceAppendixOptions::default()
            };
            compile_pr_trace_appendix_with_options(&capture, &args.workspace_id, &opts)
                .map_err(err_msg)?
        };
        let markdown = self.with_store(|store| {
            let ids = resolve_pr_block_ids(
                store,
                &args.workspace_id,
                &args.block_ids,
                &args.block_titles,
            )
            .map_err(DbError::InvalidInput)?;
            let options = PrExportOptions {
                branch: args.branch.clone(),
                pr_number: args.pr_number.clone(),
                git_sha: args.git_sha.clone(),
                trace_appendix: trace_appendix.clone(),
            };
            compile_pr_export_markdown_with_options(store, &args.workspace_id, &ids, &options)
                .map_err(DbError::InvalidInput)
        })?;
        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }

    #[tool(
        description = "Tier-1 workspace index — goal, block titles, belief, hygiene flags only. No hypothesis/action/evidence/conclusion body text. Call before get_block to save tokens."
    )]
    fn get_workspace_index(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let index = self.with_store(|store| {
            build_workspace_index(store, &workspace_id).map_err(DbError::InvalidInput)
        })?;
        ok_json(json!({ "index": index }))
    }

    #[tool(
        description = "Compile agent context packet — full block bodies with IDs and hygiene snapshot for Cursor. Optional block_ids/block_titles; omit both for entire workspace."
    )]
    fn compile_agent_context(
        &self,
        Parameters(args): Parameters<ExportBlocksArgs>,
    ) -> Result<CallToolResult, McpError> {
        let markdown = self.with_store(|store| {
            let ids = resolve_agent_block_ids(
                store,
                &args.workspace_id,
                &args.block_ids,
                &args.block_titles,
            )
            .map_err(DbError::InvalidInput)?;
            compile_agent_context_markdown(store, &args.workspace_id, &ids)
                .map_err(DbError::InvalidInput)
        })?;
        Ok(CallToolResult::success(vec![Content::text(markdown)]))
    }

    #[tool(
        description = "Import a pasted Cursor/chat transcript into a new workspace as draft blocks (needs_review). Heuristic mapper — verify in desktop after import."
    )]
    fn import_session(
        &self,
        Parameters(args): Parameters<ImportSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let result = self.with_store(|store| {
            import_transcript(
                store,
                &args.workspace_name,
                &args.goal,
                &args.transcript,
            )
            .map_err(DbError::InvalidInput)
        })?;
        let log_count = self.with_capture(|cap| {
            contextlayer_trace::import_session_log(cap, &result.workspace_id, &args.transcript)
        })?;
        ok_json(json!({ "import": result, "log_messages_imported": log_count }))
    }

    #[tool(
        description = "Append a session trace event to the local capture store (~/.contextlayer/traces). Summary redacted for secrets."
    )]
    fn append_trace_event(
        &self,
        Parameters(args): Parameters<AppendTraceEventArgs>,
    ) -> Result<CallToolResult, McpError> {
        let event = self.with_trace(|store| {
            store.append_event(
                &args.workspace_id,
                &args.event_type,
                &args.summary,
                args.payload.unwrap_or(json!({})),
            )
        })?;
        ok_json(json!({ "event": event }))
    }

    #[tool(
        description = "Commit a decision checkpoint to the raw trace — decision moments only, not every prompt. Links to workspace and optional block_ids."
    )]
    fn commit_checkpoint(
        &self,
        Parameters(args): Parameters<CommitCheckpointArgs>,
    ) -> Result<CallToolResult, McpError> {
        let cp = self.with_trace(|store| {
            store.commit_checkpoint(
                &args.workspace_id,
                &args.intent,
                &args.note,
                args.rejected_paths,
                args.git_sha,
                args.block_ids,
            )
        })?;
        ok_json(json!({ "checkpoint": cp }))
    }

    #[tool(
        description = "List checkpoint commits for a workspace from the trace store (capture v0)."
    )]
    fn list_checkpoints(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let list = self.with_trace(|store| store.list_checkpoints(&workspace_id))?;
        ok_json(json!({ "checkpoints": list }))
    }

    #[tool(
        description = "Trace store summary — event and checkpoint counts for a workspace."
    )]
    fn get_trace_summary(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let summary = self.with_trace(|store| store.summary(&workspace_id))?;
        ok_json(json!({ "trace": summary }))
    }

    #[tool(
        description = "Windowed segment of the workspace session log (user/assistant/tool turns)."
    )]
    fn get_context_log(
        &self,
        Parameters(args): Parameters<ContextLogArgs>,
    ) -> Result<CallToolResult, McpError> {
        let window = self.with_capture(|cap| {
            cap.context_log(&args.workspace_id, args.from_seq, args.limit)
        })?;
        ok_json(json!({ "context_log": window }))
    }

    #[tool(
        description = "Recent capture commits with log seq ranges and contributions."
    )]
    fn get_context_commits(
        &self,
        Parameters(args): Parameters<ContextCommitsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let window = self.with_capture(|cap| cap.context_commits(&args.workspace_id, args.limit))?;
        ok_json(json!({ "context_commits": window }))
    }

    #[tool(
        description = "Map a Cursor project folder to a ContextLayer workspace (does not start recording — use start_capture)."
    )]
    fn bind_capture_project(
        &self,
        Parameters(args): Parameters<BindCaptureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mut bindings = contextlayer_trace::load_bindings().map_err(err_msg)?;
        bindings
            .cursor_projects
            .insert(args.cursor_project.clone(), args.workspace_id.clone());
        if let Some(repo) = args.repo_path {
            bindings.repo_paths.insert(repo, args.workspace_id.clone());
        }
        contextlayer_trace::save_bindings(&bindings).map_err(err_msg)?;
        ok_json(json!({
            "bound": true,
            "cursor_project": args.cursor_project,
            "workspace_id": args.workspace_id,
        }))
    }

    #[tool(
        description = "Start opt-in live capture for a workspace. Nothing is recorded until this runs. Baselines existing transcript bytes — only new chat lines ingest. Run contextlayer-recorder watch in background (or poll via recorder once). Stop with stop_capture when done."
    )]
    fn start_capture(
        &self,
        Parameters(args): Parameters<StartCaptureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_id = self.with_store(|store| store.resolve_workspace_id(&args.workspace))?;
        let (session, baselined) = contextlayer_trace::start_capture_session(
            &workspace_id,
            args.cursor_project,
            args.transcript_path,
            args.label,
        )
        .map_err(err_msg)?;
        ok_json(json!({
            "session": session,
            "workspace_id": workspace_id,
            "workspace_ref": args.workspace,
            "baselined_transcript_files": baselined,
            "hint": "Run `contextlayer-recorder watch` or call start_capture before the investigation chat; stop_capture when finished."
        }))
    }

    #[tool(
        description = "Stop opt-in live capture for a workspace. Pollers become a no-op for that workspace until start_capture again."
    )]
    fn stop_capture(
        &self,
        Parameters(args): Parameters<StopCaptureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_id = self.with_store(|store| store.resolve_workspace_id(&args.workspace))?;
        let stopped = contextlayer_trace::stop_capture_session(&workspace_id).map_err(err_msg)?;
        ok_json(json!({ "stopped": stopped, "workspace_id": workspace_id }))
    }

    #[tool(
        description = "List active opt-in capture sessions (empty means recorder will not ingest anything)."
    )]
    fn capture_status(&self) -> Result<CallToolResult, McpError> {
        let sessions = contextlayer_trace::list_active_sessions().map_err(err_msg)?;
        ok_json(json!({ "active_sessions": sessions }))
    }

    #[tool(
        description = "GCC-style quick status: capture counts, latest checkpoint, active session, open branches."
    )]
    fn get_context_summary(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_id = self.with_store(|store| {
            store
                .resolve_workspace_id(&workspace_id)
                .map_err(|e| DbError::InvalidInput(e.to_string()))
        })?;
        let active = contextlayer_trace::list_active_sessions()
            .map_err(err_msg)?
            .iter()
            .any(|s| s.workspace_id == workspace_id);
        let open_branches = self.with_capture(|cap| {
            list_branches_for_workspace(cap, &workspace_id)
        })?
        .into_iter()
        .filter(|b| b.status == "active")
        .count() as u32;
        let active_branch = contextlayer_trace::list_active_sessions()
            .map_err(err_msg)?
            .into_iter()
            .find(|s| s.workspace_id == workspace_id)
            .map(|s| s.capture_branch);
        let summary = self.with_capture(|cap| {
            build_context_summary(
                cap,
                &workspace_id,
                active,
                open_branches,
                active_branch,
            )
        })?;
        let index = self.with_store(|store| {
            build_workspace_index(store, &workspace_id).map_err(DbError::InvalidInput)
        })?;
        ok_json(json!({ "summary": summary, "workspace_index": index }))
    }

    #[tool(
        description = "Deep dive one checkpoint by UUID prefix or git_sha (GCC context --hash)."
    )]
    fn get_context_checkpoint(
        &self,
        Parameters(args): Parameters<CheckpointLookupArgs>,
    ) -> Result<CallToolResult, McpError> {
        let detail = self.with_capture(|cap| {
            find_checkpoint(cap, &args.workspace_id, &args.id_or_sha)
        })?;
        match detail {
            Some(d) => ok_json(json!({ "checkpoint": d })),
            None => Err(McpError::invalid_params(
                "Not found",
                Some(json!({ "workspace_id": args.workspace_id, "id_or_sha": args.id_or_sha })),
            )),
        }
    }

    #[tool(
        description = "Branch capture within a workspace (GCC branch): main log frozen; new messages go to branches/{slug}/. Requires active capture on main — does not restart watch."
    )]
    fn branch_capture_session(
        &self,
        Parameters(args): Parameters<BranchCaptureArgs>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_id = self.with_store(|store| {
            store
                .resolve_workspace_id(&args.workspace)
                .map_err(|e| DbError::InvalidInput(e.to_string()))
        })?;
        let record = self.with_capture(|cap| create_capture_branch(cap, &workspace_id, &args.label))?;
        ok_json(json!({
            "branch": record,
            "workspace_id": workspace_id,
            "capture_branch": record.slug,
            "hint": "Chat continues on branch line; commit_checkpoint writes to branch. merge_capture_branch when done."
        }))
    }

    #[tool(
        description = "Merge a capture branch back to parent (GCC merge). outcome: confirmed | rejected."
    )]
    fn merge_capture_branch(
        &self,
        Parameters(args): Parameters<MergeBranchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let merged = self.with_capture(|cap| merge_branch_record(cap, &args.branch_id, &args.outcome))?;
        ok_json(json!({ "branch": merged }))
    }

    #[tool(
        description = "List capture branches for a workspace (parent or branch side)."
    )]
    fn list_capture_branches(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let workspace_id = self.with_store(|store| {
            store
                .resolve_workspace_id(&workspace_id)
                .map_err(|e| DbError::InvalidInput(e.to_string()))
        })?;
        let branches = self.with_capture(|cap| list_branches_for_workspace(cap, &workspace_id))?;
        ok_json(json!({ "branches": branches }))
    }

    #[tool(
        description = "Update workspace name, goal, and/or template. Omitted fields are unchanged."
    )]
    fn update_workspace(
        &self,
        Parameters(args): Parameters<UpdateWorkspaceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ws = self.with_store(|store| {
            let existing = store.get_workspace(&args.workspace_id)?;
            let name = args.name.as_deref().unwrap_or(&existing.name);
            let goal = args.goal.as_deref().unwrap_or(&existing.goal);
            let template = args
                .template
                .as_deref()
                .unwrap_or(existing.template.as_str());
            store.update_workspace(&args.workspace_id, name, goal, template)
        })?;
        ok_json(json!({ "workspace": ws }))
    }

    #[tool(description = "Create a workspace (bug bounty program, CTF, etc.). Returns the new workspace.")]
    fn create_workspace(
        &self,
        Parameters(args): Parameters<CreateWorkspaceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let ws = self.with_store(|store| {
            store.create_workspace(&args.name, &args.goal, &args.template)
        })?;
        ok_json(json!({ "workspace": ws }))
    }

    #[tool(
        description = "Log a hypothesis — uncertain or testable claim. Text is stored exactly as provided."
    )]
    fn create_hypothesis(
        &self,
        Parameters(args): Parameters<TextNodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.with_store(|store| store.create_hypothesis(&args.workspace_id, &args.text))?;
        ok_json(json!({ "hypothesis": node }))
    }

    #[tool(
        description = "Log an action — a test or operation you performed (curl, scan, manual step). Text stored verbatim."
    )]
    fn create_action(
        &self,
        Parameters(args): Parameters<TextNodeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.with_store(|store| store.create_action(&args.workspace_id, &args.text))?;
        ok_json(json!({ "action": node }))
    }

    #[tool(
        description = "Log evidence — raw observed output (status codes, tool output, response snippets). Not interpretation."
    )]
    fn create_evidence(
        &self,
        Parameters(args): Parameters<CreateEvidenceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.with_store(|store| {
            store.create_evidence(
                &args.workspace_id,
                &args.text,
                args.source.as_deref(),
            )
        })?;
        ok_json(json!({ "evidence": node }))
    }

    #[tool(
        description = "Save a conclusion — requires at least one hypothesis_id and one evidence_id. Text stored verbatim."
    )]
    fn save_conclusion(
        &self,
        Parameters(args): Parameters<SaveConclusionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let node = self.with_store(|store| {
            store.save_conclusion(
                &args.workspace_id,
                &args.text,
                &args.outcome,
                &args.tag,
                args.confidence,
                &args.hypothesis_ids,
                &args.evidence_ids,
            )
        })?;
        ok_json(json!({ "conclusion": node }))
    }

    #[tool(
        description = "Workspace reasoning health — orphan blocks, stale investigations, dead ends, open hypotheses with no action, decision log. Call before suggesting next tests."
    )]
    fn get_workspace_hygiene(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let report = self.with_store(|store| store.fetch_workspace_hygiene(&workspace_id))?;
        ok_json(json!({ "hygiene": report }))
    }

    #[tool(
        description = "Save a reasoning block — one timeline row with optional hypothesis, action, evidence, conclusion. Title-only blocks are allowed on create. Text verbatim. On update (block_id or block_title), omitted fields are preserved — only send fields you want to change. Prefer block_title when the user names a block. Prefer this over create_* tools."
    )]
    fn save_block(
        &self,
        Parameters(args): Parameters<SaveBlockArgs>,
    ) -> Result<CallToolResult, McpError> {
        let block = self.with_store(|store| {
            store.save_block(SaveBlockInput {
                workspace_id: args.workspace_id,
                block_id: args.block_id,
                block_title: args.block_title,
                title: args.title,
                hypothesis_text: args.hypothesis_text,
                action_text: args.action_text,
                evidence_text: args.evidence_text,
                evidence_source: args.evidence_source,
                conclusion_text: args.conclusion_text,
                conclusion_outcome: args.conclusion_outcome,
                conclusion_tag: args.conclusion_tag,
                confidence_level: args.confidence_level,
                belief_state: args.belief_state,
                system_tag: args.system_tag,
                user_tag: args.user_tag,
                link_to_block_ids: args.link_to_block_ids.unwrap_or_default(),
            })
        })?;
        ok_json(json!({ "block": block }))
    }

    #[tool(
        description = "List blocks in a workspace with id and title — use to resolve block_title before save_block updates."
    )]
    fn list_blocks(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let blocks = self.with_store(|store| store.fetch_blocks(&workspace_id, false))?;
        let list: Vec<_> = blocks
            .iter()
            .map(|b| {
                json!({
                    "id": b.id,
                    "title": b.title,
                    "belief_state": b.belief_state,
                    "incomplete": b.incomplete,
                })
            })
            .collect();
        ok_json(json!({ "blocks": list }))
    }

    #[tool(
        description = "Read one reasoning block by block_id or block_title — full hypothesis/action/evidence/conclusion fields."
    )]
    fn get_block(
        &self,
        Parameters(args): Parameters<GetBlockArgs>,
    ) -> Result<CallToolResult, McpError> {
        let block = self.with_store(|store| {
            store.get_block_in_workspace(
                &args.workspace_id,
                args.block_id,
                args.block_title,
            )
        })?;
        ok_json(json!({ "block": block }))
    }

    #[tool(
        description = "Delete (soft-delete) a reasoning block by block_id or block_title. Use list_blocks to resolve IDs."
    )]
    fn delete_block(
        &self,
        Parameters(args): Parameters<DeleteBlockArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.with_store(|store| {
            store.delete_block(
                &args.workspace_id,
                args.block_id,
                args.block_title,
            )
        })?;
        ok_json(json!({ "deleted": true }))
    }

    #[tool(
        description = "Permanently delete a workspace and all its blocks, links, and related data. Irreversible."
    )]
    fn delete_workspace(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.with_store(|store| store.delete_workspace(&workspace_id))?;
        ok_json(json!({ "deleted": true, "workspace_id": workspace_id }))
    }

    #[tool(
        description = "Link two nodes. Allowed: hypothesis→action, action→evidence, conclusion→hypothesis, conclusion→evidence."
    )]
    fn add_link(
        &self,
        Parameters(args): Parameters<AddLinkArgs>,
    ) -> Result<CallToolResult, McpError> {
        let link = self.with_store(|store| {
            store.add_link(
                &args.workspace_id,
                &args.from_type,
                &args.from_id,
                &args.to_type,
                &args.to_id,
            )
        })?;
        ok_json(json!({ "link": link }))
    }

    #[tool(
        description = "List node links in a workspace (hypothesis→action, etc.) — use link id with remove_link."
    )]
    fn list_links(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let links = self.with_store(|store| store.list_links_for_workspace(&workspace_id))?;
        let list: Vec<_> = links
            .iter()
            .map(|l| {
                json!({
                    "id": l.id,
                    "from_type": l.from_type.as_str(),
                    "from_id": l.from_id,
                    "to_type": l.to_type.as_str(),
                    "to_id": l.to_id,
                })
            })
            .collect();
        ok_json(json!({ "links": list }))
    }

    #[tool(
        description = "Remove a node link by link_id from list_links."
    )]
    fn remove_link(
        &self,
        Parameters(args): Parameters<RemoveLinkArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.with_store(|store| store.remove_link(&args.link_id))?;
        ok_json(json!({ "removed": true, "link_id": args.link_id }))
    }

    #[tool(
        description = "List block-to-block links in a workspace — use link id with remove_block_link."
    )]
    fn list_block_links(
        &self,
        Parameters(WorkspaceIdArgs { workspace_id }): Parameters<WorkspaceIdArgs>,
    ) -> Result<CallToolResult, McpError> {
        let links = self.with_store(|store| store.list_block_links(&workspace_id))?;
        ok_json(json!({ "block_links": links }))
    }

    #[tool(
        description = "Remove a block-to-block link by link_id from list_block_links."
    )]
    fn remove_block_link(
        &self,
        Parameters(args): Parameters<RemoveBlockLinkArgs>,
    ) -> Result<CallToolResult, McpError> {
        self.with_store(|store| store.remove_block_link(&args.link_id))?;
        ok_json(json!({ "removed": true, "link_id": args.link_id }))
    }
}

#[tool_handler]
impl ServerHandler for ContextLayerMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .build(),
        )
        .with_instructions(
            "ContextLayer reasoning graph MCP. Same database as the desktop app (~/.contextlayer/graph.db). \
             Record the user's exact wording in text fields — do not rewrite or normalize. \
             Only write when the user asks you to log something. \
             Before suggesting tests, call get_workspace_index (tier 1) then get_block for details — avoid loading full workspace text. get_workspace_hygiene for open loops and dead ends. \
             Flow: one block per reasoning step; give each block a short title when you can; fill any subset of hypothesis, action, evidence, conclusion — title-only placeholders are OK. \
             On save_block updates, only include fields you are changing — omitted fields are kept. Use block_title or list_blocks to target a block by name. \
             Use get_block to read one block; compile_agent_context for a full workspace packet; update_workspace to rename or change goal; delete_block or delete_workspace to remove data. \
             export_blocks accepts block_ids and/or block_titles for PR markdown. \
             Capture: session log + seq-anchored commits under ~/.contextlayer/capture/{workspace_id}/. \
             Live capture is opt-in: call start_capture for a workspace before an investigation (optionally scope cursor_project or transcript_path). \
             Run contextlayer-recorder watch in background while sessions are active; stop_capture when done. \
             bind_capture_project only maps Cursor project → workspace — it does not record by itself. \
             get_context_log / get_context_commits for tiered reads; commit_checkpoint slices the log. \
             list_links / remove_link for node links; list_block_links / remove_block_link for block links. \
             Prefer save_block over individual create_* tools."
                .to_string(),
        )
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_path = std::env::var("CONTEXTLAYER_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_db_path());

    let server = ContextLayerMcp::new(db_path);
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
