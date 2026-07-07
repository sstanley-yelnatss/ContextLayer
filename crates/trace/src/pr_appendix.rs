//! Optional capture appendix for PR markdown export.

use crate::capture::CaptureStore;

const DEFAULT_COMMIT_LIMIT: usize = 5;
const DEFAULT_LOG_LIMIT: usize = 50;

/// Controls which capture lanes appear in PR trace export.
#[derive(Debug, Clone)]
pub struct PrTraceAppendixOptions {
    pub include_checkpoints: bool,
    pub include_log: bool,
    pub commit_limit: usize,
    pub log_message_limit: usize,
}

impl Default for PrTraceAppendixOptions {
    fn default() -> Self {
        Self {
            include_checkpoints: true,
            include_log: false,
            commit_limit: DEFAULT_COMMIT_LIMIT,
            log_message_limit: DEFAULT_LOG_LIMIT,
        }
    }
}

/// Compile decision commits (+ optional session log head) for PR attachment.
/// Returns `None` when nothing is selected or no matching capture data.
pub fn compile_pr_trace_appendix(
    capture: &CaptureStore,
    workspace_id: &str,
) -> Result<Option<String>, String> {
    compile_pr_trace_appendix_with_options(capture, workspace_id, &PrTraceAppendixOptions::default())
}

pub fn compile_pr_trace_appendix_with_options(
    capture: &CaptureStore,
    workspace_id: &str,
    options: &PrTraceAppendixOptions,
) -> Result<Option<String>, String> {
    compile_pr_trace_appendix_with_limits(
        capture,
        workspace_id,
        options.include_checkpoints,
        options.include_log,
        options.commit_limit,
        options.log_message_limit,
    )
}

pub fn compile_pr_trace_appendix_with_limits(
    capture: &CaptureStore,
    workspace_id: &str,
    include_checkpoints: bool,
    include_log: bool,
    commit_limit: usize,
    log_message_limit: usize,
) -> Result<Option<String>, String> {
    if !include_checkpoints && !include_log {
        return Ok(None);
    }

    let commits = if include_checkpoints {
        capture.context_commits(workspace_id, commit_limit)?
    } else {
        crate::capture::ContextCommitWindow {
            workspace_id: workspace_id.to_string(),
            commits: vec![],
        }
    };
    let log = if include_log {
        capture.context_log(workspace_id, None, log_message_limit)?
    } else {
        crate::capture::ContextLogWindow {
            workspace_id: workspace_id.to_string(),
            from_seq: 0,
            to_seq: 0,
            messages: vec![],
        }
    };

    if commits.commits.is_empty() && log.messages.is_empty() {
        return Ok(None);
    }

    let mut md = String::new();
    md.push_str("## Session trace (optional)\n\n");
    md.push_str(
        "_Decision checkpoints and/or session log from ContextLayer capture (since capture start) — not a full chat dump._\n\n",
    );

    if !commits.commits.is_empty() {
        md.push_str("### Decision checkpoints\n\n");
        for c in &commits.commits {
            md.push_str(&format!(
                "**{}** (log seq {}–{})\n\n",
                c.intent, c.log_from_seq, c.log_to_seq
            ));
            if !c.note.trim().is_empty() {
                md.push_str(&format!("Note: {}\n\n", c.note.trim()));
            }
            if !c.contribution.trim().is_empty() {
                md.push_str(&format!("Contribution:\n{}\n\n", c.contribution.trim()));
            }
            if !c.rejected_paths.is_empty() {
                md.push_str(&format!(
                    "Rejected paths: {}\n\n",
                    c.rejected_paths.join("; ")
                ));
            }
            if !c.block_ids.is_empty() {
                md.push_str(&format!(
                    "Linked blocks: `{}`\n\n",
                    c.block_ids.join("`, `")
                ));
            }
        }
    }

    if !log.messages.is_empty() {
        md.push_str("### Session log (since capture start)\n\n");
        for m in &log.messages {
            md.push_str(&format!("**{}** (seq {}):\n", m.role, m.seq));
            let body = if m.content.len() > 800 {
                format!("{}…", m.content.chars().take(800).collect::<String>())
            } else {
                m.content.clone()
            };
            md.push_str(&body);
            md.push_str("\n\n");
        }
    }

    Ok(Some(md))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureStore;
    use tempfile::TempDir;

    #[test]
    fn appendix_none_when_empty() {
        let dir = TempDir::new().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-1";
        assert!(compile_pr_trace_appendix(&store, ws).unwrap().is_none());
    }

    #[test]
    fn appendix_checkpoints_only_by_default() {
        let dir = TempDir::new().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-1";
        store
            .append_message(ws, "user", "tried auth fix", "test", None)
            .unwrap();
        store
            .commit(
                ws,
                "Ready for PR",
                "",
                "ship it",
                vec![],
                None,
                vec![],
                None,
                None,
            )
            .unwrap();
        let md = compile_pr_trace_appendix(&store, ws).unwrap().unwrap();
        assert!(md.contains("Decision checkpoints"));
        assert!(md.contains("Ready for PR"));
        assert!(!md.contains("Session log"));
    }

    #[test]
    fn appendix_log_only_when_requested() {
        let dir = TempDir::new().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-1";
        store
            .append_message(ws, "user", "hello", "test", None)
            .unwrap();
        let md = compile_pr_trace_appendix_with_options(
            &store,
            ws,
            &PrTraceAppendixOptions {
                include_checkpoints: false,
                include_log: true,
                ..Default::default()
            },
        )
        .unwrap()
        .unwrap();
        assert!(md.contains("Session log"));
        assert!(!md.contains("### Decision checkpoints"));
    }
}
