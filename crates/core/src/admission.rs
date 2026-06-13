use thiserror::Error;

use crate::types::{
    ConclusionOutcome, ConclusionTag, NodeType, WorkspaceTemplate, ALLOWED_LINK_PAIRS,
};

pub const CONCLUSION_LINK_ERROR: &str =
    "Conclusion requires at least one hypothesis and one evidence item.";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdmissionError {
    #[error("workspace name is required")]
    WorkspaceNameRequired,
    #[error("workspace goal is required")]
    WorkspaceGoalRequired,
    #[error("invalid workspace template")]
    InvalidTemplate,
    #[error("text is required")]
    TextRequired,
    #[error("invalid conclusion outcome")]
    InvalidOutcome,
    #[error("invalid conclusion tag")]
    InvalidTag,
    #[error("confidence must be between 0 and 1")]
    InvalidConfidence,
    #[error("{CONCLUSION_LINK_ERROR}")]
    ConclusionMissingLinks,
    #[error("disallowed link pair: {from} -> {to}")]
    DisallowedLink { from: String, to: String },
    #[error("content looks like unstructured notes; convert to hypothesis, action, evidence, or conclusion")]
    UnstructuredInput,
    #[error("hypothesis must be a falsifiable claim, not a bare fact or note")]
    InvalidHypothesis,
    #[error("action must describe something done, not observed knowledge")]
    InvalidAction,
    #[error("evidence must be observable; move interpretation to a conclusion")]
    InvalidEvidence,
}

pub fn validate_workspace(name: &str, goal: &str, template: &str) -> Result<WorkspaceTemplate, AdmissionError> {
    if name.trim().is_empty() {
        return Err(AdmissionError::WorkspaceNameRequired);
    }
    if goal.trim().is_empty() {
        return Err(AdmissionError::WorkspaceGoalRequired);
    }
    WorkspaceTemplate::parse(template).ok_or(AdmissionError::InvalidTemplate)
}

pub fn validate_conclusion_links(hypothesis_count: usize, evidence_count: usize) -> Result<(), AdmissionError> {
    if hypothesis_count == 0 || evidence_count == 0 {
        return Err(AdmissionError::ConclusionMissingLinks);
    }
    Ok(())
}

pub fn validate_link_pair(from: NodeType, to: NodeType) -> Result<(), AdmissionError> {
    if ALLOWED_LINK_PAIRS
        .iter()
        .any(|pair| pair.from == from && pair.to == to)
    {
        Ok(())
    } else {
        Err(AdmissionError::DisallowedLink {
            from: from.as_str().into(),
            to: to.as_str().into(),
        })
    }
}

pub fn validate_hypothesis_text(text: &str) -> Result<(), AdmissionError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AdmissionError::TextRequired);
    }
    if looks_like_unstructured(trimmed) {
        return Err(AdmissionError::UnstructuredInput);
    }
    if trimmed.len() < 8 {
        return Err(AdmissionError::InvalidHypothesis);
    }
    Ok(())
}

pub fn validate_action_text(text: &str) -> Result<(), AdmissionError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AdmissionError::TextRequired);
    }
    if looks_like_unstructured(trimmed) {
        return Err(AdmissionError::UnstructuredInput);
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("todo:") || lower.starts_with("- [ ]") {
        return Err(AdmissionError::InvalidAction);
    }
    Ok(())
}

pub fn validate_evidence_text(text: &str) -> Result<(), AdmissionError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(AdmissionError::TextRequired);
    }
    if looks_like_unstructured(trimmed) {
        return Err(AdmissionError::UnstructuredInput);
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("therefore") || lower.starts_with("this means") {
        return Err(AdmissionError::InvalidEvidence);
    }
    Ok(())
}

pub fn validate_conclusion_fields(
    text: &str,
    outcome: &str,
    tag: &str,
    confidence: Option<f64>,
) -> Result<(ConclusionOutcome, ConclusionTag), AdmissionError> {
    if text.trim().is_empty() {
        return Err(AdmissionError::TextRequired);
    }
    let outcome = ConclusionOutcome::parse(outcome).ok_or(AdmissionError::InvalidOutcome)?;
    let tag = ConclusionTag::parse(tag).ok_or(AdmissionError::InvalidTag)?;
    if let Some(c) = confidence {
        if !(0.0..=1.0).contains(&c) {
            return Err(AdmissionError::InvalidConfidence);
        }
    }
    Ok((outcome, tag))
}

fn looks_like_unstructured(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.starts_with("note:")
        || lower.starts_with("summary:")
        || lower.starts_with("chat log")
        || lower.contains("lorem ipsum")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conclusion_requires_hypothesis_and_evidence() {
        assert!(validate_conclusion_links(1, 1).is_ok());
        assert_eq!(
            validate_conclusion_links(0, 1),
            Err(AdmissionError::ConclusionMissingLinks)
        );
        assert_eq!(
            validate_conclusion_links(1, 0),
            Err(AdmissionError::ConclusionMissingLinks)
        );
        assert_eq!(
            AdmissionError::ConclusionMissingLinks.to_string(),
            CONCLUSION_LINK_ERROR
        );
    }

    #[test]
    fn rejects_disallowed_links() {
        assert!(validate_link_pair(NodeType::Hypothesis, NodeType::Evidence).is_err());
    }
}
