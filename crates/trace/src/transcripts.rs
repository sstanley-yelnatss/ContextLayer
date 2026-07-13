//! Unified Cursor + Claude Code transcript discovery, labels, and polling.

use std::path::{Path, PathBuf};

use crate::capture::{CaptureStore, RecorderState};
use crate::claude_code::{
    claude_project_key_from_transcript_path, claude_projects_root, discover_claude_transcript_files,
    format_claude_scope_label, poll_claude_transcripts,
};
use crate::cursor::{
    cursor_project_key_from_transcript_path, cursor_projects_root, discover_transcript_files,
    format_transcript_scope_label, poll_cursor_transcripts, IngestStats, ParsedTranscriptLine,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscriptSource {
    Cursor,
    Claude,
}

impl TranscriptSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cursor => "cursor",
            Self::Claude => "claude",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cursor" => Some(Self::Cursor),
            "claude" => Some(Self::Claude),
            _ => None,
        }
    }
}

pub fn transcript_source(path: &Path) -> Option<TranscriptSource> {
    if claude_project_key_from_transcript_path(path).is_some() {
        return Some(TranscriptSource::Claude);
    }
    if cursor_project_key_from_transcript_path(path).is_some() {
        return Some(TranscriptSource::Cursor);
    }
    None
}

pub fn discover_all_transcript_files() -> Vec<PathBuf> {
    let mut out = discover_transcript_files(&cursor_projects_root());
    out.extend(discover_claude_transcript_files(&claude_projects_root()));
    out
}

pub fn project_key_from_transcript_path(path: &Path) -> Option<String> {
    cursor_project_key_from_transcript_path(path)
        .or_else(|| claude_project_key_from_transcript_path(path))
}

pub fn format_scope_label(project_key: &str, path: &Path) -> String {
    match transcript_source(path) {
        Some(TranscriptSource::Claude) => format_claude_scope_label(project_key, path),
        _ => format_transcript_scope_label(project_key, path),
    }
}

pub fn parse_transcript_line_any(line: &str) -> Option<ParsedTranscriptLine> {
    crate::cursor::parse_transcript_line(line)
        .or_else(|| crate::claude_code::parse_claude_transcript_line(line))
}

/// Poll Cursor agent transcripts and Claude Code session JSONL into active capture sessions.
pub fn poll_all_transcripts(
    capture: &CaptureStore,
    state: &mut RecorderState,
) -> Result<IngestStats, String> {
    let cursor = poll_cursor_transcripts(capture, state)?;
    let claude = poll_claude_transcripts(capture, state)?;
    Ok(IngestStats {
        files_scanned: cursor.files_scanned + claude.files_scanned,
        messages_appended: cursor.messages_appended + claude.messages_appended,
        files_skipped_unbound: cursor.files_skipped_unbound + claude.files_skipped_unbound,
        files_skipped_gated: cursor.files_skipped_gated + claude.files_skipped_gated,
    })
}
