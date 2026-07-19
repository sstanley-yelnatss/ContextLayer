//! Parse Cursor agent-transcript JSONL and map projects → workspaces.

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use crate::capture::{CaptureStore, ProjectBindings, RecorderFileState, RecorderState};
use crate::recording::{load_capture_sessions, matching_capture_session, recorder_state_key};

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
    #[serde(default)]
    subtitle: Option<String>,
}

/// Fields Cursor stores on a composer (table `value` JSON or ItemTable blob entry).
#[derive(Debug, Deserialize)]
struct ComposerValueFields {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    subtitle: Option<String>,
}

const LABEL_MAX_CHARS: usize = 72;

fn truncate_label(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    if let Some(i) = out.rfind(' ') {
        if i > keep / 2 {
            out.truncate(i);
        }
    }
    out.push('…');
    out
}

fn label_from_name_subtitle(name: Option<&str>, subtitle: Option<&str>) -> Option<String> {
    if let Some(n) = name.map(str::trim).filter(|s| !s.is_empty()) {
        return Some(n.to_string());
    }
    let s = subtitle.map(str::trim).filter(|s| !s.is_empty())?;
    Some(truncate_label(s, LABEL_MAX_CHARS))
}

fn insert_composer_label(out: &mut HashMap<String, String>, id: String, name: Option<&str>, subtitle: Option<&str>) {
    if out.contains_key(&id) {
        return;
    }
    if let Some(label) = label_from_name_subtitle(name, subtitle) {
        out.insert(id, label);
    }
}

fn parse_composer_titles_json(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let Ok(blob) = serde_json::from_str::<ComposerHeadersBlob>(text) else {
        return out;
    };
    for entry in blob.all_composers {
        insert_composer_label(
            &mut out,
            entry.composer_id,
            entry.name.as_deref(),
            entry.subtitle.as_deref(),
        );
    }
    out
}

fn load_titles_from_headers_table(
    conn: &rusqlite::Connection,
    out: &mut HashMap<String, String>,
) -> bool {
    let Ok(mut stmt) = conn.prepare("SELECT composerId, value FROM composerHeaders") else {
        return false;
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }) else {
        return false;
    };
    let mut any = false;
    for row in rows.flatten() {
        any = true;
        let (id, value) = row;
        let Ok(fields) = serde_json::from_str::<ComposerValueFields>(&value) else {
            continue;
        };
        insert_composer_label(&mut *out, id, fields.name.as_deref(), fields.subtitle.as_deref());
    }
    any
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

    let mut out = HashMap::new();
    // Cursor now prefers the composerHeaders table (tableGateEnabled); JSON blob can lag.
    let _ = load_titles_from_headers_table(&conn, &mut out);
    if let Ok(text) = conn.query_row(
        "SELECT value FROM ItemTable WHERE key = ?1",
        ["composer.composerHeaders"],
        |row| row.get::<_, String>(0),
    ) {
        for (id, label) in parse_composer_titles_json(&text) {
            out.entry(id).or_insert(label);
        }
    }
    Ok(out)
}

/// First user message preview from a transcript (untitled chats only).
fn first_user_line_preview(path: &Path) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let mut buf = String::new();
    let mut limited = file.take(12_288);
    limited.read_to_string(&mut buf).ok()?;
    for line in buf.lines().take(40) {
        if line.trim().is_empty() {
            continue;
        }
        let Some(parsed) = parse_transcript_line(line) else {
            continue;
        };
        if parsed.role != "user" {
            continue;
        }
        let text = parsed.content.lines().find(|l| !l.trim().is_empty())?.trim();
        if text.is_empty() {
            continue;
        }
        return Some(truncate_label(text, LABEL_MAX_CHARS));
    }
    None
}

fn short_composer_id(id: &str) -> &str {
    let end = id.char_indices().nth(8).map(|(i, _)| i).unwrap_or(id.len());
    &id[..end]
}

struct ComposerTitleCache {
    loaded_at: Instant,
    titles: Arc<HashMap<String, String>>,
}

static COMPOSER_TITLE_CACHE: Mutex<Option<ComposerTitleCache>> = Mutex::new(None);
const COMPOSER_TITLE_CACHE_TTL: Duration = Duration::from_secs(30);

/// Cursor UI chat titles keyed by composer id (from global state.vscdb).
pub fn load_composer_titles() -> Arc<HashMap<String, String>> {
    let now = Instant::now();
    if let Ok(mut guard) = COMPOSER_TITLE_CACHE.lock() {
        if let Some(cache) = guard.as_ref() {
            if now.duration_since(cache.loaded_at) < COMPOSER_TITLE_CACHE_TTL {
                return Arc::clone(&cache.titles);
            }
        }
        let titles = Arc::new(read_composer_titles_from_db().unwrap_or_default());
        *guard = Some(ComposerTitleCache {
            loaded_at: now,
            titles: Arc::clone(&titles),
        });
        return titles;
    }
    Arc::new(read_composer_titles_from_db().unwrap_or_default())
}

/// Human-readable Cursor chat title for a transcript, if one can be resolved.
pub fn composer_chat_title(path: &Path) -> Option<String> {
    let id = transcript_composer_id(path)?;
    if let Some(title) = load_composer_titles().get(&id) {
        return Some(title.clone());
    }
    first_user_line_preview(path)
}

/// Label for capture UI: chat name / subtitle / first user line, else project / short id.
pub fn format_transcript_scope_label(project_key: &str, path: &Path) -> String {
    if let Some(title) = composer_chat_title(path) {
        return title;
    }
    let chat = transcript_composer_id(path)
        .map(|id| short_composer_id(&id).to_string())
        .unwrap_or_else(|| "chat".to_string());
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
    fn composer_titles_json_falls_back_to_subtitle() {
        let json = r#"{"allComposers":[{"composerId":"x","subtitle":"Edited page.tsx and HelpPage.tsx for capture"}]}"#;
        let map = parse_composer_titles_json(json);
        assert_eq!(
            map.get("x").map(String::as_str),
            Some("Edited page.tsx and HelpPage.tsx for capture")
        );
    }

    #[test]
    fn label_prefers_name_over_subtitle() {
        assert_eq!(
            label_from_name_subtitle(Some("My chat"), Some("Edited foo.rs")).as_deref(),
            Some("My chat")
        );
    }

    #[test]
    fn short_composer_id_truncates() {
        assert_eq!(short_composer_id("4f447f43-05ab-478d-bd14-6678d57c21c7"), "4f447f43");
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
