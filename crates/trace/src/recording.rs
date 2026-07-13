//! Opt-in capture sessions — live transcript ingest only while a session is active.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capture::{
    default_capture_branch_name, load_recorder_state, save_recorder_state, CaptureStore,
    RecorderFileState, RecorderState,
};
use crate::capture_scope::{persist_last_capture_boundary, transcript_paths_match};
use crate::transcripts::{discover_all_transcript_files, project_key_from_transcript_path};

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
    /// Log seq already present when this session started (counter baseline).
    #[serde(default)]
    pub log_seq_at_start: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CaptureSessionStore {
    #[serde(default)]
    pub active: Vec<CaptureSession>,
}

pub fn sessions_path() -> PathBuf {
    crate::capture::contextlayer_dir().join("capture_sessions.json")
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

/// Remove catch-all sessions with no chat scope (test leftovers / ghosts).
pub fn prune_unscoped_capture_sessions() -> Result<u32, String> {
    let mut store = load_capture_sessions()?;
    let before = store.active.len();
    store.active.retain(|s| {
        s.cursor_project.is_some() || s.transcript_path.is_some()
    });
    let removed = before.saturating_sub(store.active.len());
    if removed > 0 {
        save_capture_sessions(&store)?;
    }
    Ok(removed as u32)
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

/// Lower rank = more specific scope (transcript path beats project beats unscoped).
pub fn session_scope_rank(session: &CaptureSession) -> u8 {
    match (&session.transcript_path, &session.cursor_project) {
        (Some(_), _) => 0,
        (None, Some(_)) => 1,
        (None, None) => 2,
    }
}

/// Pick the best matching session for a transcript file (scoped sessions win over catch-alls).
pub fn matching_capture_session<'a>(
    sessions: &'a [CaptureSession],
    project_key: &str,
    path: &Path,
) -> Option<&'a CaptureSession> {
    sessions
        .iter()
        .filter(|s| session_allows_transcript(s, project_key, path))
        .min_by_key(|s| session_scope_rank(s))
}

fn unscoped_capture_allowed() -> bool {
    #[cfg(test)]
    {
        if crate::capture::test_contextlayer_isolated() {
            return true;
        }
    }
    std::env::var("CONTEXTLAYER_ALLOW_UNSCOPED_CAPTURE").is_ok()
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
) -> Result<u32, String> {
    let mut baselined = 0u32;
    for path in discover_all_transcript_files() {
        let Some(project_key) = project_key_from_transcript_path(&path) else {
            continue;
        };
        if let Some(cp) = cursor_project {
            if project_key != cp {
                continue;
            }
        }
        if let Some(tp) = transcript_path {
            if !transcript_paths_match(&path, tp) {
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

/// Returns an error if another active session already targets the same transcript file.
pub fn ensure_no_transcript_scope_conflict(
    sessions: &[CaptureSession],
    workspace_id: &str,
    transcript_path: Option<&str>,
) -> Result<(), String> {
    let Some(tp) = transcript_path else {
        return Ok(());
    };
    for s in sessions {
        if s.workspace_id == workspace_id {
            continue;
        }
        let Some(other) = s.transcript_path.as_deref() else {
            continue;
        };
        if transcript_paths_match(Path::new(tp), other) {
            return Err(format!(
                "transcript already captured on workspace `{}` — stop that session first",
                s.workspace_id
            ));
        }
    }
    Ok(())
}

fn latest_log_seq(workspace_id: &str, capture_branch: &str) -> Result<u64, String> {
    let capture = CaptureStore::default_open()?;
    let branch = if capture_branch == "main" {
        None
    } else {
        Some(capture_branch)
    };
    let messages = capture.read_log_messages_on_line(workspace_id, branch)?;
    Ok(messages.last().map(|m| m.seq).unwrap_or(0))
}

pub fn session_message_count(session: &CaptureSession) -> Result<u32, String> {
    let capture = CaptureStore::default_open()?;
    let branch = if session.capture_branch == "main" {
        None
    } else {
        Some(session.capture_branch.as_str())
    };
    let messages = capture.read_log_messages_on_line(&session.workspace_id, branch)?;
    Ok(messages
        .iter()
        .filter(|m| m.seq > session.log_seq_at_start)
        .count() as u32)
}

/// Start an opt-in capture session. Replaces any existing session for the same workspace.
pub fn start_capture_session(
    workspace_id: &str,
    cursor_project: Option<String>,
    transcript_path: Option<String>,
    label: Option<String>,
) -> Result<(CaptureSession, u32), String> {
    if cursor_project.is_none() && transcript_path.is_none() && !unscoped_capture_allowed() {
        return Err(
            "capture requires a scoped chat — pick a Cursor or Claude thread in the app"
                .to_string(),
        );
    }

    let mut store = load_capture_sessions()?;
    ensure_no_transcript_scope_conflict(&store.active, workspace_id, transcript_path.as_deref())?;

    let mut state = load_recorder_state()?;
    let baselined = baseline_transcripts_for_session(
        &mut state,
        workspace_id,
        "main",
        cursor_project.as_deref(),
        transcript_path.as_deref(),
    )?;
    save_recorder_state(&state)?;

    let log_seq_at_start = latest_log_seq(workspace_id, "main")?;

    let session = CaptureSession {
        id: Uuid::new_v4().to_string(),
        workspace_id: workspace_id.to_string(),
        started_at: Utc::now().to_rfc3339(),
        label: label.unwrap_or_default(),
        cursor_project,
        transcript_path,
        capture_branch: "main".to_string(),
        log_seq_at_start,
    };

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

    if let Ok(capture) = CaptureStore::default_open() {
        let log_seq_at_stop = capture
            .read_log_messages(workspace_id)
            .ok()
            .and_then(|msgs| msgs.last().map(|m| m.seq))
            .unwrap_or(removed.log_seq_at_start);
        let stopped_at = Utc::now().to_rfc3339();
        let _ = persist_last_capture_boundary(
            workspace_id,
            removed.log_seq_at_start,
            log_seq_at_stop,
            &removed.started_at,
            &stopped_at,
        );
    }

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

    if let Ok(capture) = CaptureStore::default_open() {
        let log_seq_at_stop = capture
            .read_log_messages(&removed.workspace_id)
            .ok()
            .and_then(|msgs| msgs.last().map(|m| m.seq))
            .unwrap_or(removed.log_seq_at_start);
        let stopped_at = Utc::now().to_rfc3339();
        let _ = persist_last_capture_boundary(
            &removed.workspace_id,
            removed.log_seq_at_start,
            log_seq_at_stop,
            &removed.started_at,
            &stopped_at,
        );
    }

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
            log_seq_at_start: 0,
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

    #[test]
    fn matching_session_prefers_scoped_over_unscoped() {
        let path = Path::new("C:\\proj\\agent-transcripts\\id\\id.jsonl");
        let unscoped = CaptureSession {
            id: "ghost".into(),
            workspace_id: "ws-branch-flow".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: None,
            transcript_path: None,
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        };
        let scoped = CaptureSession {
            id: "real".into(),
            workspace_id: "ws-real".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: Some("proj".into()),
            transcript_path: None,
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        };
        let sessions = vec![unscoped, scoped];
        let picked = matching_capture_session(&sessions, "proj", path).unwrap();
        assert_eq!(picked.workspace_id, "ws-real");
    }

    #[test]
    fn matching_session_prefers_transcript_path_over_project() {
        let dir = tempfile::tempdir().unwrap();
        let transcript = dir.path().join("chat.jsonl");
        std::fs::write(&transcript, "{}\n").unwrap();
        let project = CaptureSession {
            id: "proj-only".into(),
            workspace_id: "ws-project".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: Some("proj".into()),
            transcript_path: None,
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        };
        let pinned = CaptureSession {
            id: "pinned".into(),
            workspace_id: "ws-pinned".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: Some("proj".into()),
            transcript_path: Some(transcript.to_string_lossy().to_string()),
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        };
        let sessions = vec![project, pinned];
        let picked = matching_capture_session(&sessions, "proj", &transcript).unwrap();
        assert_eq!(picked.workspace_id, "ws-pinned");
    }

    #[test]
    fn prune_removes_unscoped_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let _guard = crate::capture::TestContextlayerGuard::new(dir.path().to_path_buf());
        let mut store = CaptureSessionStore::default();
        store.active.push(CaptureSession {
            id: "ghost".into(),
            workspace_id: "ws-branch-flow".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: None,
            transcript_path: None,
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        });
        store.active.push(CaptureSession {
            id: "real".into(),
            workspace_id: "ws-real".into(),
            started_at: "now".into(),
            label: String::new(),
            cursor_project: Some("proj".into()),
            transcript_path: Some("/tmp/chat.jsonl".into()),
            capture_branch: "main".into(),
            log_seq_at_start: 0,
        });
        save_capture_sessions(&store).unwrap();
        assert_eq!(prune_unscoped_capture_sessions().unwrap(), 1);
        let after = load_capture_sessions().unwrap();
        assert_eq!(after.active.len(), 1);
        assert_eq!(after.active[0].workspace_id, "ws-real");
    }
}
