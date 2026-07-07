//! Capture session branches — fork/merge conversation lines within one workspace.

use std::fs;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::capture::{load_bindings, load_recorder_state, save_recorder_state, CaptureStore};
use crate::recording::{
    baseline_transcripts_for_session, load_capture_sessions, save_capture_sessions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    Active,
    MergedConfirmed,
    MergedRejected,
}

impl BranchStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::MergedConfirmed => "merged_confirmed",
            Self::MergedRejected => "merged_rejected",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "merged_confirmed" => Some(Self::MergedConfirmed),
            "merged_rejected" => Some(Self::MergedRejected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureBranchRecord {
    pub id: String,
    pub workspace_id: String,
    pub slug: String,
    pub label: String,
    pub status: String,
    pub created_at: String,
    /// Main-line log seq at fork — main log frozen at this point while branch is active.
    pub main_log_seq_at_fork: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<String>,
}

pub fn slugify_branch_label(label: &str) -> String {
    let raw: String = label
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '-'
            }
        })
        .collect();
    raw.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn unique_slug(capture: &CaptureStore, workspace_id: &str, label: &str) -> Result<String, String> {
    let base = slugify_branch_label(label);
    if base.is_empty() {
        return Err("branch label must contain at least one alphanumeric character".into());
    }
    let mut slug = base.clone();
    let mut n = 2u32;
    while capture.branch_meta_path(workspace_id, &slug).exists() {
        slug = format!("{base}-{n}");
        n += 1;
    }
    Ok(slug)
}

fn write_branch_meta(capture: &CaptureStore, record: &CaptureBranchRecord) -> Result<(), String> {
    let path = capture.branch_meta_path(&record.workspace_id, &record.slug);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = serde_json::to_string_pretty(record).map_err(|e| e.to_string())?;
    fs::write(&path, text).map_err(|e| e.to_string())
}

fn read_branch_meta(capture: &CaptureStore, workspace_id: &str, slug: &str) -> Result<CaptureBranchRecord, String> {
    let path = capture.branch_meta_path(workspace_id, slug);
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn update_branch_meta(
    capture: &CaptureStore,
    workspace_id: &str,
    slug: &str,
    f: impl FnOnce(&mut CaptureBranchRecord),
) -> Result<CaptureBranchRecord, String> {
    let mut record = read_branch_meta(capture, workspace_id, slug)?;
    f(&mut record);
    write_branch_meta(capture, &record)?;
    Ok(record)
}

pub fn list_branches_for_workspace(
    capture: &CaptureStore,
    workspace_id: &str,
) -> Result<Vec<CaptureBranchRecord>, String> {
    let branches_root = capture.branches_root(workspace_id);
    if !branches_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entries = fs::read_dir(&branches_root).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let Some(slug) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        if slug == "." {
            continue;
        }
        if let Ok(record) = read_branch_meta(capture, workspace_id, &slug) {
            out.push(record);
        }
    }
    out.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(out)
}

pub fn get_branch(capture: &CaptureStore, branch_id: &str) -> Result<Option<CaptureBranchRecord>, String> {
    let root = capture.root();
    if !root.is_dir() {
        return Ok(None);
    }
    let needle = branch_id.trim();
    for ws_entry in fs::read_dir(root).map_err(|e| e.to_string())?.flatten() {
        if !ws_entry.path().is_dir() {
            continue;
        }
        let Some(workspace_id) = ws_entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        for record in list_branches_for_workspace(capture, &workspace_id)? {
            if record.id == needle || record.id.starts_with(needle) {
                return Ok(Some(record));
            }
        }
    }
    Ok(None)
}

/// Fork capture to a branch subfolder. Requires active capture on main; does not restart watch.
pub fn create_capture_branch(
    capture: &CaptureStore,
    workspace_id: &str,
    label: &str,
) -> Result<CaptureBranchRecord, String> {
    let mut sessions = load_capture_sessions()?;
    let session_idx = sessions
        .active
        .iter()
        .position(|s| s.workspace_id == workspace_id)
        .ok_or_else(|| {
            format!(
                "no active capture session for workspace `{workspace_id}` — run start_capture first"
            )
        })?;
    if sessions.active[session_idx].capture_branch != "main" {
        return Err(format!(
            "already on capture branch `{}` — merge or reject before branching again",
            sessions.active[session_idx].capture_branch
        ));
    }

    let slug = unique_slug(capture, workspace_id, label)?;
    let main_log_seq_at_fork = capture.last_seq_on_line(workspace_id, None)?;

    let record = CaptureBranchRecord {
        id: Uuid::new_v4().to_string(),
        workspace_id: workspace_id.to_string(),
        slug: slug.clone(),
        label: label.trim().to_string(),
        status: BranchStatus::Active.as_str().to_string(),
        created_at: Utc::now().to_rfc3339(),
        main_log_seq_at_fork,
        merged_at: None,
    };
    write_branch_meta(capture, &record)?;

    let bindings = load_bindings()?;
    let mut recorder_state = load_recorder_state()?;
    let session = &sessions.active[session_idx];
    let _ = baseline_transcripts_for_session(
        &mut recorder_state,
        workspace_id,
        &slug,
        session.cursor_project.as_deref(),
        session.transcript_path.as_deref(),
        &bindings,
    )?;
    save_recorder_state(&recorder_state)?;

    sessions.active[session_idx].capture_branch = slug;
    save_capture_sessions(&sessions)?;

    Ok(record)
}

pub fn merge_capture_branch(
    capture: &CaptureStore,
    branch_id: &str,
    outcome: &str,
) -> Result<CaptureBranchRecord, String> {
    let confirmed = match outcome {
        "confirmed" | "merge" | "merged_confirmed" => true,
        "rejected" | "reject" | "merged_rejected" => false,
        other => {
            return Err(format!(
                "outcome must be confirmed or rejected (got `{other}`)"
            ))
        }
    };

    let branch = get_branch(capture, branch_id)?
        .ok_or_else(|| format!("branch not found: {branch_id}"))?;
    if BranchStatus::parse(&branch.status) != Some(BranchStatus::Active) {
        return Err(format!("branch `{}` is already merged", branch.label));
    }

    if confirmed {
        capture.promote_branch_checkpoint(&branch.workspace_id, &branch.slug, &branch.label)?;
        let _ = capture.append_message(
            &branch.workspace_id,
            "system",
            &format!(
                "Merged capture branch `{}` ({}) — outcome: confirmed",
                branch.label, branch.id
            ),
            "branch_merge",
            None,
        );
    }

    let status = if confirmed {
        BranchStatus::MergedConfirmed
    } else {
        BranchStatus::MergedRejected
    };
    let merged_at = Utc::now().to_rfc3339();
    let updated = update_branch_meta(capture, &branch.workspace_id, &branch.slug, |r| {
        r.status = status.as_str().to_string();
        r.merged_at = Some(merged_at.clone());
    })?;

    // Return session to main line if it was on this branch.
    let mut sessions = load_capture_sessions()?;
    if let Some(session) = sessions.active.iter_mut().find(|s| s.workspace_id == branch.workspace_id)
    {
        if session.capture_branch == branch.slug {
            session.capture_branch = "main".to_string();
        }
    }
    save_capture_sessions(&sessions)?;

    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::CaptureStore;
    use crate::recording::start_capture_session;

    #[test]
    fn slugify_label() {
        assert_eq!(slugify_branch_label("Try Redis!"), "try-redis");
    }

    #[test]
    fn branch_fork_freezes_main_and_merges() {
        let dir = tempfile::tempdir().unwrap();
        let capture = CaptureStore::new(dir.path()).unwrap();
        let ws = "ws-branch-flow";

        capture
            .append_message(ws, "user", "main before fork", "import", None)
            .unwrap();

        let (_session, _) = start_capture_session(ws, None, None, None).unwrap();

        let record = create_capture_branch(&capture, ws, "try-redis").unwrap();
        assert_eq!(record.slug, "try-redis");
        assert_eq!(record.main_log_seq_at_fork, 1);

        capture
            .append_message_on_line(ws, Some("try-redis"), "user", "branch only", "import", None)
            .unwrap();
        assert_eq!(capture.read_log_messages(ws).unwrap().len(), 1);

        capture
            .commit_on_line(
                ws,
                Some("try-redis"),
                "chose redis",
                "try-redis",
                "",
                vec![],
                None,
                vec![],
                None,
                None,
            )
            .unwrap();

        let merged = merge_capture_branch(&capture, &record.id, "confirmed").unwrap();
        assert_eq!(merged.status, "merged_confirmed");
        assert_eq!(capture.read_commits_public(ws).unwrap().len(), 1);

        let sessions = load_capture_sessions().unwrap();
        assert_eq!(sessions.active[0].capture_branch, "main");
    }
}
