//! Read-only markdown compile — block-first (Phase 1.1) + PR export (B1)

use std::collections::HashSet;
use std::path::Path;

use contextlayer_core::{BeliefState, BlockSystemTag};
use contextlayer_db::{BlockEntry, GraphStore};

pub fn compile_workspace_summary_markdown(
    store: &GraphStore,
    workspace_id: &str,
) -> Result<String, String> {
    let workspace = store.get_workspace(workspace_id).map_err(|e| e.to_string())?;
    let blocks = store
        .fetch_blocks(workspace_id, false)
        .map_err(|e| e.to_string())?;

    let mut md = String::new();
    append_workspace_header(&mut md, &workspace.name, &workspace.goal, workspace.template.as_str(), None);
    md.push_str("## Reasoning blocks\n\n");
    if blocks.is_empty() {
        md.push_str("_No blocks yet._\n\n");
    } else {
        for block in &blocks {
            append_block(&mut md, block);
        }
    }

    Ok(md)
}

/// PR-sized export: only selected blocks, chronological order (oldest first).
pub fn compile_pr_export_markdown(
    store: &GraphStore,
    workspace_id: &str,
    block_ids: &[String],
) -> Result<String, String> {
    if block_ids.is_empty() {
        return Err("Select at least one block".into());
    }

    let workspace = store.get_workspace(workspace_id).map_err(|e| e.to_string())?;
    let all_blocks = store
        .fetch_blocks(workspace_id, false)
        .map_err(|e| e.to_string())?;
    let total = all_blocks.len();

    let id_set: HashSet<&str> = block_ids.iter().map(String::as_str).collect();
    let mut selected: Vec<&BlockEntry> = all_blocks
        .iter()
        .filter(|b| id_set.contains(b.id.as_str()))
        .collect();

    if selected.len() != id_set.len() {
        let found: HashSet<&str> = selected.iter().map(|b| b.id.as_str()).collect();
        let missing: Vec<&str> = id_set
            .iter()
            .copied()
            .filter(|id| !found.contains(id))
            .collect();
        return Err(format!("Block ID(s) not found in workspace: {}", missing.join(", ")));
    }

    selected.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    let selected_count = selected.len();

    let reviewer_summary = latest_conclusion_summary(&selected);
    let (unsettled_belief, incomplete) = count_pr_hygiene(&selected);

    let mut md = String::new();
    append_pr_header(&mut md, &workspace.name, &workspace.goal, reviewer_summary.as_deref());

    for (i, block) in selected.iter().enumerate() {
        if i > 0 {
            md.push_str("---\n\n");
        }
        append_pr_block(&mut md, block);
    }

    md.push_str("---\n\n");
    md.push_str(&format!(
        "Exported from ContextLayer ({selected_count} of {total} blocks in this workspace).\n",
    ));
    if let Some(note) = format_pr_hygiene_note(selected_count, unsettled_belief, incomplete) {
        md.push_str(&note);
    }

    Ok(md)
}

/// Resolve block IDs from explicit IDs and/or case-insensitive titles (unique per title).
pub fn resolve_pr_block_ids(
    store: &GraphStore,
    workspace_id: &str,
    block_ids: &[String],
    block_titles: &[String],
) -> Result<Vec<String>, String> {
    if block_ids.is_empty() && block_titles.is_empty() {
        return Err("Provide block_ids and/or block_titles".into());
    }

    let mut resolved: Vec<String> = Vec::new();
    let mut seen = HashSet::new();

    for id in block_ids {
        let id = id.trim();
        if id.is_empty() {
            continue;
        }
        store
            .get_block(id)
            .map_err(|e| e.to_string())
            .and_then(|b| {
                if b.workspace_id != workspace_id {
                    Err(format!("Block {id} is not in this workspace"))
                } else {
                    Ok(())
                }
            })?;
        if seen.insert(id.to_string()) {
            resolved.push(id.to_string());
        }
    }

    for title in block_titles {
        let title = title.trim();
        if title.is_empty() {
            continue;
        }
        let id = store
            .find_block_id_by_title(workspace_id, title)
            .map_err(|e| e.to_string())?;
        if seen.insert(id.clone()) {
            resolved.push(id);
        }
    }

    if resolved.is_empty() {
        return Err("No blocks resolved from block_ids or block_titles".into());
    }

    Ok(resolved)
}

fn is_unsettled_belief(belief_state: &str) -> bool {
    matches!(
        BeliefState::parse(belief_state),
        Some(BeliefState::Open | BeliefState::LeaningTrue | BeliefState::LeaningFalse)
    )
}

fn count_pr_hygiene(selected: &[&BlockEntry]) -> (usize, usize) {
    let unsettled = selected
        .iter()
        .filter(|b| is_unsettled_belief(&b.belief_state))
        .count();
    let incomplete = selected.iter().filter(|b| b.incomplete).count();
    (unsettled, incomplete)
}

fn format_pr_hygiene_note(selected_count: usize, unsettled: usize, incomplete: usize) -> Option<String> {
    if unsettled == 0 && incomplete == 0 {
        return None;
    }
    let mut parts = Vec::new();
    if unsettled > 0 {
        parts.push(format!(
            "{unsettled} of {selected_count} exported block(s) still have unsettled belief"
        ));
    }
    if incomplete > 0 {
        parts.push(format!("{incomplete} incomplete"));
    }
    Some(format!("Hygiene: {}.\n", parts.join("; ")))
}

fn latest_conclusion_summary(selected: &[&BlockEntry]) -> Option<String> {
    selected
        .iter()
        .filter(|b| b.conclusion.is_some())
        .max_by_key(|b| b.created_at.as_str())
        .and_then(|b| b.conclusion.as_ref())
        .map(|c| c.text.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn append_pr_header(md: &mut String, name: &str, goal: &str, reviewer_summary: Option<&str>) {
    md.push_str(&format!("PR Reasoning: {name}\n\n"));
    md.push_str(&format!("Goal: {goal}\n\n"));
    if let Some(summary) = reviewer_summary {
        md.push_str("Summary for reviewers:\n");
        md.push_str(summary);
        md.push_str("\n\n");
    }
    md.push_str("---\n\n");
}

fn append_pr_block(md: &mut String, block: &BlockEntry) {
    let belief = BeliefState::parse(&block.belief_state)
        .map(|b| b.label())
        .unwrap_or(&block.belief_state);
    let mut meta = vec![format!("Belief: {belief}")];
    if block.system_tag != "none" {
        let tag = BlockSystemTag::parse(&block.system_tag)
            .map(|t| t.label())
            .unwrap_or(&block.system_tag);
        meta.push(format!("Tag: {tag}"));
    }
    if let Some(ut) = &block.user_tag {
        if !ut.trim().is_empty() {
            meta.push(format!("Label: {ut}"));
        }
    }
    if let Some(c) = &block.conclusion {
        meta.push(format!("Outcome: {}", c.outcome));
        if c.tag != "none" {
            meta.push(format!("Decision: {}", c.tag));
        }
        if let Some(cl) = &c.confidence_level {
            meta.push(format!("Confidence: {cl}"));
        }
    }
    md.push_str(&meta.join(" · "));
    md.push_str("\n\n");

    if let Some(h) = &block.hypothesis {
        md.push_str("Hypothesis:\n");
        md.push_str(&h.text);
        md.push_str("\n\n");
    }
    if let Some(a) = &block.action {
        md.push_str("Action:\n");
        md.push_str(&a.text);
        md.push_str("\n\n");
    }
    if let Some(e) = &block.evidence {
        md.push_str("Evidence:\n");
        md.push_str(&e.text);
        md.push_str("\n");
        if let Some(src) = &e.source {
            if !src.trim().is_empty() {
                md.push_str(&format!("Source: {src}\n"));
            }
        }
        md.push_str("\n");
    }
    if let Some(c) = &block.conclusion {
        md.push_str("Conclusion:\n");
        md.push_str(&c.text);
        md.push_str("\n\n");
    }

    if block.incomplete {
        md.push_str("(Incomplete — some expected fields are still missing.)\n\n");
    }
}

fn append_workspace_header(
    md: &mut String,
    name: &str,
    goal: &str,
    template: &str,
    pr_block_count: Option<usize>,
) {
    match pr_block_count {
        Some(n) => {
            md.push_str(&format!("# PR reasoning: {name}\n\n"));
            md.push_str(&format!("**Goal:** {goal}\n\n"));
            md.push_str(&format!(
                "_Reasoning export for pull request review — {n} block(s) selected._\n\n"
            ));
        }
        None => {
            md.push_str(&format!("# {name}\n\n"));
            md.push_str(&format!("**Goal:** {goal}\n\n"));
            md.push_str(&format!("_Template: {template}_\n\n"));
        }
    }
}

fn append_block(md: &mut String, block: &BlockEntry) {
    let belief = BeliefState::parse(&block.belief_state)
        .map(|b| b.label())
        .unwrap_or(&block.belief_state);
    let tag = BlockSystemTag::parse(&block.system_tag)
        .map(|t| t.label())
        .unwrap_or(&block.system_tag);

    md.push_str(&format!("### {} (`{}`)\n\n", block.title, block.id));
    md.push_str(&format!("- Belief: {belief}\n"));
    if block.system_tag != "none" {
        md.push_str(&format!("- System tag: {tag}\n"));
    }
    if let Some(ut) = &block.user_tag {
        md.push_str(&format!("- User tag: {ut}\n"));
    }
    if !block.linked_block_ids.is_empty() {
        md.push_str(&format!(
            "- Links to blocks: {}\n",
            block.linked_block_ids.join(", ")
        ));
    }
    md.push('\n');

    if let Some(h) = &block.hypothesis {
        md.push_str(&format!("**Hypothesis:** {}\n\n", h.text));
    }
    if let Some(a) = &block.action {
        md.push_str(&format!("**Action:** {}\n\n", a.text));
    }
    if let Some(e) = &block.evidence {
        md.push_str(&format!("**Evidence:** {}\n\n", e.text));
        if let Some(src) = &e.source {
            md.push_str(&format!("_Source: {src}_\n\n"));
        }
    }
    if let Some(c) = &block.conclusion {
        md.push_str(&format!("**Conclusion:** {}\n\n", c.text));
        md.push_str(&format!("- Outcome: {}\n", c.outcome));
        if c.tag != "none" {
            md.push_str(&format!("- Decision tag: {}\n", c.tag));
        }
        if let Some(cl) = &c.confidence_level {
            md.push_str(&format!("- Confidence: {cl}\n"));
        }
        md.push('\n');
    }

    if block.incomplete {
        md.push_str("_Incomplete block; missing expected fields._\n\n");
    }
}

pub fn compile_from_path(db_path: &Path, workspace_id: &str) -> Result<String, String> {
    let store = GraphStore::open(db_path).map_err(|e| e.to_string())?;
    compile_workspace_summary_markdown(&store, workspace_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlayer_db::{GraphStore, SaveBlockInput};

    fn seed_two_blocks(store: &GraphStore, ws_id: &str) -> (String, String) {
        let a = store
            .save_block(SaveBlockInput {
                workspace_id: ws_id.to_string(),
                title: Some("First".into()),
                hypothesis_text: Some("Hypothesis A".into()),
                ..Default::default()
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let b = store
            .save_block(SaveBlockInput {
                workspace_id: ws_id.to_string(),
                title: Some("Second".into()),
                hypothesis_text: Some("Hypothesis B".into()),
                ..Default::default()
            })
            .unwrap();
        (a.id, b.id)
    }

    #[test]
    fn export_renders_block_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = GraphStore::open(&path).unwrap();
        let ws = store
            .create_workspace("Test", "Goal", "blank")
            .unwrap();
        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                hypothesis_text: Some("API may expose auth bypass".into()),
                ..Default::default()
            })
            .unwrap();
        let md = compile_workspace_summary_markdown(&store, &ws.id).unwrap();
        assert!(md.contains("Reasoning blocks"));
        assert!(md.contains("API may expose auth bypass"));
    }

    #[test]
    fn pr_export_only_selected_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = GraphStore::open(&path).unwrap();
        let ws = store
            .create_workspace("Auth fix", "Fix refresh bug", "blank")
            .unwrap();
        let (id_a, id_b) = seed_two_blocks(&store, &ws.id);

        let md = compile_pr_export_markdown(&store, &ws.id, &[id_b.clone()]).unwrap();
        assert!(md.contains("PR Reasoning:"));
        assert!(md.contains("Hypothesis:\n"));
        assert!(md.contains("Hypothesis B"));
        assert!(!md.contains("Hypothesis A"));
        assert!(!md.contains("**Hypothesis:**"));
        assert!(!md.contains(&id_b));

        let md_both = compile_pr_export_markdown(&store, &ws.id, &[id_a, id_b]).unwrap();
        assert!(md_both.contains("Hypothesis A"));
        assert!(md_both.contains("Hypothesis B"));
        let pos_a = md_both.find("Hypothesis A").unwrap();
        let pos_b = md_both.find("Hypothesis B").unwrap();
        assert!(pos_a < pos_b, "chronological order");
    }

    #[test]
    fn pr_export_empty_selection_errors() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("test.db")).unwrap();
        let ws = store
            .create_workspace("T", "G", "blank")
            .unwrap();
        assert!(compile_pr_export_markdown(&store, &ws.id, &[]).is_err());
    }

    #[test]
    fn pr_export_reviewer_summary_and_hygiene_footer() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("test.db")).unwrap();
        let ws = store
            .create_workspace("Auth fix", "Fix refresh bug", "blank")
            .unwrap();
        let block = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("Flow".into()),
                hypothesis_text: Some("Maybe broken".into()),
                evidence_text: Some("Logs show error".into()),
                conclusion_text: Some("Ship the fix after review".into()),
                conclusion_outcome: Some("confirmed".into()),
                belief_state: Some("open".into()),
                ..Default::default()
            })
            .unwrap();

        let md = compile_pr_export_markdown(&store, &ws.id, &[block.id]).unwrap();
        assert!(md.contains("Summary for reviewers:\nShip the fix after review"));
        assert!(md.contains("Hygiene: 1 of 1 exported block(s) still have unsettled belief"));
    }

    #[test]
    fn pr_export_sorts_by_created_at_not_updated_at() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("test.db")).unwrap();
        let ws = store
            .create_workspace("Order", "Test ordering", "blank")
            .unwrap();
        let first = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("First".into()),
                hypothesis_text: Some("First hypothesis".into()),
                ..Default::default()
            })
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let second = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("Second".into()),
                hypothesis_text: Some("Second hypothesis".into()),
                ..Default::default()
            })
            .unwrap();

        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                block_id: Some(first.id.clone()),
                hypothesis_text: Some("First hypothesis edited later".into()),
                ..Default::default()
            })
            .unwrap();

        let md = compile_pr_export_markdown(&store, &ws.id, &[second.id.clone(), first.id.clone()]).unwrap();
        let pos_first = md.find("First hypothesis edited later").unwrap();
        let pos_second = md.find("Second hypothesis").unwrap();
        assert!(pos_first < pos_second, "created_at order preserved after edit");
    }

    #[test]
    fn resolve_pr_block_ids_by_title() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("test.db")).unwrap();
        let ws = store
            .create_workspace("T", "G", "blank")
            .unwrap();
        let block = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("My block".into()),
                ..Default::default()
            })
            .unwrap();

        let ids = resolve_pr_block_ids(&store, &ws.id, &[], &["My block".into()]).unwrap();
        assert_eq!(ids, vec![block.id]);
    }
}
