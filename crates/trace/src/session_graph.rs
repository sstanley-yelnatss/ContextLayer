//! Session graph — git-style lane model for capture checkpoints, branches, and ranges.

use serde::{Deserialize, Serialize};

use crate::branches::list_branches_for_workspace;
use crate::capture::{CaptureStore, LogMessage};
use crate::capture_scope::load_workspace_capture_prefs;
use crate::recording::{active_session_for_workspace, list_active_sessions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGraphLane {
    pub id: String,
    pub label: String,
    /// main | active | merged_confirmed | merged_rejected
    pub status: String,
    pub color_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGraphRow {
    pub id: String,
    /// checkpoint | branch_fork | branch_merge | capture_started | capture_stopped | message_range
    pub kind: String,
    pub lane: String,
    pub at: String,
    pub primary_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_label: Option<String>,
    pub log_from_seq: u64,
    pub log_to_seq: u64,
    pub message_count: u32,
    #[serde(default)]
    pub linked_block_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default)]
    pub rejected_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_outcome: Option<String>,
    #[serde(default)]
    pub is_active_head: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionGraph {
    pub workspace_id: String,
    pub rows: Vec<SessionGraphRow>,
    pub lanes: Vec<SessionGraphLane>,
    pub capture_active: bool,
    pub empty: bool,
}

#[derive(Debug, Clone)]
struct LaneEvent {
    kind: &'static str,
    lane: String,
    at: String,
    sort_ms: i64,
    primary: String,
    secondary: Option<String>,
    log_from: u64,
    log_to: u64,
    linked_block_ids: Vec<String>,
    intent: Option<String>,
    note: Option<String>,
    rejected_paths: Vec<String>,
    git_sha: Option<String>,
    merge_outcome: Option<String>,
    /// For gap calculation: seq after this event (exclusive end of "covered" region going forward in time)
    seq_after: u64,
    /// Seq before this event (exclusive start going backward)
    seq_before: u64,
}

fn parse_at_ms(at: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(at)
        .map(|d| d.timestamp_millis())
        .unwrap_or(0)
}

fn truncate(s: &str, max: usize) -> String {
    let t = s.trim().replace('\n', " ");
    if t.chars().count() <= max {
        t
    } else {
        let mut out: String = t.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn first_user_preview(messages: &[LogMessage], from: u64, to: u64) -> Option<String> {
    messages
        .iter()
        .filter(|m| m.seq >= from && m.seq <= to && m.role == "user")
        .map(|m| truncate(&m.content, 60))
        .find(|s| !s.is_empty())
}

fn count_messages(messages: &[LogMessage], from: u64, to: u64) -> u32 {
    messages
        .iter()
        .filter(|m| m.seq >= from && m.seq <= to)
        .count() as u32
}

/// Timestamp when a message range ended — last message in the seq window.
fn last_message_at(messages: &[LogMessage], from: u64, to: u64) -> Option<String> {
    messages
        .iter()
        .filter(|m| m.seq >= from && m.seq <= to)
        .max_by_key(|m| m.seq)
        .map(|m| m.at.clone())
}

/// Newest-first tiebreak: higher log_to_seq first; message_range loses to structural kinds.
fn cmp_rows_newest_first(a: &SessionGraphRow, b: &SessionGraphRow) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let by_at = parse_at_ms(&b.at).cmp(&parse_at_ms(&a.at));
    if by_at != Ordering::Equal {
        return by_at;
    }
    let by_seq = b.log_to_seq.cmp(&a.log_to_seq);
    if by_seq != Ordering::Equal {
        return by_seq;
    }
    match (
        a.kind.as_str() == "message_range",
        b.kind.as_str() == "message_range",
    ) {
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => Ordering::Equal,
    }
}

fn lane_label(slug: &str, branch_label: Option<&str>) -> String {
    if slug == "main" {
        "main".to_string()
    } else {
        branch_label
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .unwrap_or_else(|| slug.to_string())
    }
}

fn branch_color_key(index: usize, status: &str) -> String {
    if status == "merged_confirmed" || status == "merged_rejected" {
        format!("merged_{index}")
    } else {
        format!("fork_{index}")
    }
}

pub fn build_session_graph(capture: &CaptureStore, workspace_id: &str) -> Result<SessionGraph, String> {
    let main_messages = capture.read_log_messages_on_line(workspace_id, None)?;
    let main_commits = capture.read_commits_on_line(workspace_id, None)?;
    let branches = list_branches_for_workspace(capture, workspace_id)?;
    let prefs = load_workspace_capture_prefs(workspace_id);
    let active_sessions = list_active_sessions()?;
    let active_session = active_session_for_workspace(&active_sessions, workspace_id);

    let mut lanes: Vec<SessionGraphLane> = vec![SessionGraphLane {
        id: "main".to_string(),
        label: "main".to_string(),
        status: "main".to_string(),
        color_key: "main".to_string(),
    }];

    for (i, b) in branches.iter().enumerate() {
        lanes.push(SessionGraphLane {
            id: b.slug.clone(),
            label: b.label.clone(),
            status: b.status.clone(),
            color_key: branch_color_key(i, &b.status),
        });
    }

    let mut events: Vec<LaneEvent> = Vec::new();

    // Capture started / stopped on main
    if let Some(session) = active_session {
        let scope = {
            let label = session.label.trim();
            if !label.is_empty() {
                Some(truncate(label, 60))
            } else {
                session
                    .cursor_project
                    .as_ref()
                    .map(|p| truncate(p, 60))
            }
        };
        events.push(LaneEvent {
            kind: "capture_started",
            lane: "main".to_string(),
            at: session.started_at.clone(),
            sort_ms: parse_at_ms(&session.started_at),
            primary: "Capture started".to_string(),
            secondary: scope,
            log_from: session.log_seq_at_start.saturating_add(1),
            log_to: session.log_seq_at_start,
            linked_block_ids: vec![],
            intent: None,
            note: None,
            rejected_paths: vec![],
            git_sha: None,
            merge_outcome: None,
            seq_after: session.log_seq_at_start,
            seq_before: session.log_seq_at_start.saturating_sub(1),
        });
    } else if let Some(ref last) = prefs.last_capture {
        events.push(LaneEvent {
            kind: "capture_started",
            lane: "main".to_string(),
            at: last.started_at.clone(),
            sort_ms: parse_at_ms(&last.started_at),
            primary: "Capture started".to_string(),
            secondary: None,
            log_from: last.log_seq_at_start.saturating_add(1),
            log_to: last.log_seq_at_start,
            linked_block_ids: vec![],
            intent: None,
            note: None,
            rejected_paths: vec![],
            git_sha: None,
            merge_outcome: None,
            seq_after: last.log_seq_at_start,
            seq_before: last.log_seq_at_start.saturating_sub(1),
        });
        events.push(LaneEvent {
            kind: "capture_stopped",
            lane: "main".to_string(),
            at: last.stopped_at.clone(),
            sort_ms: parse_at_ms(&last.stopped_at),
            primary: "Capture stopped".to_string(),
            secondary: Some(format!(
                "{} messages",
                count_messages(&main_messages, last.log_seq_at_start.saturating_add(1), last.log_seq_at_stop)
            )),
            log_from: last.log_seq_at_stop,
            log_to: last.log_seq_at_stop,
            linked_block_ids: vec![],
            intent: None,
            note: None,
            rejected_paths: vec![],
            git_sha: None,
            merge_outcome: None,
            seq_after: last.log_seq_at_stop,
            seq_before: last.log_seq_at_stop.saturating_sub(1),
        });
    }

    // Main checkpoints
    for c in &main_commits {
        let secondary = if c.note.trim().is_empty() {
            None
        } else {
            Some(truncate(&c.note, 60))
        };
        events.push(LaneEvent {
            kind: "checkpoint",
            lane: "main".to_string(),
            at: c.at.clone(),
            sort_ms: parse_at_ms(&c.at),
            primary: c.intent.clone(),
            secondary,
            log_from: c.log_from_seq,
            log_to: c.log_to_seq,
            linked_block_ids: c.block_ids.clone(),
            intent: Some(c.intent.clone()),
            note: if c.note.trim().is_empty() {
                None
            } else {
                Some(c.note.clone())
            },
            rejected_paths: c.rejected_paths.clone(),
            git_sha: c.git_sha.clone(),
            merge_outcome: None,
            seq_after: c.log_to_seq,
            seq_before: c.log_from_seq.saturating_sub(1),
        });
    }

    // Branches
    for b in &branches {
        let branch_msgs = capture
            .read_log_messages_on_line(workspace_id, Some(&b.slug))
            .unwrap_or_default();
        let branch_commits = capture
            .read_commits_on_line(workspace_id, Some(&b.slug))
            .unwrap_or_default();

        events.push(LaneEvent {
            kind: "branch_fork",
            lane: b.slug.clone(),
            at: b.created_at.clone(),
            sort_ms: parse_at_ms(&b.created_at),
            primary: b.label.clone(),
            secondary: Some(format!("Fork from main @ seq {}", b.main_log_seq_at_fork)),
            log_from: b.main_log_seq_at_fork,
            log_to: b.main_log_seq_at_fork,
            linked_block_ids: vec![],
            intent: None,
            note: None,
            rejected_paths: vec![],
            git_sha: None,
            merge_outcome: None,
            seq_after: b.main_log_seq_at_fork,
            seq_before: b.main_log_seq_at_fork.saturating_sub(1),
        });

        for c in &branch_commits {
            let secondary = if c.note.trim().is_empty() {
                None
            } else {
                Some(truncate(&c.note, 60))
            };
            events.push(LaneEvent {
                kind: "checkpoint",
                lane: b.slug.clone(),
                at: c.at.clone(),
                sort_ms: parse_at_ms(&c.at),
                primary: c.intent.clone(),
                secondary,
                log_from: c.log_from_seq,
                log_to: c.log_to_seq,
                linked_block_ids: c.block_ids.clone(),
                intent: Some(c.intent.clone()),
                note: if c.note.trim().is_empty() {
                    None
                } else {
                    Some(c.note.clone())
                },
                rejected_paths: c.rejected_paths.clone(),
                git_sha: c.git_sha.clone(),
                merge_outcome: None,
                seq_after: c.log_to_seq,
                seq_before: c.log_from_seq.saturating_sub(1),
            });
        }

        if let Some(merged_at) = &b.merged_at {
            let outcome = if b.status == "merged_rejected" {
                "rejected"
            } else {
                "confirmed"
            };
            let last_seq = branch_msgs.last().map(|m| m.seq).unwrap_or(b.main_log_seq_at_fork);
            events.push(LaneEvent {
                kind: "branch_merge",
                lane: b.slug.clone(),
                at: merged_at.clone(),
                sort_ms: parse_at_ms(merged_at),
                primary: format!("Merged: {}", b.label),
                secondary: Some(outcome.to_string()),
                log_from: last_seq,
                log_to: last_seq,
                linked_block_ids: vec![],
                intent: None,
                note: None,
                rejected_paths: vec![],
                git_sha: None,
                merge_outcome: Some(outcome.to_string()),
                seq_after: last_seq,
                seq_before: last_seq.saturating_sub(1),
            });
        }
    }

    let capture_active = active_session.is_some();
    let latest_main_seq = main_messages.last().map(|m| m.seq).unwrap_or(0);

    let mut rows: Vec<SessionGraphRow> = Vec::new();

    // Range rows per lane
    let lane_ids: Vec<String> = lanes.iter().map(|l| l.id.clone()).collect();
    for lane_id in &lane_ids {
        let messages: Vec<LogMessage> = if lane_id == "main" {
            main_messages.clone()
        } else {
            capture
                .read_log_messages_on_line(workspace_id, Some(lane_id))
                .unwrap_or_default()
        };

        let mut lane_events: Vec<&LaneEvent> = events.iter().filter(|e| e.lane == *lane_id).collect();
        lane_events.sort_by_key(|e| e.sort_ms);

        if lane_events.is_empty() {
            continue;
        }

        // Gap before first event
        let first = lane_events[0];
        if first.log_from > 1 || first.kind == "capture_started" {
            let from = if first.kind == "capture_started" {
                first.seq_after.saturating_add(1)
            } else {
                1
            };
            let to = first.log_from.saturating_sub(1).max(from);
            if from <= to {
                let count = count_messages(&messages, from, to);
                if count > 0 {
                    if let Some(at) = last_message_at(&messages, from, to) {
                        let preview = first_user_preview(&messages, from, to);
                        rows.push(SessionGraphRow {
                            id: format!("range-{lane_id}-{from}-{to}"),
                            kind: "message_range".to_string(),
                            lane: lane_id.clone(),
                            at,
                            primary_label: format!("{count} messages · {}", lane_label(lane_id, None)),
                            secondary_label: preview,
                            log_from_seq: from,
                            log_to_seq: to,
                            message_count: count,
                            linked_block_ids: vec![],
                            intent: None,
                            note: None,
                            rejected_paths: vec![],
                            git_sha: None,
                            merge_outcome: None,
                            is_active_head: false,
                        });
                    }
                }
            }
        }

        for w in lane_events.windows(2) {
            let older = w[0];
            let newer = w[1];
            let from = older.seq_after.saturating_add(1);
            let to = newer.log_from.saturating_sub(1);
            if from > to {
                continue;
            }
            let count = count_messages(&messages, from, to);
            if count == 0 {
                continue;
            }
            let Some(at) = last_message_at(&messages, from, to) else {
                continue;
            };
            let preview = first_user_preview(&messages, from, to);
            rows.push(SessionGraphRow {
                id: format!("range-{lane_id}-{from}-{to}"),
                kind: "message_range".to_string(),
                lane: lane_id.clone(),
                at,
                primary_label: format!(
                    "{count} messages · {}",
                    lane_label(lane_id, branches.iter().find(|b| b.slug == *lane_id).map(|b| b.label.as_str()))
                ),
                secondary_label: preview,
                log_from_seq: from,
                log_to_seq: to,
                message_count: count,
                linked_block_ids: vec![],
                intent: None,
                note: None,
                rejected_paths: vec![],
                git_sha: None,
                merge_outcome: None,
                is_active_head: false,
            });
        }

        // Open range after newest event if capture active on this lane
        if capture_active {
            if let Some(session) = active_session {
                let on_lane = session.capture_branch == *lane_id
                    || (session.capture_branch == "main" && lane_id == "main");
                if on_lane {
                    if let Some(newest) = lane_events.last() {
                        let from = newest.seq_after.saturating_add(1);
                        let to = if lane_id == "main" {
                            latest_main_seq
                        } else {
                            messages.last().map(|m| m.seq).unwrap_or(from)
                        };
                        if from <= to {
                            let count = count_messages(&messages, from, to);
                            if count > 0 {
                                if let Some(at) = last_message_at(&messages, from, to) {
                                    let preview = first_user_preview(&messages, from, to);
                                    rows.push(SessionGraphRow {
                                        id: format!("range-{lane_id}-{from}-{to}-open"),
                                        kind: "message_range".to_string(),
                                        lane: lane_id.clone(),
                                        at,
                                        primary_label: format!(
                                            "{count} messages · {}",
                                            lane_label(lane_id, branches.iter().find(|b| b.slug == *lane_id).map(|b| b.label.as_str()))
                                        ),
                                        secondary_label: preview,
                                        log_from_seq: from,
                                        log_to_seq: to,
                                        message_count: count,
                                        linked_block_ids: vec![],
                                        intent: None,
                                        note: None,
                                        rejected_paths: vec![],
                                        git_sha: None,
                                        merge_outcome: None,
                                        is_active_head: false,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Structural rows
    for e in &events {
        let msg_count = if e.kind == "checkpoint" {
            let branch_messages = if e.lane == "main" {
                main_messages.clone()
            } else {
                capture
                    .read_log_messages_on_line(workspace_id, Some(&e.lane))
                    .unwrap_or_default()
            };
            count_messages(&branch_messages, e.log_from, e.log_to)
        } else {
            0
        };

        let row_id = if e.kind == "checkpoint" {
            format!("checkpoint-{}-{}", e.lane, e.at)
        } else {
            format!("{}-{}-{}", e.kind, e.lane, e.at)
        };

        rows.push(SessionGraphRow {
            id: row_id,
            kind: e.kind.to_string(),
            lane: e.lane.clone(),
            at: e.at.clone(),
            primary_label: e.primary.clone(),
            secondary_label: e.secondary.clone(),
            log_from_seq: e.log_from,
            log_to_seq: e.log_to,
            message_count: msg_count,
            linked_block_ids: e.linked_block_ids.clone(),
            intent: e.intent.clone(),
            note: e.note.clone(),
            rejected_paths: e.rejected_paths.clone(),
            git_sha: e.git_sha.clone(),
            merge_outcome: e.merge_outcome.clone(),
            is_active_head: false,
        });
    }

    // Newest first (ranges use last-message time; tiebreak: seq then structural over ranges)
    rows.sort_by(cmp_rows_newest_first);

    // Mark active head
    if capture_active {
        if let Some(session) = active_session {
            let lane = if session.capture_branch == "main" {
                "main"
            } else {
                session.capture_branch.as_str()
            };
            if let Some(row) = rows.iter_mut().find(|r| r.lane == lane) {
                row.is_active_head = true;
            }
        }
    }

    let empty = main_messages.is_empty()
        && main_commits.is_empty()
        && branches.is_empty()
        && !capture_active
        && prefs.last_capture.is_none();

    Ok(SessionGraph {
        workspace_id: workspace_id.to_string(),
        rows,
        lanes,
        capture_active,
        empty,
    })
}
