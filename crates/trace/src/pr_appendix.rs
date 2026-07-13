//! Optional capture appendix for PR markdown export.

use crate::branches::list_branches_for_workspace;
use crate::capture::{CaptureStore, LogMessage};
use crate::capture_scope::resolve_capture_log_boundary;

const DEFAULT_COMMIT_LIMIT: usize = 5;
pub const DEFAULT_LOG_SLICE: &str = "past_50";
pub const SINCE_CAPTURE_LOG_MAX: usize = 100;

/// How many messages to include from the workspace capture log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogSliceMode {
    Past(usize),
    First(usize),
    SinceLastCaptureStart,
}

impl LogSliceMode {
    pub fn label(&self) -> &'static str {
        match self {
            LogSliceMode::Past(n) => match n {
                25 => "Past 25",
                50 => "Past 50",
                75 => "Past 75",
                100 => "Past 100",
                _ => "Past N",
            },
            LogSliceMode::First(n) => match n {
                25 => "First 25",
                50 => "First 50",
                75 => "First 75",
                100 => "First 100",
                _ => "First N",
            },
            LogSliceMode::SinceLastCaptureStart => "Since last capture start",
        }
    }

    pub fn section_heading(&self) -> String {
        format!("### Session log ({})\n\n", self.label())
    }
}

/// Parse desktop/MCP `trace_log_slice` values.
pub fn parse_log_slice_mode(raw: &str) -> Result<LogSliceMode, String> {
    let key = raw.trim().to_lowercase();
    if key == "since_last_capture_start" {
        return Ok(LogSliceMode::SinceLastCaptureStart);
    }
    if let Some(rest) = key.strip_prefix("past_") {
        let n: usize = rest
            .parse()
            .map_err(|_| invalid_log_slice(raw))?;
        validate_log_slice_preset(n)?;
        return Ok(LogSliceMode::Past(n));
    }
    if let Some(rest) = key.strip_prefix("first_") {
        let n: usize = rest
            .parse()
            .map_err(|_| invalid_log_slice(raw))?;
        validate_log_slice_preset(n)?;
        return Ok(LogSliceMode::First(n));
    }
    Err(invalid_log_slice(raw))
}

fn invalid_log_slice(raw: &str) -> String {
    format!(
        "invalid trace_log_slice `{raw}` — use past_25|50|75|100, first_25|50|75|100, or since_last_capture_start"
    )
}

fn validate_log_slice_preset(n: usize) -> Result<(), String> {
    if [25, 50, 75, 100].contains(&n) {
        Ok(())
    } else {
        Err(format!(
            "invalid log slice count {n} — allowed presets: 25, 50, 75, 100"
        ))
    }
}

/// Controls which capture lanes appear in PR trace export.
#[derive(Debug, Clone)]
pub struct PrTraceAppendixOptions {
    pub include_checkpoints: bool,
    pub include_log: bool,
    pub include_branch_logs: bool,
    pub commit_limit: usize,
    pub log_slice_mode: LogSliceMode,
}

impl Default for PrTraceAppendixOptions {
    fn default() -> Self {
        Self {
            include_checkpoints: true,
            include_log: false,
            include_branch_logs: false,
            commit_limit: DEFAULT_COMMIT_LIMIT,
            log_slice_mode: LogSliceMode::Past(50),
        }
    }
}

/// Compile decision commits (+ optional session log slice) for PR attachment.
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
        options.include_branch_logs,
        options.commit_limit,
        options.log_slice_mode,
    )
}

pub fn compile_pr_trace_appendix_with_limits(
    capture: &CaptureStore,
    workspace_id: &str,
    include_checkpoints: bool,
    include_log: bool,
    include_branch_logs: bool,
    commit_limit: usize,
    log_slice_mode: LogSliceMode,
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

    let mut log_messages: Vec<LogMessage> = Vec::new();
    let mut log_slice_unavailable = false;
    if include_log {
        let (messages, unavailable) = collect_log_messages(
            capture,
            workspace_id,
            include_branch_logs,
            log_slice_mode,
        )?;
        log_messages = messages;
        log_slice_unavailable = unavailable;
    }

    if commits.commits.is_empty() && log_messages.is_empty() && !include_log {
        return Ok(None);
    }

    let mut md = String::new();
    md.push_str("## Session trace (optional)\n\n");
    md.push_str(
        "_Decision checkpoints and/or a capped slice of the workspace capture log. Not a full chat dump._\n\n",
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

    if include_log {
        md.push_str(&log_slice_mode.section_heading());
        if log_slice_unavailable {
            md.push_str(
                "_No capture session found for this workspace. Start capture, chat, then stop — or choose Past/First N._\n\n",
            );
        } else if log_messages.is_empty() {
            md.push_str(
                "_No messages match this slice. Try a different option or send messages while capture is on._\n\n",
            );
        } else {
            for m in &log_messages {
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
    }

    Ok(Some(md))
}

fn apply_log_slice(messages: Vec<LogMessage>, mode: LogSliceMode, boundary: Option<u64>) -> Vec<LogMessage> {
    let mut msgs: Vec<LogMessage> = if let Some(start) = boundary {
        messages
            .into_iter()
            .filter(|m| m.seq > start)
            .collect()
    } else {
        messages
    };

    match mode {
        LogSliceMode::Past(n) => {
            if msgs.len() > n {
                msgs = msgs.split_off(msgs.len() - n);
            }
            msgs
        }
        LogSliceMode::First(n) => msgs.into_iter().take(n).collect(),
        LogSliceMode::SinceLastCaptureStart => {
            if msgs.len() > SINCE_CAPTURE_LOG_MAX {
                msgs = msgs.split_off(msgs.len() - SINCE_CAPTURE_LOG_MAX);
            }
            msgs
        }
    }
}

/// Returns `(messages, boundary_unavailable)` for since-last-capture mode.
fn collect_log_messages(
    capture: &CaptureStore,
    workspace_id: &str,
    include_branch_logs: bool,
    mode: LogSliceMode,
) -> Result<(Vec<LogMessage>, bool), String> {
    let boundary = if matches!(mode, LogSliceMode::SinceLastCaptureStart) {
        let boundary = resolve_capture_log_boundary(workspace_id)?;
        if boundary.is_none() {
            return Ok((vec![], true));
        }
        boundary
    } else {
        None
    };

    let mut messages = capture.read_log_messages(workspace_id)?;
    if include_branch_logs {
        let branches = list_branches_for_workspace(capture, workspace_id)?;
        for branch in branches {
            let branch_msgs =
                capture.read_log_messages_on_line(workspace_id, Some(&branch.slug))?;
            messages.extend(branch_msgs);
        }
        messages.sort_by_key(|m| m.seq);
        messages.dedup_by_key(|m| m.seq);
    }

    Ok((apply_log_slice(messages, mode, boundary), false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureStore;
    use tempfile::TempDir;

    fn seed_messages(store: &CaptureStore, ws: &str, count: usize) {
        for i in 1..=count {
            store
                .append_message(ws, "user", &format!("msg-{i}"), "test", None)
                .unwrap();
        }
    }

    #[test]
    fn parse_log_slice_modes() {
        assert_eq!(parse_log_slice_mode("past_50").unwrap(), LogSliceMode::Past(50));
        assert_eq!(
            parse_log_slice_mode("first_75").unwrap(),
            LogSliceMode::First(75)
        );
        assert_eq!(
            parse_log_slice_mode("since_last_capture_start").unwrap(),
            LogSliceMode::SinceLastCaptureStart
        );
        assert!(parse_log_slice_mode("past_99").is_err());
    }

    #[test]
    fn apply_log_slice_past_takes_tail() {
        let msgs: Vec<LogMessage> = (1..=10)
            .map(|seq| LogMessage {
                id: seq.to_string(),
                seq,
                at: "now".into(),
                role: "user".into(),
                content: format!("m{seq}"),
                source: "test".into(),
                source_ref: None,
            })
            .collect();
        let out = apply_log_slice(msgs, LogSliceMode::Past(3), None);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].seq, 8);
        assert_eq!(out[2].seq, 10);
    }

    #[test]
    fn apply_log_slice_first_takes_head() {
        let msgs: Vec<LogMessage> = (1..=10)
            .map(|seq| LogMessage {
                id: seq.to_string(),
                seq,
                at: "now".into(),
                role: "user".into(),
                content: format!("m{seq}"),
                source: "test".into(),
                source_ref: None,
            })
            .collect();
        let out = apply_log_slice(msgs, LogSliceMode::First(3), None);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].seq, 1);
        assert_eq!(out[2].seq, 3);
    }

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
    fn appendix_log_past_50_uses_recent_messages() {
        let dir = TempDir::new().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-1";
        seed_messages(&store, ws, 60);
        let md = compile_pr_trace_appendix_with_options(
            &store,
            ws,
            &PrTraceAppendixOptions {
                include_checkpoints: false,
                include_log: true,
                log_slice_mode: LogSliceMode::Past(50),
                ..Default::default()
            },
        )
        .unwrap()
        .unwrap();
        assert!(md.contains("Session log (Past 50)"));
        assert!(md.contains("msg-60"));
        assert!(!md.contains("(seq 1):"));
        assert!(md.contains("(seq 11):"));
    }

    #[test]
    fn appendix_empty_log_notice_when_slice_empty() {
        let dir = TempDir::new().unwrap();
        let store = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-1";
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
        assert!(md.contains("No messages match this slice"));
    }
}
