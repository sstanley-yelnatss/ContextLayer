//! Session-scoped capture: auto-detect active transcripts and per-workspace prefs.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::transcripts::{
    discover_all_transcript_files, format_scope_label, project_key_from_transcript_path,
    TranscriptSource,
};
use crate::recording::{start_capture_session, CaptureSession};

/// How recently a transcript must have been touched to count as "active".
pub const DEFAULT_RECENT_SECS: u64 = 300;
/// If two candidates are this close in mtime, ask the user to pick.
pub const AMBIGUOUS_MTIME_SECS: u64 = 120;
/// Desktop picker: show chats touched within this window (7 days).
pub const PICKER_LOOKBACK_SECS: u64 = 7 * 24 * 3600;
pub const PICKER_CANDIDATE_LIMIT: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptCandidate {
    pub cursor_project: String,
    pub transcript_path: String,
    pub label: String,
    pub modified_secs_ago: u64,
    /// `cursor` or `claude`
    #[serde(default = "default_transcript_source")]
    pub source: String,
}

fn default_transcript_source() -> String {
    TranscriptSource::Cursor.as_str().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CaptureScopeResolution {
    Resolved {
        cursor_project: String,
        transcript_path: String,
    },
    NeedsPicker {
        candidates: Vec<TranscriptCandidate>,
    },
    NoCandidates {
        hint: String,
    },
}

/// Boundary of the most recently stopped capture session (for PR export slicing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastCaptureBoundary {
    pub log_seq_at_start: u64,
    pub log_seq_at_stop: u64,
    pub started_at: String,
    pub stopped_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceCapturePrefs {
    pub workspace_id: String,
    #[serde(default)]
    pub remembered_cursor_projects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_transcript_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_capture: Option<LastCaptureBoundary>,
}

/// Persist the stopped session boundary so PR export can slice "since last capture start".
pub fn persist_last_capture_boundary(
    workspace_id: &str,
    log_seq_at_start: u64,
    log_seq_at_stop: u64,
    started_at: &str,
    stopped_at: &str,
) -> Result<(), String> {
    let mut prefs = load_workspace_capture_prefs(workspace_id);
    prefs.workspace_id = workspace_id.to_string();
    prefs.last_capture = Some(LastCaptureBoundary {
        log_seq_at_start,
        log_seq_at_stop,
        started_at: started_at.to_string(),
        stopped_at: stopped_at.to_string(),
    });
    save_workspace_capture_prefs(&prefs)
}

/// Active capture session or last stopped session defines the export boundary.
pub fn resolve_capture_log_boundary(workspace_id: &str) -> Result<Option<u64>, String> {
    use crate::recording::{active_session_for_workspace, load_capture_sessions};

    let active = load_capture_sessions()?.active;
    if let Some(session) = active_session_for_workspace(&active, workspace_id) {
        return Ok(Some(session.log_seq_at_start));
    }
    Ok(load_workspace_capture_prefs(workspace_id)
        .last_capture
        .map(|c| c.log_seq_at_start))
}

pub fn capture_log_boundary_available(workspace_id: &str) -> Result<bool, String> {
    Ok(resolve_capture_log_boundary(workspace_id)?.is_some())
}

pub fn workspace_prefs_dir() -> PathBuf {
    crate::capture::contextlayer_dir().join("capture_prefs")
}

pub fn workspace_prefs_path(workspace_id: &str) -> PathBuf {
    workspace_prefs_dir().join(format!("{workspace_id}.json"))
}

pub fn load_workspace_capture_prefs(workspace_id: &str) -> WorkspaceCapturePrefs {
    let path = workspace_prefs_path(workspace_id);
    if !path.exists() {
        return WorkspaceCapturePrefs {
            workspace_id: workspace_id.to_string(),
            ..Default::default()
        };
    }
    let text = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&text).unwrap_or_else(|_| WorkspaceCapturePrefs {
        workspace_id: workspace_id.to_string(),
        ..Default::default()
    })
}

pub fn save_workspace_capture_prefs(prefs: &WorkspaceCapturePrefs) -> Result<(), String> {
    let dir = workspace_prefs_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    fs::write(workspace_prefs_path(&prefs.workspace_id), text).map_err(|e| e.to_string())
}

pub fn remember_scope_for_workspace(
    workspace_id: &str,
    cursor_project: &str,
    transcript_path: &str,
) -> Result<(), String> {
    let mut prefs = load_workspace_capture_prefs(workspace_id);
    prefs.workspace_id = workspace_id.to_string();
    prefs.preferred_transcript_path = Some(transcript_path.to_string());
    if !prefs
        .remembered_cursor_projects
        .iter()
        .any(|p| p == cursor_project)
    {
        prefs.remembered_cursor_projects.push(cursor_project.to_string());
    }
    save_workspace_capture_prefs(&prefs)
}

fn file_mtime_secs_ago(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    let now = SystemTime::now();
    now.duration_since(modified)
        .ok()
        .map(|d| d.as_secs())
}

fn candidate_label(project_key: &str, path: &Path) -> String {
    format_scope_label(project_key, path)
}

/// Public alias for desktop status + picker labels.
pub fn capture_scope_label(project_key: &str, transcript_path: &str) -> String {
    format_scope_label(project_key, Path::new(transcript_path))
}

pub fn transcript_paths_match(path: &Path, want: &str) -> bool {
    path.canonicalize()
        .ok()
        .zip(PathBuf::from(want).canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or_else(|| path.to_string_lossy() == want)
}

/// List transcript files touched within `max_age_secs`, newest first.
pub fn list_transcript_candidates(max_age_secs: u64) -> Vec<TranscriptCandidate> {
    let mut out = Vec::new();
    for path in discover_all_transcript_files() {
        let Some(age) = file_mtime_secs_ago(&path) else {
            continue;
        };
        if age > max_age_secs {
            continue;
        }
        let Some(project) = project_key_from_transcript_path(&path) else {
            continue;
        };
        let source = crate::transcripts::transcript_source(&path)
            .map(|s| s.as_str().to_string())
            .unwrap_or_else(|| default_transcript_source());
        out.push(TranscriptCandidate {
            cursor_project: project.clone(),
            transcript_path: path.to_string_lossy().to_string(),
            label: candidate_label(&project, &path),
            modified_secs_ago: age,
            source,
        });
    }
    out.sort_by_key(|c| c.modified_secs_ago);
    out
}

/// Chats for the desktop picker (wide lookback, capped).
pub fn list_picker_candidates() -> Vec<TranscriptCandidate> {
    list_transcript_candidates(PICKER_LOOKBACK_SECS)
        .into_iter()
        .take(PICKER_CANDIDATE_LIMIT)
        .collect()
}

pub fn detect_capture_scope(workspace_id: &str) -> CaptureScopeResolution {
    let prefs = load_workspace_capture_prefs(workspace_id);

    if let Some(ref preferred) = prefs.preferred_transcript_path {
        let path = PathBuf::from(preferred);
        if path.is_file() {
            if let Some(project) = project_key_from_transcript_path(&path) {
                return CaptureScopeResolution::Resolved {
                    cursor_project: project,
                    transcript_path: preferred.clone(),
                };
            }
        }
    }

    let mut candidates = list_transcript_candidates(DEFAULT_RECENT_SECS);
    if !prefs.remembered_cursor_projects.is_empty() {
        candidates.retain(|c| {
            prefs
                .remembered_cursor_projects
                .iter()
                .any(|p| p == &c.cursor_project)
        });
    }

    if candidates.is_empty() {
        candidates = list_transcript_candidates(DEFAULT_RECENT_SECS * 6);
        if candidates.is_empty() {
            return CaptureScopeResolution::NoCandidates {
                hint: "No recent chat transcripts found. Send a message in Cursor or Claude Code, then try again."
                    .to_string(),
            };
        }
        let take = candidates.len().min(8);
        return CaptureScopeResolution::NeedsPicker {
            candidates: candidates[..take].to_vec(),
        };
    }

    if candidates.len() == 1 {
        let c = &candidates[0];
        return CaptureScopeResolution::Resolved {
            cursor_project: c.cursor_project.clone(),
            transcript_path: c.transcript_path.clone(),
        };
    }

    let top = &candidates[0];
    let second = &candidates[1];
    if second.modified_secs_ago.saturating_sub(top.modified_secs_ago) >= AMBIGUOUS_MTIME_SECS {
        return CaptureScopeResolution::Resolved {
            cursor_project: top.cursor_project.clone(),
            transcript_path: top.transcript_path.clone(),
        };
    }

    CaptureScopeResolution::NeedsPicker {
        candidates: candidates.into_iter().take(8).collect(),
    }
}

pub fn resolve_capture_scope(
    workspace_id: &str,
    cursor_project: Option<String>,
    transcript_path: Option<String>,
) -> CaptureScopeResolution {
    match (cursor_project, transcript_path) {
        (Some(cp), Some(tp)) => CaptureScopeResolution::Resolved {
            cursor_project: cp,
            transcript_path: tp,
        },
        (Some(cp), None) => {
            let recent = list_transcript_candidates(DEFAULT_RECENT_SECS)
                .into_iter()
                .find(|c| c.cursor_project == cp);
            if let Some(c) = recent {
                CaptureScopeResolution::Resolved {
                    cursor_project: cp,
                    transcript_path: c.transcript_path,
                }
            } else {
                CaptureScopeResolution::NeedsPicker {
                    candidates: list_transcript_candidates(DEFAULT_RECENT_SECS)
                        .into_iter()
                        .filter(|c| c.cursor_project == cp)
                        .take(8)
                        .collect(),
                }
            }
        }
        (None, Some(tp)) => {
            let path = PathBuf::from(&tp);
            let project = project_key_from_transcript_path(&path).unwrap_or_default();
            CaptureScopeResolution::Resolved {
                cursor_project: project,
                transcript_path: tp,
            }
        }
        (None, None) => detect_capture_scope(workspace_id),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartCaptureOutcome {
    pub session: CaptureSession,
    pub baselined: u32,
    pub scope_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CaptureStartResult {
    Started(StartCaptureOutcome),
    NeedsPicker {
        candidates: Vec<TranscriptCandidate>,
    },
    NoCandidates {
        hint: String,
    },
}

pub fn begin_capture_session(
    workspace_id: &str,
    cursor_project: Option<String>,
    transcript_path: Option<String>,
    label: Option<String>,
    remember_scope: bool,
) -> Result<CaptureStartResult, String> {
    match resolve_capture_scope(workspace_id, cursor_project, transcript_path) {
        CaptureScopeResolution::Resolved {
            cursor_project,
            transcript_path,
        } => {
            if remember_scope {
                remember_scope_for_workspace(workspace_id, &cursor_project, &transcript_path)?;
            }
            let (session, baselined) = start_capture_session(
                workspace_id,
                Some(cursor_project.clone()),
                Some(transcript_path.clone()),
                label,
            )?;
            let scope_label = candidate_label(&cursor_project, Path::new(&transcript_path));
            Ok(CaptureStartResult::Started(StartCaptureOutcome {
                session,
                baselined,
                scope_label,
            }))
        }
        CaptureScopeResolution::NeedsPicker { candidates } => {
            Ok(CaptureStartResult::NeedsPicker { candidates })
        }
        CaptureScopeResolution::NoCandidates { hint } => {
            Ok(CaptureStartResult::NoCandidates { hint })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_explicit_scope() {
        let r = resolve_capture_scope(
            "ws-1",
            Some("proj-a".into()),
            Some("/tmp/chat.jsonl".into()),
        );
        match r {
            CaptureScopeResolution::Resolved { cursor_project, .. } => {
                assert_eq!(cursor_project, "proj-a");
            }
            _ => panic!("expected resolved"),
        }
    }
}
