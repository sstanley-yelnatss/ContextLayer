//! ContextLayer minimal MCP — stdio read/write lane to ~/.contextlayer/graph.db
//! No text normalization; records user/agent wording as provided.

use std::path::PathBuf;

use contextlayer_db::{default_db_path, DbError, GraphStore, SaveBlockInput};
use contextlayer_export::compile_workspace_summary_markdown;
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
    /// blank | security_hunt | product_research
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
    block_id: Option<String>,
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
        description = "Save a reasoning block — one timeline row with optional hypothesis, action, evidence, conclusion. Text verbatim. Prefer this over create_* tools."
    )]
    fn save_block(
        &self,
        Parameters(args): Parameters<SaveBlockArgs>,
    ) -> Result<CallToolResult, McpError> {
        let block = self.with_store(|store| {
            store.save_block(SaveBlockInput {
                workspace_id: args.workspace_id,
                block_id: args.block_id,
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
             Flow: one block per reasoning step; fill any subset of hypothesis, action, evidence, conclusion. \
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
