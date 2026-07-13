//! Parse Cursor agent-transcript JSONL and map projects → workspaces.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use crate::capture::{CaptureStore, ProjectBindings, RecorderFileState, RecorderState};
use crate::recording::{load_capture_sessions, recorder_state_key, session_allows_transcript};

#[derive(Debug, Clone)]
pub struct ParsedTranscriptLine {
    pub line_index: u64,
    pub role: String,
    pub content: String,
}

/// Extract user-visible text from a Cursor agent-transcript JSONL line.
pub fn parse_transcript_line(line: &str) -> Option<ParsedTranscriptLine> {
    let v: Value = serde_json::from_str(line.trim()).ok()?;
    let role = v.get("role")?.as_str()?.to_string();
    let content = extract_message_text(v.get("message")?)?;
    if content.trim().is_empty() {
        return None;
    }
    Some(ParsedTranscriptLine {
        line_index: 0,
        role,
        content,
    })
}

fn extract_message_text(message: &Value) -> Option<String> {
    let content = message.get("content")?;
    let arr = content.as_array()?;
    let mut parts = Vec::new();
    for item in arr {
        if item.get("type")?.as_str()? != "text" {
            continue;
        }
        if let Some(t) = item.get("text").and_then(|x| x.as_str()) {
            parts.push(t.to_string());
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("\n"))
}

pub fn cursor_projects_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cursor")
        .join("projects")
}

pub fn cursor_global_state_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("AppData")
        .join("Roaming")
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb")
}

/// Composer/chat folder id from a transcript path (UUID folder under agent-transcripts).
pub fn transcript_composer_id(path: &Path) -> Option<String> {
    path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}

#[derive(Debug, Deserialize)]
struct ComposerHeadersBlob {
    #[serde(default, rename = "allComposers")]
    all_composers: Vec<ComposerHeaderEntry>,
}

#[derive(Debug, Deserialize)]
struct ComposerHeaderEntry {
    #[serde(rename = "composerId")]
    composer_id: String,
    #[serde(default)]
    name: Option<String>,
}

fn parse_composer_titles_json(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(blob) = serde_json::from_str::<ComposerHeadersBlob>(text) else {
        return out;
    };
    for entry in blob.all_composers {
        if let Some(name) = entry.name {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                out.insert(entry.composer_id, trimmed.to_string());
            }
        }
    }
    out
}

fn read_composer_titles_from_db() -> Result<HashMap<String, String>, String> {
    let path = cursor_global_state_db_path();
    if !path.is_file() {
        return Ok(HashMap::new());
    }
    let conn = rusqlite::Connection::open_with_flags(
        &path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| e.to_string())?;
    let text: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            ["composer.composerHeaders"],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(parse_composer_titles_json(&text))
}

struct ComposerTitleCache {
    loaded_at: Instant,
    titles: HashMap<String, String>,
}

static COMPOSER_TITLE_CACHE: Mutex<Option<ComposerTitleCache>> = Mutex::new(None);
const COMPOSER_TITLE_CACHE_TTL: Duration = Duration::from_secs(30);

/// Cursor UI chat titles keyed by composer id (from global state.vscdb).
pub fn load_composer_titles() -> HashMap<String, String> {
    let now = Instant::now();
    if let Ok(mut guard) = COMPOSER_TITLE_CACHE.lock() {
        if let Some(cache) = guard.as_ref() {
            if now.duration_since(cache.loaded_at) < COMPOSER_TITLE_CACHE_TTL {
                return cache.titles.clone();
            }
        }
        let titles = read_composer_titles_from_db().unwrap_or_default();
        *guard = Some(ComposerTitleCache {
            loaded_at: now,
            titles: titles.clone(),
        });
        return titles;
    }
    read_composer_titles_from_db().unwrap_or_default()
}

/// Human-readable Cursor chat title for a transcript, if Cursor has named it.
pub fn composer_chat_title(path: &Path) -> Option<String> {
    let id = transcript_composer_id(path)?;
    load_composer_titles().get(&id).cloned()
}

/// Label for capture UI: Cursor chat name when available, else project / composer id.
pub fn format_transcript_scope_label(project_key: &str, path: &Path) -> String {
    if let Some(title) = composer_chat_title(path) {
        return title;
    }
    let chat = transcript_composer_id(path).unwrap_or_else(|| "chat".to_string());
    format!("{project_key} / {chat}")
}

/// Sanitized Cursor project folder name from absolute repo path (best-effort).
pub fn sanitize_project_key(repo_path: &str) -> String {
    let mut s = repo_path.to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        s = stripped.to_string();
    }
    s.replace('\\', "-")
        .replace('/', "-")
        .replace(':', "")
        .trim_matches('-')
        .to_string()
}

pub fn resolve_workspace_for_cursor_project(
    bindings: &ProjectBindings,
    cursor_project_key: &str,
) -> Option<String> {
    bindings
        .cursor_projects
        .get(cursor_project_key)
        .cloned()
}

/// Discover main agent transcript jsonl files (exclude subagents/).
pub fn discover_transcript_files(projects_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !projects_root.is_dir() {
        return out;
    }
    let Ok(entries) = fs::read_dir(projects_root) else {
        return out;
    };
    for project_entry in entries.flatten() {
        let transcripts = project_entry.path().join("agent-transcripts");
        if !transcripts.is_dir() {
            continue;
        }
        collect_jsonl_files(&transcripts, &mut out, false);
    }
    out
}

fn collect_jsonl_files(dir: &Path, out: &mut Vec<PathBuf>, in_subagents: bool) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "subagents" {
                collect_jsonl_files(&path, out, true);
            } else {
                collect_jsonl_files(&path, out, in_subagents);
            }
            continue;
        }
        if in_subagents {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
}

pub fn cursor_project_key_from_transcript_path(path: &Path) -> Option<String> {
    let projects = cursor_projects_root();
    // Windows canonicalize() uses \\?\ extended paths; strip_prefix requires matching roots.
    let projects = projects.canonicalize().unwrap_or(projects);
    let path = path.canonicalize().ok()?;
    let rel = path.strip_prefix(&projects).ok()?;
    rel.components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .map(|s| s.to_string())
}

/// Tail a transcript file from byte offset; returns (new_offset, parsed lines with indices).
pub fn read_transcript_delta(
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
    let base_line = if start == 0 {
        0
    } else {
        // approximate — dedupe uses source_ref with absolute line index from file start
        0
    };
    for (i, line) in buf.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(mut parsed) = parse_transcript_line(line) {
            parsed.line_index = base_line + i as u64 + 1;
            lines.push(parsed);
        }
    }
    Ok((new_offset, lines))
}

#[derive(Debug, Clone, Default)]
pub struct IngestStats {
    pub files_scanned: u32,
    pub messages_appended: u32,
    pub files_skipped_unbound: u32,
    /// No active capture session for this workspace / scope.
    pub files_skipped_gated: u32,
}

/// One poll pass: tail scoped Cursor transcript files into active capture sessions.
pub fn poll_cursor_transcripts(
    capture: &CaptureStore,
    state: &mut RecorderState,
) -> Result<IngestStats, String> {
    let mut stats = IngestStats::default();
    let sessions = load_capture_sessions()?.active;
    if sessions.is_empty() {
        return Ok(stats);
    }
    let files = discover_transcript_files(&cursor_projects_root());
    for path in files {
        stats.files_scanned += 1;
        let Some(project_key) = cursor_project_key_from_transcript_path(&path) else {
            continue;
        };

        let mut matched = false;
        for session in &sessions {
            if !session_allows_transcript(session, &project_key, &path) {
                continue;
            }
            matched = true;
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
            let (new_offset, lines) = read_transcript_delta(&path, offset)?;
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
                let source_ref = format!("{}:{}", path_key, line_counter);
                match capture.append_message_on_line(
                    &workspace_id,
                    branch_slug,
                    &line.role,
                    &line.content,
                    "cursor_transcript",
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
                    composer_id: path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string()),
                    lines_ingested: line_counter,
                },
            );
            break;
        }
        if !matched {
            stats.files_skipped_gated += 1;
        }
    }
    Ok(stats)
}

/// Import an entire agent-transcript JSONL file into a workspace log (onboarding).
pub fn import_transcript_file(
    capture: &CaptureStore,
    workspace_id: &str,
    path: &Path,
) -> Result<u32, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    import_transcript_text(capture, workspace_id, &text, "import", Some(&path.to_string_lossy()))
}

pub fn import_session_log(
    capture: &CaptureStore,
    workspace_id: &str,
    text: &str,
) -> Result<u32, String> {
    let jsonl_count = import_transcript_text(capture, workspace_id, text, "import", None)?;
    if jsonl_count > 0 {
        return Ok(jsonl_count);
    }
    import_paste_transcript(capture, workspace_id, text)
}

fn import_paste_transcript(
    capture: &CaptureStore,
    workspace_id: &str,
    text: &str,
) -> Result<u32, String> {
    let mut count = 0u32;
    let mut current_role = "user";
    let mut buf = String::new();
    let flush = |capture: &CaptureStore, role: &str, buf: &str, count: &mut u32| -> Result<(), String> {
        let t = buf.trim();
        if t.len() < 3 {
            return Ok(());
        }
        capture.append_message(workspace_id, role, t, "import", None)?;
        *count += 1;
        Ok(())
    };

    for line in text.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_ascii_lowercase();
        if lower.starts_with("user:") || lower.starts_with("human:") {
            flush(capture, current_role, &buf, &mut count)?;
            buf.clear();
            current_role = "user";
            buf.push_str(trimmed.split_once(':').map(|(_, r)| r).unwrap_or("").trim());
            buf.push('\n');
            continue;
        }
        if lower.starts_with("assistant:") || lower.starts_with("cursor:") {
            flush(capture, current_role, &buf, &mut count)?;
            buf.clear();
            current_role = "assistant";
            buf.push_str(trimmed.split_once(':').map(|(_, r)| r).unwrap_or("").trim());
            buf.push('\n');
            continue;
        }
        buf.push_str(line);
        buf.push('\n');
    }
    flush(capture, current_role, &buf, &mut count)?;
    Ok(count)
}

pub fn import_transcript_text(
    capture: &CaptureStore,
    workspace_id: &str,
    text: &str,
    source: &str,
    source_path: Option<&str>,
) -> Result<u32, String> {
    let mut count = 0u32;
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let Some(mut parsed) = crate::transcripts::parse_transcript_line_any(line) else {
            continue;
        };
        parsed.line_index = i as u64 + 1;
        let source_ref = source_path.map(|p| format!("{p}:{}", parsed.line_index));
        match capture.append_message(
            workspace_id,
            &parsed.role,
            &parsed.content,
            source,
            source_ref,
        ) {
            Ok(_) => count += 1,
            Err(e) if e == "duplicate source_ref" => {}
            Err(e) => return Err(e),
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_line() {
        let line = r#"{"role":"user","message":{"content":[{"type":"text","text":"hello world"}]}}"#;
        let p = parse_transcript_line(line).unwrap();
        assert_eq!(p.role, "user");
        assert!(p.content.contains("hello"));
    }

    #[test]
    fn parses_composer_titles_json() {
        let json = r#"{"allComposers":[{"composerId":"abc-123","name":"Cross-AI context continuity system PRD review"}]}"#;
        let map = parse_composer_titles_json(json);
        assert_eq!(
            map.get("abc-123").map(String::as_str),
            Some("Cross-AI context continuity system PRD review")
        );
    }

    #[test]
    fn sanitize_project_key_strips_windows_extended_prefix() {
        let key = sanitize_project_key(r"\\?\C:\Users\miles\ContextLayer");
        assert_eq!(key, "C-Users-miles-ContextLayer");
    }

    #[test]
    fn cursor_project_key_from_transcript_path_windows() {
        let root = cursor_projects_root();
        if !root.is_dir() {
            return;
        }
        let files = discover_transcript_files(&root);
        let Some(path) = files.into_iter().next() else {
            return;
        };
        let key = cursor_project_key_from_transcript_path(&path);
        assert!(
            key.is_some(),
            "expected project key for {}",
            path.display()
        );
    }
}
