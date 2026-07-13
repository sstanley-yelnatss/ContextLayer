//! Parse Claude Code CLI session JSONL (`~/.claude/projects/.../*.jsonl`).

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use crate::capture::{CaptureStore, RecorderFileState, RecorderState};
use crate::cursor::{IngestStats, ParsedTranscriptLine};
use crate::recording::{load_capture_sessions, matching_capture_session, recorder_state_key};

pub fn claude_projects_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("projects")
}

pub fn claude_sessions_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("sessions")
}

/// Session UUID from `.../<uuid>.jsonl`.
pub fn claude_session_id_from_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

pub fn claude_project_key_from_transcript_path(path: &Path) -> Option<String> {
    let projects = claude_projects_root();
    let projects = projects.canonicalize().unwrap_or(projects);
    let path = path.canonicalize().ok()?;
    let rel = path.strip_prefix(&projects).ok()?;
    rel.components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string())
}

/// Discover session JSONL files directly under each project folder (not `memory/`).
pub fn discover_claude_transcript_files(projects_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !projects_root.is_dir() {
        return out;
    }
    let Ok(entries) = fs::read_dir(projects_root) else {
        return out;
    };
    for project_entry in entries.flatten() {
        let project_dir = project_entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        let Ok(files) = fs::read_dir(&project_dir) else {
            continue;
        };
        for file_entry in files.flatten() {
            let path = file_entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                out.push(path);
            }
        }
    }
    out
}

fn extract_claude_text_content(content: &Value) -> Option<String> {
    if let Some(text) = content.as_str() {
        let t = text.trim();
        if t.is_empty() {
            return None;
        }
        return Some(text.to_string());
    }
    let arr = content.as_array()?;
    let mut parts = Vec::new();
    for item in arr {
        if item.get("type")?.as_str()? != "text" {
            continue;
        }
        if let Some(t) = item.get("text").and_then(|x| x.as_str()) {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                parts.push(t.to_string());
            }
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("\n"))
}

/// Extract user/assistant chat from a Claude Code JSONL line.
pub fn parse_claude_transcript_line(line: &str) -> Option<ParsedTranscriptLine> {
    let v: Value = serde_json::from_str(line.trim()).ok()?;
    let typ = v.get("type")?.as_str()?;
    let message = v.get("message")?;
    let role = match typ {
        "user" => "user",
        "assistant" => "assistant",
        _ => return None,
    };
    let content = extract_claude_text_content(message.get("content")?)?;
    Some(ParsedTranscriptLine {
        line_index: 0,
        role: role.to_string(),
        content,
    })
}

#[derive(Debug, Deserialize)]
struct ClaudeSessionMeta {
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(default)]
    name: Option<String>,
}

fn load_claude_session_names() -> HashMap<String, String> {
    let mut out = HashMap::new();
    let root = claude_sessions_root();
    if !root.is_dir() {
        return out;
    }
    let Ok(entries) = fs::read_dir(&root) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<ClaudeSessionMeta>(&text) else {
            continue;
        };
        if let Some(name) = meta.name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                out.insert(meta.session_id, trimmed.to_string());
            }
        }
    }
    out
}

fn read_ai_title_from_jsonl(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let mut last = None;
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v.get("type").and_then(|t| t.as_str()) == Some("ai-title") {
            if let Some(title) = v.get("aiTitle").and_then(|t| t.as_str()) {
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    last = Some(trimmed.to_string());
                }
            }
        }
    }
    last
}

struct ClaudeTitleCache {
    loaded_at: Instant,
    session_names: HashMap<String, String>,
}

static CLAUDE_TITLE_CACHE: Mutex<Option<ClaudeTitleCache>> = Mutex::new(None);
const CLAUDE_TITLE_CACHE_TTL: Duration = Duration::from_secs(30);

pub fn claude_chat_title(path: &Path) -> Option<String> {
    if let Some(title) = read_ai_title_from_jsonl(path) {
        return Some(title);
    }
    let session_id = claude_session_id_from_path(path)?;
    let now = Instant::now();
    if let Ok(mut guard) = CLAUDE_TITLE_CACHE.lock() {
        if let Some(cache) = guard.as_ref() {
            if now.duration_since(cache.loaded_at) < CLAUDE_TITLE_CACHE_TTL {
                return cache.session_names.get(&session_id).cloned();
            }
        }
        let session_names = load_claude_session_names();
        let title = session_names.get(&session_id).cloned();
        *guard = Some(ClaudeTitleCache {
            loaded_at: now,
            session_names,
        });
        return title;
    }
    load_claude_session_names().get(&session_id).cloned()
}

pub fn format_claude_scope_label(project_key: &str, path: &Path) -> String {
    if let Some(title) = claude_chat_title(path) {
        return title;
    }
    let session = claude_session_id_from_path(path).unwrap_or_else(|| "session".to_string());
    let short = session.chars().take(8).collect::<String>();
    format!("Claude · {project_key} / {short}")
}

pub fn read_claude_transcript_delta(
    path: &Path,
    byte_offset: u64,
) -> Result<(u64, Vec<ParsedTranscriptLine>), String> {
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let len = file.metadata().map_err(|e| e.to_string())?.len();
    let start = byte_offset.min(len);
    file.seek(SeekFrom::Start(start))
        .map_err(|e| e.to_string())?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
    let new_offset = len;
    let mut lines = Vec::new();
    for (i, line) in buf.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(mut parsed) = parse_claude_transcript_line(line) {
            parsed.line_index = i as u64 + 1;
            lines.push(parsed);
        }
    }
    Ok((new_offset, lines))
}

/// Tail Claude Code session files into active capture sessions.
pub fn poll_claude_transcripts(
    capture: &CaptureStore,
    state: &mut RecorderState,
) -> Result<IngestStats, String> {
    let mut stats = IngestStats::default();
    let sessions = load_capture_sessions()?.active;
    if sessions.is_empty() {
        return Ok(stats);
    }
    let files = discover_claude_transcript_files(&claude_projects_root());
    for path in files {
        stats.files_scanned += 1;
        let Some(project_key) = claude_project_key_from_transcript_path(&path) else {
            continue;
        };

        let Some(session) = matching_capture_session(&sessions, &project_key, &path) else {
            stats.files_skipped_gated += 1;
            continue;
        };
        let workspace_id = session.workspace_id.clone();
        let capture_branch = session.capture_branch.clone();
        let branch_slug = if capture_branch == "main" {
            None
        } else {
            Some(capture_branch.as_str())
        };
        let path_key = path.to_string_lossy().to_string();
        let state_key = recorder_state_key(&path_key, &workspace_id, &capture_branch);
        let legacy = state.files.get(&path_key);
        let offset = state
            .files
            .get(&state_key)
            .map(|s| s.byte_offset)
            .or_else(|| {
                if capture_branch == "main" {
                    legacy.map(|s| s.byte_offset)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        let (new_offset, lines) = read_claude_transcript_delta(&path, offset)?;
        let mut line_counter = state
            .files
            .get(&state_key)
            .map(|s| s.lines_ingested)
            .or_else(|| {
                if capture_branch == "main" {
                    legacy.map(|s| s.lines_ingested)
                } else {
                    None
                }
            })
            .unwrap_or(0);
        for line in lines {
            line_counter += 1;
            let source_ref = format!("{path_key}:{}", line_counter);
            match capture.append_message_on_line(
                &workspace_id,
                branch_slug,
                &line.role,
                &line.content,
                "claude_transcript",
                Some(source_ref),
            ) {
                Ok(_) => stats.messages_appended += 1,
                Err(e) if e == "duplicate source_ref" => {}
                Err(e) => return Err(e),
            }
        }
        state.files.insert(
            state_key,
            RecorderFileState {
                byte_offset: new_offset,
                workspace_id: workspace_id.clone(),
                capture_branch,
                composer_id: claude_session_id_from_path(&path),
                lines_ingested: line_counter,
            },
        );
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_claude_user_line() {
        let line = r#"{"type":"user","message":{"role":"user","content":"hello claude"},"sessionId":"abc"}"#;
        let p = parse_claude_transcript_line(line).unwrap();
        assert_eq!(p.role, "user");
        assert_eq!(p.content, "hello claude");
    }

    #[test]
    fn parses_claude_assistant_text_array() {
        let line = r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hi there"}]}}"#;
        let p = parse_claude_transcript_line(line).unwrap();
        assert_eq!(p.role, "assistant");
        assert!(p.content.contains("Hi there"));
    }

    #[test]
    fn skips_claude_attachment_lines() {
        let line = r#"{"type":"attachment","attachment":{"type":"skill_listing"}}"#;
        assert!(parse_claude_transcript_line(line).is_none());
    }

    #[test]
    fn claude_project_key_encoding() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("projects");
        let project = root.join("C--Users-miles-ContextLayer");
        std::fs::create_dir_all(&project).unwrap();
        let path = project.join("sess.jsonl");
        std::fs::write(&path, "{}\n").unwrap();
        // Patch: test via strip logic with canonical paths under temp root
        let projects = root.canonicalize().unwrap();
        let path = path.canonicalize().unwrap();
        let rel = path.strip_prefix(&projects).unwrap();
        let key = rel
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
            .map(|s| s.to_string());
        assert_eq!(key.as_deref(), Some("C--Users-miles-ContextLayer"));
    }
}
