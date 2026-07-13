use std::path::{Path, PathBuf};
use std::sync::Mutex;

use contextlayer_db::{default_db_path, BlockEntry, GraphStore, PickerNode, SaveBlockInput, TimelineEntry};
use contextlayer_export::compile_workspace_summary_markdown;
use contextlayer_export::{compile_agent_context_markdown, compile_pr_export_markdown_with_options, PrExportOptions};
use contextlayer_trace::{
    begin_capture_session, capture_log_boundary_available, capture_scope_label,
    compile_pr_trace_appendix_with_options, list_active_sessions, list_picker_candidates,
    load_workspace_capture_prefs, parse_log_slice_mode, prune_unscoped_capture_sessions,
    save_workspace_capture_prefs, session_message_count, stop_capture_session,
    CaptureStartResult, CaptureStore, PrTraceAppendixOptions, TraceStore, WorkspaceCapturePrefs,
    DEFAULT_LOG_SLICE,
};
use tauri::{RunEvent, State};

mod capture_watcher;

struct AppState {
    db_path: PathBuf,
    store: Mutex<Option<GraphStore>>,
}

impl AppState {
    fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            store: Mutex::new(None),
        }
    }

    fn with_store<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&GraphStore) -> Result<T, contextlayer_db::DbError>,
    {
        let mut guard = self.store.lock().map_err(|e| e.to_string())?;
        if guard.is_none() {
            *guard = Some(
                GraphStore::open(&self.db_path).map_err(|e| e.to_string())?,
            );
        }
        f(guard.as_ref().unwrap()).map_err(|e| e.to_string())
    }

    /// Drop cached SQLite handle so the next read sees MCP / external writes (WAL).
    fn invalidate_store(&self) -> Result<(), String> {
        let mut guard = self.store.lock().map_err(|e| e.to_string())?;
        *guard = None;
        Ok(())
    }
}

fn resolve_bundled_tool(exe_name: &str) -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = exe_dir.join(exe_name);
    if bundled.is_file() {
        return Some(bundled);
    }

    // Dev fallback: workspace target/release next to the monorepo root.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev = manifest
        .join("../../../target/release")
        .join(exe_name);
    if dev.is_file() {
        return Some(dev);
    }
    None
}

#[tauri::command]
fn get_bundled_tool_paths() -> Result<serde_json::Value, String> {
    let recorder = resolve_bundled_tool("contextlayer-recorder.exe");
    let mcp = resolve_bundled_tool("contextlayer-mcp.exe");
    let trace = resolve_bundled_tool("contextlayer-trace.exe");
    let install_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .map(|p| p.display().to_string());

    let mcp_json = mcp.as_ref().map(|path| {
        serde_json::json!({
            "mcpServers": {
                "contextlayer": {
                    "command": path.display().to_string(),
                    "args": []
                }
            }
        })
    });

    Ok(serde_json::json!({
        "install_dir": install_dir,
        "recorder": recorder.map(|p| p.display().to_string()),
        "mcp": mcp.map(|p| p.display().to_string()),
        "trace": trace.map(|p| p.display().to_string()),
        "capture_watcher_running": capture_watcher::is_running(),
        "mcp_json": mcp_json,
    }))
}

#[tauri::command]
fn get_db_path(state: State<'_, AppState>) -> String {
    state.db_path.display().to_string()
}

#[tauri::command]
fn init_database(state: State<'_, AppState>) -> Result<String, String> {
    let path = state.db_path.clone();
    {
        let mut guard = state.store.lock().map_err(|e| e.to_string())?;
        *guard = Some(GraphStore::open(&path).map_err(|e| e.to_string())?);
    }
    state.with_store(|store| store.seed_dogfood_if_empty())?;
    Ok(path.display().to_string())
}

#[tauri::command]
fn list_workspaces(
    state: State<'_, AppState>,
    include_archived: Option<bool>,
) -> Result<Vec<contextlayer_core::Workspace>, String> {
    state.with_store(|store| store.list_workspaces(include_archived.unwrap_or(false)))
}

#[tauri::command]
fn set_workspace_archived(
    state: State<'_, AppState>,
    id: String,
    archived: bool,
) -> Result<contextlayer_core::Workspace, String> {
    state.with_store(|store| store.set_workspace_archived(&id, archived))
}

#[tauri::command]
fn create_workspace(
    state: State<'_, AppState>,
    name: String,
    goal: String,
    template: String,
) -> Result<contextlayer_core::Workspace, String> {
    state.with_store(|store| store.create_workspace(&name, &goal, &template))
}

#[tauri::command]
fn update_workspace(
    state: State<'_, AppState>,
    id: String,
    name: String,
    goal: String,
    template: String,
) -> Result<contextlayer_core::Workspace, String> {
    state.with_store(|store| store.update_workspace(&id, &name, &goal, &template))
}

#[tauri::command]
fn create_hypothesis(
    state: State<'_, AppState>,
    workspace_id: String,
    text: String,
) -> Result<contextlayer_core::Hypothesis, String> {
    state.with_store(|store| store.create_hypothesis(&workspace_id, &text))
}

#[tauri::command]
fn create_action(
    state: State<'_, AppState>,
    workspace_id: String,
    text: String,
) -> Result<contextlayer_core::Action, String> {
    state.with_store(|store| store.create_action(&workspace_id, &text))
}

#[tauri::command]
fn create_evidence(
    state: State<'_, AppState>,
    workspace_id: String,
    text: String,
    source: Option<String>,
) -> Result<contextlayer_core::Evidence, String> {
    state.with_store(|store| store.create_evidence(&workspace_id, &text, source.as_deref()))
}

#[tauri::command]
fn save_conclusion(
    state: State<'_, AppState>,
    workspace_id: String,
    text: String,
    outcome: String,
    tag: String,
    confidence: Option<f64>,
    hypothesis_ids: Vec<String>,
    evidence_ids: Vec<String>,
) -> Result<contextlayer_core::Conclusion, String> {
    state.with_store(|store| {
        store.save_conclusion(
            &workspace_id,
            &text,
            &outcome,
            &tag,
            confidence,
            &hypothesis_ids,
            &evidence_ids,
        )
    })
}

#[tauri::command]
fn add_link(
    state: State<'_, AppState>,
    workspace_id: String,
    from_type: String,
    from_id: String,
    to_type: String,
    to_id: String,
) -> Result<contextlayer_core::NodeLink, String> {
    state.with_store(|store| {
        store.add_link(&workspace_id, &from_type, &from_id, &to_type, &to_id)
    })
}

#[tauri::command]
fn remove_link(state: State<'_, AppState>, link_id: String) -> Result<(), String> {
    state.with_store(|store| store.remove_link(&link_id))
}

#[tauri::command]
fn soft_delete_node(
    state: State<'_, AppState>,
    node_type: String,
    node_id: String,
) -> Result<(), String> {
    state.with_store(|store| store.soft_delete_node(&node_type, &node_id))
}

#[tauri::command]
fn edit_hypothesis(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> Result<contextlayer_core::Hypothesis, String> {
    state.with_store(|store| store.edit_hypothesis(&id, &text))
}

#[tauri::command]
fn fetch_workspace_hygiene(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<contextlayer_db::WorkspaceHygieneReport, String> {
    state.with_store(|store| store.fetch_workspace_hygiene(&workspace_id))
}

#[tauri::command]
fn fetch_blocks(
    state: State<'_, AppState>,
    workspace_id: String,
    ascending: bool,
    fresh: Option<bool>,
) -> Result<Vec<BlockEntry>, String> {
    if fresh.unwrap_or(false) {
        state.invalidate_store()?;
    }
    state.with_store(|store| store.fetch_blocks(&workspace_id, ascending))
}

#[tauri::command]
fn save_block(
    state: State<'_, AppState>,
    workspace_id: String,
    block_id: Option<String>,
    block_title: Option<String>,
    title: Option<String>,
    hypothesis_text: Option<String>,
    action_text: Option<String>,
    evidence_text: Option<String>,
    evidence_source: Option<String>,
    conclusion_text: Option<String>,
    conclusion_outcome: Option<String>,
    conclusion_tag: Option<String>,
    confidence_level: Option<String>,
    belief_state: Option<String>,
    system_tag: Option<String>,
    user_tag: Option<String>,
    link_to_block_ids: Vec<String>,
) -> Result<BlockEntry, String> {
    state.with_store(|store| {
        store.save_block(SaveBlockInput {
            workspace_id,
            block_id,
            block_title,
            title,
            hypothesis_text,
            action_text,
            evidence_text,
            evidence_source,
            conclusion_text,
            conclusion_outcome,
            conclusion_tag,
            confidence_level,
            belief_state,
            system_tag,
            user_tag,
            link_to_block_ids,
        })
    })
}

#[tauri::command]
fn soft_delete_block(state: State<'_, AppState>, block_id: String) -> Result<(), String> {
    state.with_store(|store| store.soft_delete_block(&block_id))
}

#[tauri::command]
fn list_blocks_for_picker(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<(String, String)>, String> {
    state.with_store(|store| store.list_blocks_for_picker(&workspace_id))
}

#[tauri::command]
fn add_block_link(
    state: State<'_, AppState>,
    workspace_id: String,
    from_block_id: String,
    to_block_id: String,
) -> Result<contextlayer_core::BlockLink, String> {
    state.with_store(|store| store.add_block_link(&workspace_id, &from_block_id, &to_block_id))
}

#[tauri::command]
fn fetch_timeline(
    state: State<'_, AppState>,
    workspace_id: String,
    ascending: bool,
    types: Option<Vec<String>>,
) -> Result<Vec<TimelineEntry>, String> {
    state.with_store(|store| store.fetch_timeline(&workspace_id, ascending, types))
}

#[tauri::command]
fn list_picker_nodes(
    state: State<'_, AppState>,
    workspace_id: String,
    node_type: String,
) -> Result<Vec<PickerNode>, String> {
    state.with_store(|store| store.list_nodes_for_picker(&workspace_id, &node_type))
}

#[tauri::command]
fn export_workspace_summary(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<String, String> {
    state.with_store(|store| {
        compile_workspace_summary_markdown(store, &workspace_id).map_err(|e| {
            contextlayer_db::DbError::InvalidInput(e)
        })
    })
}

#[tauri::command]
async fn export_pr_reasoning(
    state: State<'_, AppState>,
    workspace_id: String,
    block_ids: Vec<String>,
    branch: Option<String>,
    pr_number: Option<String>,
    git_sha: Option<String>,
    include_trace_checkpoints: Option<bool>,
    include_trace_log: Option<bool>,
    include_trace_branch_logs: Option<bool>,
    include_trace: Option<bool>,
    trace_log_slice: Option<String>,
) -> Result<String, String> {
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        run_export_pr_reasoning(
            &db_path,
            &workspace_id,
            &block_ids,
            branch,
            pr_number,
            git_sha,
            include_trace_checkpoints,
            include_trace_log,
            include_trace_branch_logs,
            include_trace,
            trace_log_slice,
        )
    })
    .await
    .map_err(|e| format!("export task failed: {e}"))?
}

fn run_export_pr_reasoning(
    db_path: &Path,
    workspace_id: &str,
    block_ids: &[String],
    branch: Option<String>,
    pr_number: Option<String>,
    git_sha: Option<String>,
    include_trace_checkpoints: Option<bool>,
    include_trace_log: Option<bool>,
    include_trace_branch_logs: Option<bool>,
    include_trace: Option<bool>,
    trace_log_slice: Option<String>,
) -> Result<String, String> {
    let trace_appendix = if include_trace == Some(false) {
        None
    } else {
        let capture = CaptureStore::default_open().map_err(|e| e.to_string())?;
        let log_slice_mode = parse_log_slice_mode(
            trace_log_slice
                .as_deref()
                .unwrap_or(DEFAULT_LOG_SLICE),
        )?;
        let opts = PrTraceAppendixOptions {
            include_checkpoints: include_trace_checkpoints.unwrap_or(true),
            include_log: include_trace_log.unwrap_or(false),
            include_branch_logs: include_trace_branch_logs.unwrap_or(false),
            log_slice_mode,
            ..PrTraceAppendixOptions::default()
        };
        compile_pr_trace_appendix_with_options(&capture, workspace_id, &opts)
            .map_err(|e| e.to_string())?
    };
    let git_sha = git_sha.or_else(detect_git_sha);
    let branch = branch.or_else(detect_git_branch);
    let options = PrExportOptions {
        branch,
        pr_number,
        git_sha,
        trace_appendix,
    };
    let store = GraphStore::open(db_path).map_err(|e| e.to_string())?;
    compile_pr_export_markdown_with_options(&store, workspace_id, block_ids, &options)
        .map_err(|e| e.to_string())
}

fn git_command() -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn detect_git_branch() -> Option<String> {
    let out = git_command()
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn detect_git_sha() -> Option<String> {
    let out = git_command()
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[tauri::command]
fn get_git_context() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "branch": detect_git_branch(),
        "git_sha": detect_git_sha(),
    }))
}

#[tauri::command]
fn list_capture_candidates_cmd() -> Result<serde_json::Value, String> {
    let candidates = list_picker_candidates();
    Ok(serde_json::json!({ "candidates": candidates }))
}

#[tauri::command]
fn get_workspace_capture_prefs_cmd(workspace_id: String) -> Result<WorkspaceCapturePrefs, String> {
    Ok(load_workspace_capture_prefs(&workspace_id))
}

#[tauri::command]
fn set_workspace_capture_prefs_cmd(prefs: WorkspaceCapturePrefs) -> Result<(), String> {
    save_workspace_capture_prefs(&prefs)
}

#[tauri::command]
fn start_capture_cmd(
    workspace_id: String,
    label: Option<String>,
    cursor_project: Option<String>,
    transcript_path: Option<String>,
    remember_scope: Option<bool>,
) -> Result<serde_json::Value, String> {
    let _ = prune_unscoped_capture_sessions();

    // Desktop: always let the user pick unless they already chose a thread.
    if cursor_project.is_none() && transcript_path.is_none() {
        let candidates = list_picker_candidates();
        if candidates.is_empty() {
            return Ok(serde_json::json!({
                "status": "no_candidates",
                "hint": "No chats found in the last 7 days. Send a message in your editor, then try again.",
            }));
        }
        return Ok(serde_json::json!({
            "status": "needs_picker",
            "candidates": candidates,
        }));
    }

    let result = begin_capture_session(
        &workspace_id,
        cursor_project,
        transcript_path,
        label,
        remember_scope.unwrap_or(false),
    )?;
    match result {
        CaptureStartResult::Started(outcome) => {
            capture_watcher::ensure_running();
            Ok(serde_json::json!({
                "status": "started",
                "session": outcome.session,
                "baselined_transcript_files": outcome.baselined,
                "scope_label": outcome.scope_label,
                "capture_watcher_running": true,
            }))
        }
        CaptureStartResult::NeedsPicker { candidates } => Ok(serde_json::json!({
            "status": "needs_picker",
            "candidates": candidates,
        })),
        CaptureStartResult::NoCandidates { hint } => Ok(serde_json::json!({
            "status": "no_candidates",
            "hint": hint,
        })),
    }
}

#[tauri::command]
fn stop_capture_cmd(workspace_id: String) -> Result<serde_json::Value, String> {
    let stopped = stop_capture_session(&workspace_id).map_err(|e| e.to_string())?;
    let remaining = list_active_sessions().map_err(|e| e.to_string())?;
    if remaining.is_empty() {
        capture_watcher::stop();
    }
    Ok(serde_json::json!({
        "stopped": stopped,
        "capture_watcher_running": capture_watcher::is_running(),
    }))
}

#[tauri::command]
fn capture_status_cmd(workspace_id: String) -> Result<serde_json::Value, String> {
    let active = list_active_sessions().map_err(|e| e.to_string())?;
    let session = active.iter().find(|s| s.workspace_id == workspace_id);
    let capture = CaptureStore::default_open().map_err(|e| e.to_string())?;
    let summary = capture.summary(&workspace_id)?;
    let session_message_count = session
        .map(session_message_count)
        .transpose()?
        .unwrap_or(0);
    let scope_label = session.and_then(|s| {
        match (&s.cursor_project, &s.transcript_path) {
            (Some(p), Some(path)) => Some(capture_scope_label(p, path)),
            (Some(p), None) => Some(p.clone()),
            _ => None,
        }
    });
    Ok(serde_json::json!({
        "active_session": session,
        "summary": summary,
        "session_message_count": session_message_count,
        "scope_label": scope_label,
        "capture_log_boundary_available": capture_log_boundary_available(&workspace_id)?,
    }))
}

#[tauri::command]
fn export_agent_context(
    state: State<'_, AppState>,
    workspace_id: String,
    block_ids: Vec<String>,
) -> Result<String, String> {
    state.with_store(|store| {
        compile_agent_context_markdown(store, &workspace_id, &block_ids).map_err(|e| {
            contextlayer_db::DbError::InvalidInput(e)
        })
    })
}

#[tauri::command]
fn commit_trace_checkpoint(
    workspace_id: String,
    intent: String,
    note: String,
    rejected_paths: Vec<String>,
    git_sha: Option<String>,
    block_ids: Vec<String>,
) -> Result<serde_json::Value, String> {
    let store = TraceStore::default_open().map_err(|e| e.to_string())?;
    if store.capture().read_log_messages(&workspace_id)?.is_empty() {
        let body = if note.trim().is_empty() {
            intent.clone()
        } else {
            format!("{intent}\n\n{note}")
        };
        store.capture().append_message(
            &workspace_id,
            "system",
            &body,
            "desktop_checkpoint",
            None,
        )?;
    }
    let cp = store.commit_checkpoint(
        &workspace_id,
        &intent,
        &note,
        rejected_paths,
        git_sha,
        block_ids,
    )?;
    serde_json::to_value(cp).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_trace_summary_cmd(workspace_id: String) -> Result<serde_json::Value, String> {
    let store = TraceStore::default_open().map_err(|e| e.to_string())?;
    let summary = store.summary(&workspace_id)?;
    serde_json::to_value(summary).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = default_db_path();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new(db_path))
        .setup(|_| {
            if list_active_sessions()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
            {
                capture_watcher::ensure_running();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bundled_tool_paths,
            get_db_path,
            init_database,
            list_workspaces,
            set_workspace_archived,
            create_workspace,
            update_workspace,
            create_hypothesis,
            create_action,
            create_evidence,
            save_conclusion,
            add_link,
            remove_link,
            soft_delete_node,
            edit_hypothesis,
            fetch_workspace_hygiene,
            fetch_blocks,
            save_block,
            soft_delete_block,
            list_blocks_for_picker,
            add_block_link,
            fetch_timeline,
            list_picker_nodes,
            export_workspace_summary,
            export_pr_reasoning,
            export_agent_context,
            get_git_context,
            start_capture_cmd,
            stop_capture_cmd,
            capture_status_cmd,
            list_capture_candidates_cmd,
            get_workspace_capture_prefs_cmd,
            set_workspace_capture_prefs_cmd,
            commit_trace_checkpoint,
            get_trace_summary_cmd,
        ])
        .build(tauri::generate_context!())
        .expect("error while building ContextLayer")
        .run(|_app_handle, event| {
            if matches!(event, RunEvent::Exit) {
                capture_watcher::stop();
            }
        });
}
