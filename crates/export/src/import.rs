//! Transcript import spike (O1) — heuristic session → blocks

use contextlayer_db::{GraphStore, SaveBlockInput};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportSessionResult {
    pub workspace_id: String,
    pub blocks_created: u32,
}

/// Paste a Cursor/chat transcript → new workspace with draft blocks (`needs_review`).
pub fn import_transcript(
    store: &GraphStore,
    workspace_name: &str,
    goal: &str,
    transcript: &str,
) -> Result<ImportSessionResult, String> {
    let name = workspace_name.trim();
    if name.is_empty() {
        return Err("workspace_name is required".into());
    }
    let goal = if goal.trim().is_empty() {
        "Imported from session transcript"
    } else {
        goal.trim()
    };

    let ws = store
        .create_workspace(name, goal, "blank")
        .map_err(|e| e.to_string())?;

    let chunks = split_transcript(transcript);
    if chunks.is_empty() {
        return Err("Transcript is empty or could not be split into segments".into());
    }

    let mut blocks_created = 0u32;
    for (i, chunk) in chunks.iter().enumerate() {
        let text = chunk.trim();
        if text.is_empty() {
            continue;
        }
        let first_line = text.lines().next().unwrap_or("Segment").trim();
        let title = truncate(first_line, 72);
        let (hypothesis_text, evidence_text, action_text) = classify_segment(text);

        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some(format!("Import {}: {}", i + 1, title)),
                hypothesis_text,
                action_text,
                evidence_text,
                system_tag: Some("needs_review".into()),
                belief_state: Some("open".into()),
                ..Default::default()
            })
            .map_err(|e| e.to_string())?;
        blocks_created += 1;
    }

    if blocks_created == 0 {
        return Err("No blocks created from transcript".into());
    }

    Ok(ImportSessionResult {
        workspace_id: ws.id,
        blocks_created,
    })
}

fn truncate(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

fn split_transcript(transcript: &str) -> Vec<String> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    for sep in [
        "\n\nUser:",
        "\nUser:",
        "\n\nHuman:",
        "\nHuman:",
        "\n\nAssistant:",
        "\nAssistant:",
        "\n\nCursor:",
        "\nCursor:",
    ] {
        if trimmed.contains(sep) {
            let parts: Vec<String> = trimmed
                .split(sep)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if parts.len() >= 2 {
                return parts;
            }
        }
    }

    trimmed
        .split("\n\n")
        .map(str::trim)
        .filter(|s| !s.is_empty() && s.len() > 20)
        .take(12)
        .map(|s| s.to_string())
        .collect()
}

fn classify_segment(text: &str) -> (Option<String>, Option<String>, Option<String>) {
    let lower = text.to_lowercase();
    if lower.contains("```") || lower.contains("error:") || lower.contains("exit code") {
        return (None, Some(text.to_string()), None);
    }
    if lower.starts_with("ran ")
        || lower.contains("npm ")
        || lower.contains("cargo ")
        || lower.contains("curl ")
    {
        return (None, None, Some(text.to_string()));
    }
    (Some(text.to_string()), None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextlayer_db::GraphStore;

    #[test]
    fn import_paragraphs_creates_blocks() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let transcript = "Maybe the bug is in auth middleware.\n\n\
            We should check token refresh first.\n\n\
            ```\nerror: 401 Unauthorized\n```";

        let r = import_transcript(
            &store,
            "Session import",
            "Debug auth",
            transcript,
        )
        .unwrap();
        assert!(r.blocks_created >= 3);
        let blocks = store.fetch_blocks(&r.workspace_id, false).unwrap();
        assert!(blocks.iter().all(|b| b.system_tag == "needs_review"));
    }
}
