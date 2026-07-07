//! ContextLayer capture store — append-only session log + decision commits (seq-anchored).

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::redact::{contains_secrets, redact_secrets};

pub fn default_capture_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".contextlayer")
        .join("capture")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMeta {
    pub workspace_id: String,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub roadmap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub id: String,
    pub seq: u64,
    pub at: String,
    /// user | assistant | tool | system
    pub role: String,
    pub content: String,
    /// cursor_transcript | cursor_bubble | import | mcp
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureCommit {
    pub id: String,
    pub at: String,
    pub intent: String,
    #[serde(default)]
    pub branch_purpose: String,
    #[serde(default)]
    pub previous_summary: String,
    pub contribution: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub rejected_paths: Vec<String>,
    pub log_from_seq: u64,
    pub log_to_seq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    #[serde(default)]
    pub block_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSummary {
    pub workspace_id: String,
    pub message_count: u32,
    pub commit_count: u32,
    pub latest_seq: u64,
    pub latest_commit_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLogWindow {
    pub workspace_id: String,
    pub from_seq: u64,
    pub to_seq: u64,
    pub messages: Vec<LogMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextCommitWindow {
    pub workspace_id: String,
    pub commits: Vec<CaptureCommit>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectBindings {
    /// Cursor sanitized project folder name → workspace id
    #[serde(default)]
    pub cursor_projects: HashMap<String, String>,
    /// Absolute repo path (normalized) → workspace id
    #[serde(default)]
    pub repo_paths: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecorderFileState {
    pub byte_offset: u64,
    pub workspace_id: String,
    /// `main` or branch slug — separate offsets per capture line.
    #[serde(default = "default_capture_branch_name")]
    pub capture_branch: String,
    #[serde(default)]
    pub composer_id: Option<String>,
    /// Lines already ingested from this file (for stable source_ref).
    #[serde(default)]
    pub lines_ingested: u64,
}

pub fn default_capture_branch_name() -> String {
    "main".to_string()
}

/// `None` = main line; `Some(slug)` = `branches/{slug}/`.
pub fn normalize_branch_slug(branch: Option<&str>) -> Option<&str> {
    match branch {
        None | Some("") | Some("main") => None,
        Some(s) => Some(s),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecorderState {
    #[serde(default)]
    pub files: HashMap<String, RecorderFileState>,
}

pub fn bindings_path() -> PathBuf {
    default_capture_root()
        .parent()
        .unwrap_or(Path::new("."))
        .join("bindings.json")
}

pub fn recorder_state_path() -> PathBuf {
    default_capture_root()
        .parent()
        .unwrap_or(Path::new("."))
        .join("recorder_state.json")
}

pub fn load_bindings() -> Result<ProjectBindings, String> {
    let path = bindings_path();
    if !path.exists() {
        return Ok(ProjectBindings::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_bindings(bindings: &ProjectBindings) -> Result<(), String> {
    let path = bindings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(bindings).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

pub fn load_recorder_state() -> Result<RecorderState, String> {
    let path = recorder_state_path();
    if !path.exists() {
        return Ok(RecorderState::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub fn save_recorder_state(state: &RecorderState) -> Result<(), String> {
    let path = recorder_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

#[derive(Debug, Clone)]
pub struct CaptureStore {
    root: PathBuf,
}

impl CaptureStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        Ok(Self { root })
    }

    pub fn default_open() -> Result<Self, String> {
        Self::new(default_capture_root())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn ws_dir(&self, workspace_id: &str) -> PathBuf {
        self.root.join(workspace_id)
    }

    pub fn branch_dir(&self, workspace_id: &str, slug: &str) -> PathBuf {
        self.branches_root(workspace_id).join(slug)
    }

    pub fn branches_root(&self, workspace_id: &str) -> PathBuf {
        self.ws_dir(workspace_id).join("branches")
    }

    pub fn branch_meta_path(&self, workspace_id: &str, slug: &str) -> PathBuf {
        self.branch_dir(workspace_id, slug).join("meta.json")
    }

    fn commits_path(&self, workspace_id: &str) -> PathBuf {
        self.commits_path_on_line(workspace_id, None)
    }

    fn log_path_on_line(&self, workspace_id: &str, branch: Option<&str>) -> PathBuf {
        match normalize_branch_slug(branch) {
            None => self.ws_dir(workspace_id).join("log.jsonl"),
            Some(slug) => self.branch_dir(workspace_id, slug).join("log.jsonl"),
        }
    }

    fn commits_path_on_line(&self, workspace_id: &str, branch: Option<&str>) -> PathBuf {
        match normalize_branch_slug(branch) {
            None => self.ws_dir(workspace_id).join("commits.jsonl"),
            Some(slug) => self.branch_dir(workspace_id, slug).join("commits.jsonl"),
        }
    }

    fn ensure_line(&self, workspace_id: &str, branch: Option<&str>) -> Result<(), String> {
        match normalize_branch_slug(branch) {
            None => self.ensure_ws(workspace_id),
            Some(slug) => fs::create_dir_all(self.branch_dir(workspace_id, slug))
                .map_err(|e| e.to_string()),
        }
    }

    fn meta_path(&self, workspace_id: &str) -> PathBuf {
        self.ws_dir(workspace_id).join("meta.json")
    }

    fn ensure_ws(&self, workspace_id: &str) -> Result<(), String> {
        fs::create_dir_all(self.ws_dir(workspace_id)).map_err(|e| e.to_string())
    }

    fn append_jsonl<T: Serialize>(&self, path: &Path, value: &T) -> Result<(), String> {
        let line = serde_json::to_string(value).map_err(|e| e.to_string())?;
        if contains_secrets(&line) {
            return Err("Refusing to write capture line that matches secret patterns".into());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(line.as_bytes())
            .and_then(|_| file.write_all(b"\n"))
            .map_err(|e| e.to_string())
    }

    pub fn read_log_messages(&self, workspace_id: &str) -> Result<Vec<LogMessage>, String> {
        self.read_log_messages_on_line(workspace_id, None)
    }

    pub fn read_log_messages_on_line(
        &self,
        workspace_id: &str,
        branch: Option<&str>,
    ) -> Result<Vec<LogMessage>, String> {
        let path = self.log_path_on_line(workspace_id, branch);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let msg: LogMessage = serde_json::from_str(trimmed).map_err(|e| e.to_string())?;
            out.push(msg);
        }
        Ok(out)
    }

    /// All decision commits for a workspace main line (newest last).
    pub fn read_commits_public(&self, workspace_id: &str) -> Result<Vec<CaptureCommit>, String> {
        self.read_commits_on_line(workspace_id, None)
    }

    pub fn read_commits_on_line(
        &self,
        workspace_id: &str,
        branch: Option<&str>,
    ) -> Result<Vec<CaptureCommit>, String> {
        let path = self.commits_path_on_line(workspace_id, branch);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let c: CaptureCommit = serde_json::from_str(trimmed).map_err(|e| e.to_string())?;
            out.push(c);
        }
        Ok(out)
    }

    pub fn get_meta(&self, workspace_id: &str) -> Result<CaptureMeta, String> {
        let path = self.meta_path(workspace_id);
        if !path.exists() {
            return Ok(CaptureMeta {
                workspace_id: workspace_id.to_string(),
                goal: String::new(),
                roadmap: String::new(),
            });
        }
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    pub fn set_meta(&self, meta: &CaptureMeta) -> Result<(), String> {
        self.ensure_ws(&meta.workspace_id)?;
        let path = self.meta_path(&meta.workspace_id);
        let text = serde_json::to_string_pretty(meta).map_err(|e| e.to_string())?;
        fs::write(&path, text).map_err(|e| e.to_string())
    }

    pub fn next_seq(&self, workspace_id: &str) -> Result<u64, String> {
        self.next_seq_on_line(workspace_id, None)
    }

    pub fn next_seq_on_line(&self, workspace_id: &str, branch: Option<&str>) -> Result<u64, String> {
        Ok(self
            .read_log_messages_on_line(workspace_id, branch)?
            .last()
            .map(|m| m.seq + 1)
            .unwrap_or(1))
    }

    pub fn last_seq_on_line(&self, workspace_id: &str, branch: Option<&str>) -> Result<u64, String> {
        Ok(self
            .read_log_messages_on_line(workspace_id, branch)?
            .last()
            .map(|m| m.seq)
            .unwrap_or(0))
    }

    /// Append a message to the workspace session log (main line).
    pub fn append_message(
        &self,
        workspace_id: &str,
        role: &str,
        content: &str,
        source: &str,
        source_ref: Option<String>,
    ) -> Result<LogMessage, String> {
        self.append_message_on_line(workspace_id, None, role, content, source, source_ref)
    }

    /// Append a message to main or branch capture line.
    pub fn append_message_on_line(
        &self,
        workspace_id: &str,
        branch: Option<&str>,
        role: &str,
        content: &str,
        source: &str,
        source_ref: Option<String>,
    ) -> Result<LogMessage, String> {
        self.ensure_line(workspace_id, branch)?;
        if let Some(r) = source_ref.as_ref() {
            if self.has_source_ref_on_line(workspace_id, branch, r)? {
                return Err("duplicate source_ref".into());
            }
        }
        let msg = LogMessage {
            id: Uuid::new_v4().to_string(),
            seq: self.next_seq_on_line(workspace_id, branch)?,
            at: Utc::now().to_rfc3339(),
            role: role.to_string(),
            content: redact_secrets(content),
            source: source.to_string(),
            source_ref,
        };
        self.append_jsonl(&self.log_path_on_line(workspace_id, branch), &msg)?;
        Ok(msg)
    }

    fn has_source_ref_on_line(
        &self,
        workspace_id: &str,
        branch: Option<&str>,
        source_ref: &str,
    ) -> Result<bool, String> {
        Ok(self
            .read_log_messages_on_line(workspace_id, branch)?
            .iter()
            .any(|m| m.source_ref.as_deref() == Some(source_ref)))
    }

    /// Decision commit — anchors to log seq range; contribution derived from messages in range.
    pub fn commit(
        &self,
        workspace_id: &str,
        intent: &str,
        branch_purpose: &str,
        note: &str,
        rejected_paths: Vec<String>,
        git_sha: Option<String>,
        block_ids: Vec<String>,
        log_from_seq: Option<u64>,
        log_to_seq: Option<u64>,
    ) -> Result<CaptureCommit, String> {
        self.commit_on_line(
            workspace_id,
            None,
            intent,
            branch_purpose,
            note,
            rejected_paths,
            git_sha,
            block_ids,
            log_from_seq,
            log_to_seq,
        )
    }

    pub fn commit_on_line(
        &self,
        workspace_id: &str,
        branch: Option<&str>,
        intent: &str,
        branch_purpose: &str,
        note: &str,
        rejected_paths: Vec<String>,
        git_sha: Option<String>,
        block_ids: Vec<String>,
        log_from_seq: Option<u64>,
        log_to_seq: Option<u64>,
    ) -> Result<CaptureCommit, String> {
        self.ensure_line(workspace_id, branch)?;
        let messages = self.read_log_messages_on_line(workspace_id, branch)?;
        if messages.is_empty() {
            return Err("Cannot commit: session log is empty".into());
        }
        let to_seq = log_to_seq.unwrap_or_else(|| messages.last().map(|m| m.seq).unwrap_or(0));
        let commits = self.read_commits_on_line(workspace_id, branch)?;
        let default_from = commits
            .last()
            .map(|c| c.log_to_seq.saturating_add(1))
            .unwrap_or(messages.first().map(|m| m.seq).unwrap_or(1));
        let from_seq = log_from_seq.unwrap_or(default_from);

        let slice: Vec<&LogMessage> = messages
            .iter()
            .filter(|m| m.seq >= from_seq && m.seq <= to_seq)
            .collect();
        if slice.is_empty() {
            return Err(format!(
                "Cannot commit: no log messages in range {from_seq}..={to_seq}"
            ));
        }

        let previous_summary = commits
            .last()
            .map(|c| format!("{}\n{}", c.previous_summary, c.contribution))
            .unwrap_or_default();
        let contribution = if note.trim().is_empty() {
            synthesize_contribution(&slice)
        } else {
            redact_secrets(note)
        };

        let commit = CaptureCommit {
            id: Uuid::new_v4().to_string(),
            at: Utc::now().to_rfc3339(),
            intent: redact_secrets(intent),
            branch_purpose: redact_secrets(branch_purpose),
            previous_summary: redact_secrets(&previous_summary),
            contribution,
            note: redact_secrets(note),
            rejected_paths: rejected_paths
                .into_iter()
                .map(|p| redact_secrets(&p))
                .collect(),
            log_from_seq: from_seq,
            log_to_seq: to_seq,
            git_sha,
            block_ids,
        };
        self.append_jsonl(&self.commits_path_on_line(workspace_id, branch), &commit)?;
        Ok(commit)
    }

    /// Promote a branch checkpoint onto the main commits line (merge confirmed).
    pub fn promote_branch_checkpoint(
        &self,
        workspace_id: &str,
        branch_slug: &str,
        label: &str,
    ) -> Result<CaptureCommit, String> {
        self.ensure_ws(workspace_id)?;
        let branch_commits = self.read_commits_on_line(workspace_id, Some(branch_slug))?;
        let latest = branch_commits.last().ok_or_else(|| {
            "branch has no checkpoints to merge — commit on branch first".to_string()
        })?;
        let main_commits = self.read_commits_on_line(workspace_id, None)?;
        let previous_summary = main_commits
            .last()
            .map(|c| format!("{}\n{}", c.previous_summary, c.contribution))
            .unwrap_or_default();
        let main_to_seq = self.last_seq_on_line(workspace_id, None)?;
        let commit = CaptureCommit {
            id: Uuid::new_v4().to_string(),
            at: Utc::now().to_rfc3339(),
            intent: format!("Merged branch: {}", label),
            branch_purpose: label.to_string(),
            previous_summary: redact_secrets(&previous_summary),
            contribution: latest.contribution.clone(),
            note: if latest.note.trim().is_empty() {
                latest.intent.clone()
            } else {
                latest.note.clone()
            },
            rejected_paths: latest.rejected_paths.clone(),
            log_from_seq: latest.log_from_seq,
            log_to_seq: main_to_seq.max(latest.log_to_seq),
            git_sha: latest.git_sha.clone(),
            block_ids: latest.block_ids.clone(),
        };
        self.append_jsonl(&self.commits_path(workspace_id), &commit)?;
        Ok(commit)
    }

    pub fn list_commits(&self, workspace_id: &str) -> Result<Vec<CaptureCommit>, String> {
        self.read_commits_on_line(workspace_id, None)
    }

    pub fn summary(&self, workspace_id: &str) -> Result<CaptureSummary, String> {
        let messages = self.read_log_messages(workspace_id)?;
        let commits = self.read_commits_on_line(workspace_id, None)?;
        Ok(CaptureSummary {
            workspace_id: workspace_id.to_string(),
            message_count: messages.len() as u32,
            commit_count: commits.len() as u32,
            latest_seq: messages.last().map(|m| m.seq).unwrap_or(0),
            latest_commit_at: commits.last().map(|c| c.at.clone()),
        })
    }

    /// Tier-2 context read: windowed segment of the session message stream.
    pub fn context_log(
        &self,
        workspace_id: &str,
        from_seq: Option<u64>,
        limit: usize,
    ) -> Result<ContextLogWindow, String> {
        let messages = self.read_log_messages(workspace_id)?;
        let start = from_seq.unwrap_or_else(|| messages.first().map(|m| m.seq).unwrap_or(1));
        let selected: Vec<LogMessage> = messages
            .into_iter()
            .filter(|m| m.seq >= start)
            .take(limit)
            .collect();
        let to_seq = selected.last().map(|m| m.seq).unwrap_or(start);
        Ok(ContextLogWindow {
            workspace_id: workspace_id.to_string(),
            from_seq: start,
            to_seq,
            messages: selected,
        })
    }

    /// Tier-2 context read: recent decision commits with log seq ranges.
    pub fn context_commits(
        &self,
        workspace_id: &str,
        limit: usize,
    ) -> Result<ContextCommitWindow, String> {
        let commits = self.read_commits_on_line(workspace_id, None)?;
        let start = commits.len().saturating_sub(limit);
        Ok(ContextCommitWindow {
            workspace_id: workspace_id.to_string(),
            commits: commits[start..].to_vec(),
        })
    }
}

fn synthesize_contribution(messages: &[&LogMessage]) -> String {
    let mut parts = Vec::new();
    for m in messages {
        let head: String = m.content.lines().take(8).collect::<Vec<_>>().join("\n");
        let snippet = if m.content.len() > 600 {
            format!("{}…", &head.chars().take(600).collect::<String>())
        } else {
            head
        };
        parts.push(format!("**{}** (seq {}):\n{}", m.role, m.seq, snippet));
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_append_and_commit_with_range() {
        let dir = tempfile::tempdir().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-capture-1";
        store
            .append_message(ws, "user", "Maybe auth bug", "import", None)
            .unwrap();
        store
            .append_message(ws, "assistant", "Check token refresh", "import", None)
            .unwrap();
        let cp = store
            .commit(
                ws,
                "chose refresh fix",
                "debug auth",
                "",
                vec![],
                None,
                vec![],
                None,
                None,
            )
            .unwrap();
        assert_eq!(cp.log_from_seq, 1);
        assert_eq!(cp.log_to_seq, 2);
        assert!(cp.contribution.contains("auth"));
        let summary = store.summary(ws).unwrap();
        assert_eq!(summary.message_count, 2);
        assert_eq!(summary.commit_count, 1);
    }

    #[test]
    fn branch_line_isolated_from_main() {
        let dir = tempfile::tempdir().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-branch-lines";
        store
            .append_message(ws, "user", "main msg", "import", None)
            .unwrap();
        store
            .append_message_on_line(ws, Some("try-redis"), "user", "branch msg", "import", None)
            .unwrap();
        assert_eq!(store.read_log_messages(ws).unwrap().len(), 1);
        assert_eq!(
            store
                .read_log_messages_on_line(ws, Some("try-redis"))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            store
                .read_log_messages_on_line(ws, Some("try-redis"))
                .unwrap()[0]
                .content,
            "branch msg"
        );
    }

    #[test]
    fn dedupe_source_ref() {
        let dir = tempfile::tempdir().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-dedupe";
        store
            .append_message(
                ws,
                "user",
                "hello",
                "cursor_transcript",
                Some("file:1".into()),
            )
            .unwrap();
        assert!(store
            .append_message(
                ws,
                "user",
                "hello again",
                "cursor_transcript",
                Some("file:1".into()),
            )
            .is_err());
    }
}
