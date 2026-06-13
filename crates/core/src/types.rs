use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Hypothesis,
    Action,
    Evidence,
    Conclusion,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hypothesis => "hypothesis",
            Self::Action => "action",
            Self::Evidence => "evidence",
            Self::Conclusion => "conclusion",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Hypothesis => "Hypothesis",
            Self::Action => "Action",
            Self::Evidence => "Evidence",
            Self::Conclusion => "Conclusion",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "hypothesis" => Some(Self::Hypothesis),
            "action" => Some(Self::Action),
            "evidence" => Some(Self::Evidence),
            "conclusion" => Some(Self::Conclusion),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Workspace,
    Block,
    Hypothesis,
    Action,
    Evidence,
    Conclusion,
}

impl EntityType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Block => "block",
            Self::Hypothesis => "hypothesis",
            Self::Action => "action",
            Self::Evidence => "evidence",
            Self::Conclusion => "conclusion",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTemplate {
    Blank,
    SecurityHunt,
    ProductResearch,
}

impl WorkspaceTemplate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::SecurityHunt => "security_hunt",
            Self::ProductResearch => "product_research",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "blank" => Some(Self::Blank),
            "security_hunt" => Some(Self::SecurityHunt),
            "product_research" => Some(Self::ProductResearch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BeliefState {
    Open,
    LeaningTrue,
    LeaningFalse,
    Confirmed,
    Rejected,
}

impl BeliefState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::LeaningTrue => "leaning_true",
            Self::LeaningFalse => "leaning_false",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::LeaningTrue => "Leaning True",
            Self::LeaningFalse => "Leaning False",
            Self::Confirmed => "Confirmed",
            Self::Rejected => "Rejected",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "open" => Some(Self::Open),
            "leaning_true" | "testing" => Some(Self::LeaningTrue),
            "leaning_false" => Some(Self::LeaningFalse),
            "confirmed" | "supported" => Some(Self::Confirmed),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// Legacy alias — hypothesis.status and block.belief_state use BeliefState values.
pub type HypothesisStatus = BeliefState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockSystemTag {
    None,
    NeedsReview,
    RuledOut,
    Reportable,
    ReasoningDebt,
    Stale,
}

impl BlockSystemTag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::NeedsReview => "needs_review",
            Self::RuledOut => "ruled_out",
            Self::Reportable => "reportable",
            Self::ReasoningDebt => "reasoning_debt",
            Self::Stale => "stale",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::NeedsReview => "Needs Review",
            Self::RuledOut => "Ruled Out",
            Self::Reportable => "Reportable",
            Self::ReasoningDebt => "Reasoning Debt",
            Self::Stale => "Stale",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "needs_review" => Some(Self::NeedsReview),
            "ruled_out" => Some(Self::RuledOut),
            "reportable" => Some(Self::Reportable),
            "reasoning_debt" => Some(Self::ReasoningDebt),
            "stale" => Some(Self::Stale),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

impl ConfidenceLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionOutcome {
    Confirmed,
    Rejected,
    Uncertain,
    Refined,
}

impl ConclusionOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::Uncertain => "uncertain",
            Self::Refined => "refined",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "confirmed" => Some(Self::Confirmed),
            "rejected" => Some(Self::Rejected),
            "uncertain" => Some(Self::Uncertain),
            "refined" => Some(Self::Refined),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionTag {
    #[default]
    None,
    Pivot,
    Act,
    Ignore,
    Defer,
}

impl ConclusionTag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pivot => "pivot",
            Self::Act => "act",
            Self::Ignore => "ignore",
            Self::Defer => "defer",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "none" => Some(Self::None),
            "pivot" => Some(Self::Pivot),
            "act" => Some(Self::Act),
            "ignore" => Some(Self::Ignore),
            "defer" => Some(Self::Defer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub goal: String,
    pub template: WorkspaceTemplate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub workspace_id: String,
    pub text: String,
    pub status: BeliefState,
    pub block_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub workspace_id: String,
    pub text: String,
    pub block_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub workspace_id: String,
    pub text: String,
    pub source: Option<String>,
    pub block_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conclusion {
    pub id: String,
    pub workspace_id: String,
    pub text: String,
    pub outcome: ConclusionOutcome,
    pub tag: ConclusionTag,
    pub confidence: Option<f64>,
    pub confidence_level: Option<ConfidenceLevel>,
    pub block_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub superseded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub workspace_id: String,
    pub belief_state: BeliefState,
    pub system_tag: BlockSystemTag,
    pub user_tag: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockLink {
    pub id: String,
    pub workspace_id: String,
    pub from_block_id: String,
    pub to_block_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLink {
    pub id: String,
    pub workspace_id: String,
    pub from_type: NodeType,
    pub from_id: String,
    pub to_type: NodeType,
    pub to_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Created,
    Corrected,
    SoftDeleted,
    LinkAdded,
    LinkRemoved,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Corrected => "corrected",
            Self::SoftDeleted => "soft_deleted",
            Self::LinkAdded => "link_added",
            Self::LinkRemoved => "link_removed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkPair {
    pub from: NodeType,
    pub to: NodeType,
}

pub const ALLOWED_LINK_PAIRS: &[LinkPair] = &[
    LinkPair {
        from: NodeType::Hypothesis,
        to: NodeType::Action,
    },
    LinkPair {
        from: NodeType::Action,
        to: NodeType::Evidence,
    },
    LinkPair {
        from: NodeType::Conclusion,
        to: NodeType::Hypothesis,
    },
    LinkPair {
        from: NodeType::Conclusion,
        to: NodeType::Evidence,
    },
];

pub fn is_allowed_link(from: NodeType, to: NodeType) -> bool {
    ALLOWED_LINK_PAIRS
        .iter()
        .any(|pair| pair.from == from && pair.to == to)
}

pub fn derive_hypothesis_status(outcomes: &[ConclusionOutcome]) -> BeliefState {
    if outcomes.is_empty() {
        return BeliefState::Open;
    }
    if outcomes
        .iter()
        .any(|o| matches!(o, ConclusionOutcome::Rejected))
    {
        return BeliefState::Rejected;
    }
    if outcomes
        .iter()
        .any(|o| matches!(o, ConclusionOutcome::Confirmed))
    {
        return BeliefState::Confirmed;
    }
    if outcomes
        .iter()
        .any(|o| matches!(o, ConclusionOutcome::Uncertain | ConclusionOutcome::Refined))
    {
        return BeliefState::LeaningTrue;
    }
    BeliefState::Open
}
