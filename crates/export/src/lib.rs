//! Read-only markdown compile — block-first (Phase 1.1)

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
    md.push_str(&format!("# {}\n\n", workspace.name));
    md.push_str(&format!("**Goal:** {}\n\n", workspace.goal));
    md.push_str(&format!(
        "_Template: {}_\n\n",
        workspace.template.as_str()
    ));

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
}
