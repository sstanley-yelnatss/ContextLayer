//! Opt-in capture sessions — live transcript ingest only while a session is active.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capture::{
    default_capture_branch_name, load_bindings, load_recorder_state, save_recorder_state,
    ProjectBindings, RecorderFileState, RecorderState,
};
use crate::cursor::{
    cursor_project_key_from_transcript_path, cursor_projects_root, discover_transcript_files,
    resolve_workspace_for_cursor_project,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSession {
    pub id: String,
    pub workspace_id: String,
    pub started_at: String,
    #[serde(default)]
    pub label: String,
    /// When set, only transcripts under this Cursor project folder key are ingested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor_project: Option<String>,
    /// When set, only this transcript file (absolute path) is ingested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
    /// Active capture line: `main` or branch slug under `capture/{ws}/branches/{slug}/`.
    #[serde(default = "default_capture_branch_name")]
    pub capture_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptureSessionStore {
    #[serde(default)]
    pub active: Vec<CaptureSession>,
}

pub fn sessions_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".contextlayer")
        .join("capture_sessions.json")
}

pub fn load_capture_sessions() -> Result<CaptureSessionStore, String> {
    let path = sessions_path();
    if !path.exists() {
        return Ok(CaptureSessionStore::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_capture_sessions(store: &CaptureSessionStore) -> Result<(), String> {
    let path = sessions_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

pub fn list_active_sessions() -> Result<Vec<CaptureSession>, String> {
    Ok(load_capture_sessions()?.active)
}

/// Returns true if this transcript file should ingest for any active session.
pub fn session_allows_transcript(session: &CaptureSession, project_key: &str, path: &Path) -> bool {
    if let Some(ref cp) = session.cursor_project {
        if cp != project_key {
            return false;
        }
    }
    if let Some(ref tp) = session.transcript_path {
        let want = PathBuf::from(tp);
        let Ok(canonical) = path.canonicalize() else {
            return false;
        };
        let Ok(want_canonical) = want.canonicalize() else {
            return path.to_string_lossy() == tp.as_str();
        };
        return canonical == want_canonical;
    }
    true
}

pub fn active_session_for_workspace<'a>(
    sessions: &'a [CaptureSession],
    workspace_id: &str,
) -> Option<&'a CaptureSession> {
    sessions.iter().find(|s| s.workspace_id == workspace_id)
}

/// Composite recorder key — separate byte offsets per workspace + branch line.
pub fn recorder_state_key(path_key: &str, workspace_id: &str, capture_branch: &str) -> String {
    format!("{path_key}#{workspace_id}#{capture_branch}")
}

pub fn active_capture_branch_slug(workspace_id: &str) -> Result<Option<String>, String> {
    let sessions = load_capture_sessions()?.active;
    let Some(session) = active_session_for_workspace(&sessions, workspace_id) else {
        return Ok(None);
    };
    if session.capture_branch == "main" {
        Ok(None)
    } else {
        Ok(Some(session.capture_branch.clone()))
    }
}

pub fn should_ingest_transcript(
    sessions: &[CaptureSession],
    workspace_id: &str,
    project_key: &str,
    path: &Path,
) -> bool {
    sessions.iter().any(|s| {
        s.workspace_id == workspace_id && session_allows_transcript(s, project_key, path)
    })
}

fn count_file_lines(path: &Path) -> Result<u64, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(text.lines().filter(|l| !l.trim().is_empty()).count() as u64)
}

/// Set byte offsets to end-of-file so only *new* transcript lines ingest after start.
pub fn baseline_transcripts_for_session(
    state: &mut RecorderState,
    workspace_id: &str,
    capture_branch: &str,
    cursor_project: Option<&str>,
    transcript_path: Option<&str>,
    bindings: &ProjectBindings,
) -> Result<u32, String> {
    let mut baselined = 0u32;
    for path in discover_transcript_files(&cursor_projects_root()) {
        let Some(project_key) = cursor_project_key_from_transcript_path(&path) else {
            continue;
        };
        if let Some(cp) = cursor_project {
            if project_key != cp {
                continue;
            }
        }
        let Some(bound_ws) = resolve_workspace_for_cursor_project(bindings, &project_key) else {
            continue;
        };
        if bound_ws != workspace_id {
            continue;
        }
        if let Some(tp) = transcript_path {
            let matches = path
                .canonicalize()
                .ok()
                .zip(PathBuf::from(tp).canonicalize().ok())
                .map(|(a, b)| a == b)
                .unwrap_or_else(|| path.to_string_lossy() == tp);
            if !matches {
                continue;
            }
        }
        let len = fs::metadata(&path).map_err(|e| e.to_string())?.len();
        let line_count = count_file_lines(&path)?;
        let path_key = path.to_string_lossy().to_string();
        let state_key = recorder_state_key(&path_key, workspace_id, capture_branch);
        state.files.insert(
            state_key,
            RecorderFileState {
                byte_offset: len,
                workspace_id: workspace_id.to_string(),
                capture_branch: capture_branch.to_string(),
                composer_id: path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string()),
                lines_ingested: line_count,
            },
        );
        baselined += 1;
    }
    Ok(baselined)
}

/// Start an opt-in capture session. Replaces any existing session for the same workspace.
pub fn start_capture_session(
    workspace_id: &str,
    cursor_project: Option<String>,
    transcript_path: Option<String>,
    label: Option<String>,
) -> Result<(CaptureSession, u32), String> {
    let bindings = load_bindings()?;
    let mut state = load_recorder_state()?;
    let baselined = baseline_transcripts_for_session(
        &mut state,
        workspace_id,
        "main",
        cursor_project.as_deref(),
        transcript_path.as_deref(),
        &bindings,
    )?;
    save_recorder_state(&state)?;

    let session = CaptureSession {
        id: Uuid::new_v4().to_string(),
        workspace_id: workspace_id.to_string(),
        started_at: Utc::now().to_rfc3339(),
        label: label.unwrap_or_default(),
        cursor_project,
        transcript_path,
        capture_branch: "main".to_string(),
    };

    let mut store = load_capture_sessions()?;
    store
        .active
        .retain(|s| s.workspace_id != workspace_id);
    store.active.push(session.clone());
    save_capture_sessions(&store)?;

    Ok((session, baselined))
}

pub fn stop_capture_session(workspace_id: &str) -> Result<Option<CaptureSession>, String> {
    let mut store = load_capture_sessions()?;
    let idx = store
        .active
        .iter()
        .position(|s| s.workspace_id == workspace_id);
    let Some(idx) = idx else {
        return Ok(None);
    };
    let removed = store.active.remove(idx);
    save_capture_sessions(&store)?;
    Ok(Some(removed))
}

pub fn stop_capture_session_by_id(session_id: &str) -> Result<Option<CaptureSession>, String> {
    let mut store = load_capture_sessions()?;
    let idx = store.active.iter().position(|s| s.id == session_id);
    let Some(idx) = idx else {
        return Ok(None);
    };
    let removed = store.active.remove(idx);
    save_capture_sessions(&store)?;
    Ok(Some(removed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_scopes_by_project_and_path() {
        let session = CaptureSession {
            id: "1".into(),
            workspace_id: "ws".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: Some("proj-a".into()),
            transcript_path: None,
            capture_branch: "main".into(),
        };
        assert!(session_allows_transcript(
            &session,
            "proj-a",
            Path::new("/tmp/x.jsonl")
        ));
        assert!(!session_allows_transcript(
            &session,
            "proj-b",
            Path::new("/tmp/x.jsonl")
        ));
    }
}
