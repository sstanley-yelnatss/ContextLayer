//! ContextLayer minimal MCP — stdio read/write lane to ~/.contextlayer/graph.db
//! No text normalization; records user/agent wording as provided.

use std::path::PathBuf;

use contextlayer_db::{default_db_path, DbError, GraphStore, SaveBlockInput};
use contextlayer_export::compile_workspace_summary_markdown;
use contextlayer_export::compile_pr_export_markdown;
use contextlayer_export::resolve_pr_block_ids;
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
    /// blank | security_hunt | product_research | decision_strategy
    #[serde(default = "default_template")]
    template: String,
}

fn default_template() -> String {
    "security_hunt".to_string()
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
    /// blank | security_hunt | product_research | decision_strategy — omit to keep current.
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
    /// Block UUIDs from list_blocks — use block_ids or block_titles (or both).
    #[serde(default)]
    block_ids: Vec<String>,
    /// Case-insensitive block titles — alternative to block_ids.
    #[serde(default)]
    block_titles: Vec<String>,
}

#[tool_router]
impl ContextLayerMcp {
    #[tool(
        description = "List all ContextLayer workspaces (id, name, goal, template). Call before logging if workspace is unknown."
    )]
    fn list_workspaces(&self) -> Result<CallToolResult, McpError> {
        let list = self.with_store(|store| store.list_workspaces())?;
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
        let markdown = self.with_store(|store| {
            let ids = resolve_pr_block_ids(
                store,
                &args.workspace_id,
                &args.block_ids,
                &args.block_titles,
            )
            .map_err(DbError::InvalidInput)?;
            compile_pr_export_markdown(store, &args.workspace_id, &ids)
                .map_err(DbError::InvalidInput)
        })?;
        Ok(CallToolResult::success(vec![Content::text(markdown)]))
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
             Before suggesting tests, call get_workspace_summary and get_workspace_hygiene to avoid retesting ruled-out paths and orphans. \
             Flow: one block per reasoning step; give each block a short title when you can; fill any subset of hypothesis, action, evidence, conclusion — title-only placeholders are OK. \
             On save_block updates, only include fields you are changing — omitted fields are kept. Use block_title or list_blocks to target a block by name. \
             Use get_block to read one block; update_workspace to rename or change goal; delete_block or delete_workspace to remove data. \
             export_blocks accepts block_ids and/or block_titles. list_links / remove_link for node links; list_block_links / remove_block_link for block links. \
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
