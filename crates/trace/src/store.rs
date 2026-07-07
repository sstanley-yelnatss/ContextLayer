//! Trace API — delegates to ContextLayer [`CaptureStore`].

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::capture::{CaptureStore, default_capture_root};
use crate::recording::active_capture_branch_slug;
use crate::redact::redact_secrets;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TraceRecord {
    Event(TraceEvent),
    Checkpoint(CheckpointCommit),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: String,
    pub workspace_id: String,
    pub at: String,
    pub event_type: String,
    pub summary: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointCommit {
    pub id: String,
    pub workspace_id: String,
    pub at: String,
    pub intent: String,
    pub note: String,
    #[serde(default)]
    pub rejected_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    #[serde(default)]
    pub block_ids: Vec<String>,
    #[serde(default)]
    pub log_from_seq: u64,
    #[serde(default)]
    pub log_to_seq: u64,
    #[serde(default)]
    pub contribution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    pub workspace_id: String,
    pub event_count: u32,
    pub checkpoint_count: u32,
    pub latest_checkpoint_at: Option<String>,
    #[serde(default)]
    pub message_count: u32,
    #[serde(default)]
    pub latest_log_seq: u64,
}

pub fn default_traces_dir() -> PathBuf {
    default_capture_root()
}

#[derive(Debug, Clone)]
pub struct TraceStore {
    capture: CaptureStore,
}

impl TraceStore {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self, String> {
        Ok(Self {
            capture: CaptureStore::new(dir)?,
        })
    }

    pub fn default_open() -> Result<Self, String> {
        Ok(Self {
            capture: CaptureStore::default_open()?,
        })
    }

    pub fn capture(&self) -> &CaptureStore {
        &self.capture
    }

    pub fn append_event(
        &self,
        workspace_id: &str,
        event_type: &str,
        summary: &str,
        payload: serde_json::Value,
    ) -> Result<TraceEvent, String> {
        let content = if payload.is_null() || payload.as_object().is_some_and(|o| o.is_empty()) {
            summary.to_string()
        } else {
            format!("{summary}\n\n{}", payload)
        };
        let msg = self.capture.append_message(
            workspace_id,
            "tool",
            &content,
            "mcp",
            None,
        )?;
        Ok(TraceEvent {
            id: msg.id,
            workspace_id: workspace_id.to_string(),
            at: msg.at,
            event_type: event_type.to_string(),
            summary: redact_secrets(summary),
            payload,
        })
    }

    pub fn commit_checkpoint(
        &self,
        workspace_id: &str,
        intent: &str,
        note: &str,
        rejected_paths: Vec<String>,
        git_sha: Option<String>,
        block_ids: Vec<String>,
    ) -> Result<CheckpointCommit, String> {
        let branch = active_capture_branch_slug(workspace_id)?;
        let commit = self.capture.commit_on_line(
            workspace_id,
            branch.as_deref(),
            intent,
            "",
            note,
            rejected_paths,
            git_sha,
            block_ids,
            None,
            None,
        )?;
        Ok(CheckpointCommit {
            id: commit.id,
            workspace_id: workspace_id.to_string(),
            at: commit.at,
            intent: commit.intent,
            note: commit.note,
            rejected_paths: commit.rejected_paths,
            git_sha: commit.git_sha,
            block_ids: commit.block_ids,
            log_from_seq: commit.log_from_seq,
            log_to_seq: commit.log_to_seq,
            contribution: commit.contribution,
        })
    }

    pub fn read_all(&self, workspace_id: &str) -> Result<Vec<TraceRecord>, String> {
        let mut out = Vec::new();
        for m in self.capture.read_log_messages(workspace_id)? {
            out.push(TraceRecord::Event(TraceEvent {
                id: m.id.clone(),
                workspace_id: workspace_id.to_string(),
                at: m.at,
                event_type: m.role.clone(),
                summary: m.content,
                payload: serde_json::json!({ "source": m.source }),
            }));
        }
        for c in self.capture.list_commits(workspace_id)? {
            out.push(TraceRecord::Checkpoint(CheckpointCommit {
                id: c.id,
                workspace_id: workspace_id.to_string(),
                at: c.at,
                intent: c.intent,
                note: c.note,
                rejected_paths: c.rejected_paths,
                git_sha: c.git_sha,
                block_ids: c.block_ids,
                log_from_seq: c.log_from_seq,
                log_to_seq: c.log_to_seq,
                contribution: c.contribution,
            }));
        }
        Ok(out)
    }

    pub fn list_checkpoints(&self, workspace_id: &str) -> Result<Vec<CheckpointCommit>, String> {
        Ok(self
            .capture
            .list_commits(workspace_id)?
            .into_iter()
            .map(|c| CheckpointCommit {
                id: c.id,
                workspace_id: workspace_id.to_string(),
                at: c.at,
                intent: c.intent,
                note: c.note,
                rejected_paths: c.rejected_paths,
                git_sha: c.git_sha,
                block_ids: c.block_ids,
                log_from_seq: c.log_from_seq,
                log_to_seq: c.log_to_seq,
                contribution: c.contribution,
            })
            .collect())
    }

    pub fn summary(&self, workspace_id: &str) -> Result<TraceSummary, String> {
        let s = self.capture.summary(workspace_id)?;
        Ok(TraceSummary {
            workspace_id: s.workspace_id,
            event_count: s.message_count,
            checkpoint_count: s.commit_count,
            latest_checkpoint_at: s.latest_commit_at,
            message_count: s.message_count,
            latest_log_seq: s.latest_seq,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_event_and_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let store = TraceStore::new(dir.path()).unwrap();
        let ws = "ws-1";
        store
            .capture()
            .append_message(ws, "user", "Logged hypothesis", "test", None)
            .unwrap();
        store
            .commit_checkpoint(
                ws,
                "chose fix A",
                "",
                vec!["cache invalidation only".into()],
                Some("abc123".into()),
                vec![],
            )
            .unwrap();
        let summary = store.summary(ws).unwrap();
        assert_eq!(summary.message_count, 1);
        assert_eq!(summary.checkpoint_count, 1);
        let cps = store.list_checkpoints(ws).unwrap();
        assert_eq!(cps[0].log_from_seq, 1);
    }
}
