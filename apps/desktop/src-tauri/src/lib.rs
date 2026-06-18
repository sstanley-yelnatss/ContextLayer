use std::path::PathBuf;
use std::sync::Mutex;

use contextlayer_db::{default_db_path, BlockEntry, GraphStore, PickerNode, SaveBlockInput, TimelineEntry};
use contextlayer_export::compile_workspace_summary_markdown;
use tauri::State;

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
fn list_workspaces(state: State<'_, AppState>) -> Result<Vec<contextlayer_core::Workspace>, String> {
    state.with_store(|store| store.list_workspaces())
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
) -> Result<Vec<BlockEntry>, String> {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = default_db_path();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new(db_path))
        .invoke_handler(tauri::generate_handler![
            get_db_path,
            init_database,
            list_workspaces,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running ContextLayer");
}
