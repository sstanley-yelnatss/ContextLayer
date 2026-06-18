//! Workspace hygiene — Phase 1.2 reasoning health queries

use chrono::{DateTime, Duration, Utc};

use crate::blocks::BlockEntry;
use crate::graph::GraphStore;
use crate::DbError;

const STALE_DAYS: i64 = 14;

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceHealthSummary {
    pub total_blocks: u32,
    pub belief_open: u32,
    pub belief_leading: u32,
    pub belief_confirmed: u32,
    pub belief_rejected: u32,
    pub needs_review: u32,
    pub reasoning_debt: u32,
    pub stale: u32,
    pub orphans: u32,
    pub dead_ends: u32,
    pub still_open: u32,
    pub decisions: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct HygieneItem {
    pub block_id: String,
    pub category: String,
    pub message: String,
    pub preview: String,
    pub days_open: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceHygieneReport {
    pub summary: WorkspaceHealthSummary,
    pub orphans: Vec<HygieneItem>,
    pub stale: Vec<HygieneItem>,
    pub still_open: Vec<HygieneItem>,
    pub dead_ends: Vec<HygieneItem>,
    pub decisions: Vec<HygieneItem>,
}

impl GraphStore {
    pub fn fetch_workspace_hygiene(
        &self,
        workspace_id: &str,
    ) -> Result<WorkspaceHygieneReport, DbError> {
        let blocks = self.fetch_blocks(workspace_id, false)?;
        let now = Utc::now();
        let stale_cutoff = now - Duration::days(STALE_DAYS);

        let mut orphans = Vec::new();
        let mut stale = Vec::new();
        let mut still_open = Vec::new();
        let mut dead_ends = Vec::new();
        let mut decisions = Vec::new();

        let mut summary = WorkspaceHealthSummary {
            total_blocks: blocks.len() as u32,
            belief_open: 0,
            belief_leading: 0,
            belief_confirmed: 0,
            belief_rejected: 0,
            needs_review: 0,
            reasoning_debt: 0,
            stale: 0,
            orphans: 0,
            dead_ends: 0,
            still_open: 0,
            decisions: 0,
        };

        for block in &blocks {
            let preview = block_preview(block);
            let updated = parse_ts(&block.updated_at);
            let days = (now - updated).num_days().max(0) as u32;

            match block.belief_state.as_str() {
                "open" => summary.belief_open += 1,
                "leaning_true" | "leaning_false" => summary.belief_leading += 1,
                "confirmed" => summary.belief_confirmed += 1,
                "rejected" => summary.belief_rejected += 1,
                _ => {}
            }
            if block.system_tag == "needs_review" {
                summary.needs_review += 1;
            }
            if block.system_tag == "reasoning_debt" {
                summary.reasoning_debt += 1;
            }

            if let Some(msg) = orphan_reason(block) {
                summary.orphans += 1;
                orphans.push(HygieneItem {
                    block_id: block.id.clone(),
                    category: "orphan".into(),
                    message: msg,
                    preview: preview.clone(),
                    days_open: Some(days),
                });
            }

            if is_stale(block, stale_cutoff) {
                summary.stale += 1;
                stale.push(HygieneItem {
                    block_id: block.id.clone(),
                    category: "stale".into(),
                    message: format!(
                        "Open {days} days; no evidence or conclusion added recently"
                    ),
                    preview: preview.clone(),
                    days_open: Some(days),
                });
            }

            if is_still_open(block) {
                summary.still_open += 1;
                still_open.push(HygieneItem {
                    block_id: block.id.clone(),
                    category: "still_open".into(),
                    message: if block.action.is_none() {
                        format!("Open {days} days; no action taken since creation")
                    } else {
                        format!("Open {days} days; hypothesis not yet resolved")
                    },
                    preview: preview.clone(),
                    days_open: Some(days),
                });
            }

            if is_dead_end(block) {
                summary.dead_ends += 1;
                dead_ends.push(HygieneItem {
                    block_id: block.id.clone(),
                    category: "dead_end".into(),
                    message: "Rejected or ruled out; avoid retesting this path".into(),
                    preview: preview.clone(),
                    days_open: Some(days),
                });
            }

            if let Some(decision) = decision_item(block) {
                summary.decisions += 1;
                decisions.push(decision);
            }
        }

        Ok(WorkspaceHygieneReport {
            summary,
            orphans,
            stale,
            still_open,
            dead_ends,
            decisions,
        })
    }
}

fn block_preview(block: &BlockEntry) -> String {
    if !block.title.is_empty() {
        return block.title.chars().take(80).collect();
    }
    block
        .hypothesis
        .as_ref()
        .map(|h| h.text.as_str())
        .or(block.action.as_ref().map(|a| a.text.as_str()))
        .or(block.evidence.as_ref().map(|e| e.text.as_str()))
        .or(block.conclusion.as_ref().map(|c| c.text.as_str()))
        .unwrap_or("(empty)")
        .chars()
        .take(80)
        .collect()
}

fn orphan_reason(block: &BlockEntry) -> Option<String> {
    let has_h = block.hypothesis.is_some();
    let has_a = block.action.is_some();
    let has_e = block.evidence.is_some();
    let has_c = block.conclusion.is_some();

    if has_a && !has_e {
        return Some("Action recorded but no evidence attached".into());
    }
    if has_e && !has_h {
        return Some("Evidence without a hypothesis".into());
    }
    if has_c && (!has_h || !has_e) {
        return Some("Conclusion missing hypothesis or evidence in this block".into());
    }
    if has_h && !has_a && !has_e && !has_c {
        return Some("Hypothesis only; no test or evidence yet".into());
    }
    None
}

fn is_stale(block: &BlockEntry, cutoff: DateTime<Utc>) -> bool {
    if !matches!(
        block.belief_state.as_str(),
        "open" | "leaning_true" | "leaning_false"
    ) {
        return false;
    }
    let updated = parse_ts(&block.updated_at);
    if updated > cutoff {
        return false;
    }
    block.evidence.is_none() || block.conclusion.is_none()
}

fn is_still_open(block: &BlockEntry) -> bool {
    matches!(
        block.belief_state.as_str(),
        "open" | "leaning_true" | "leaning_false"
    ) && block.hypothesis.is_some()
        && block.action.is_none()
}

fn is_dead_end(block: &BlockEntry) -> bool {
    block.belief_state == "rejected" || block.system_tag == "ruled_out"
}

fn decision_item(block: &BlockEntry) -> Option<HygieneItem> {
    let c = block.conclusion.as_ref()?;
    if matches!(c.tag.as_str(), "pivot" | "act" | "ignore" | "defer") {
        let label = match c.tag.as_str() {
            "pivot" => "Pivot",
            "act" => "Act",
            "ignore" => "Ignore",
            "defer" => "Defer",
            _ => &c.tag,
        };
        return Some(HygieneItem {
            block_id: block.id.clone(),
            category: "decision".into(),
            message: format!("Decision: {label}"),
            preview: c.text.chars().take(80).collect(),
            days_open: None,
        });
    }
    None
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SaveBlockInput;

    #[test]
    fn hygiene_detects_orphan_action_without_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let ws = store.create_workspace("T", "G", "blank").unwrap();
        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                action_text: Some("Sent curl request to test endpoint".into()),
                ..Default::default()
            })
            .unwrap();
        let report = store.fetch_workspace_hygiene(&ws.id).unwrap();
        assert_eq!(report.summary.orphans, 1);
    }

    #[test]
    fn hygiene_counts_decisions() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let ws = store.create_workspace("T", "G", "blank").unwrap();
        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                hypothesis_text: Some("Test hypothesis for hygiene".into()),
                evidence_text: Some("Some observed evidence here".into()),
                conclusion_text: Some("We should pivot away from this path".into()),
                conclusion_outcome: Some("refined".into()),
                conclusion_tag: Some("pivot".into()),
                ..Default::default()
            })
            .unwrap();
        let report = store.fetch_workspace_hygiene(&ws.id).unwrap();
        assert_eq!(report.summary.decisions, 1);
    }
}
