//! ContextLayer core domain — four primitives only. Constraint is not first-class.

pub mod admission;
pub mod types;

pub use admission::{
    validate_action_text, validate_conclusion_fields, validate_conclusion_links,
    validate_evidence_text, validate_hypothesis_text, validate_link_pair, validate_workspace,
    AdmissionError, CONCLUSION_LINK_ERROR,
};
pub use types::{
    derive_hypothesis_status, is_allowed_link, Action, BeliefState, Block, BlockLink,
    BlockSystemTag, Conclusion, ConclusionOutcome, ConclusionTag, ConfidenceLevel, EntityType,
    EventKind, Evidence, Hypothesis, HypothesisStatus, NodeLink, NodeType, Workspace,
    WorkspaceTemplate, ALLOWED_LINK_PAIRS,
};
