//! Unified context reads (GCC-style `context --summary`, checkpoint lookup).

use crate::capture::{CaptureCommit, CaptureStore, CaptureSummary};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ContextSummary {
    pub workspace_id: String,
    pub capture: CaptureSummary,
    pub active_capture: bool,
    pub latest_checkpoint_intent: Option<String>,
    pub latest_checkpoint_at: Option<String>,
    pub open_branches: u32,
    /// Active capture line when session is running: `main` or branch slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_capture_branch: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckpointDetail {
    pub commit: CaptureCommit,
    pub log_messages: Vec<crate::capture::LogMessage>,
}

pub fn build_context_summary(
    capture: &CaptureStore,
    workspace_id: &str,
    active_capture: bool,
    open_branches: u32,
    active_capture_branch: Option<String>,
) -> Result<ContextSummary, String> {
    let cap = capture.summary(workspace_id)?;
    let commits = capture.read_commits_public(workspace_id)?;
    let latest = commits.last();
    Ok(ContextSummary {
        workspace_id: workspace_id.to_string(),
        capture: cap,
        active_capture,
        latest_checkpoint_intent: latest.map(|c| c.intent.clone()),
        latest_checkpoint_at: latest.map(|c| c.at.clone()),
        open_branches,
        active_capture_branch: if active_capture {
            active_capture_branch
        } else {
            None
        },
    })
}

pub fn find_checkpoint(
    capture: &CaptureStore,
    workspace_id: &str,
    id_or_sha: &str,
) -> Result<Option<CheckpointDetail>, String> {
    let needle = id_or_sha.trim().to_lowercase();
    let commits = capture.read_commits_public(workspace_id)?;
    let Some(commit) = commits.into_iter().find(|c| {
        c.id.to_lowercase() == needle
            || c.id.to_lowercase().starts_with(&needle)
            || c
                .git_sha
                .as_ref()
                .is_some_and(|s| s.to_lowercase().starts_with(&needle))
    }) else {
        return Ok(None);
    };
    let messages = capture.read_log_messages(workspace_id)?;
    let log_messages: Vec<_> = messages
        .into_iter()
        .filter(|m| m.seq >= commit.log_from_seq && m.seq <= commit.log_to_seq)
        .collect();
    Ok(Some(CheckpointDetail {
        commit,
        log_messages,
    }))
}
