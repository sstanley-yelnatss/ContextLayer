export interface HygieneItem {
  block_id: string;
  category: string;
  message: string;
  preview: string;
  days_open?: number;
}

export interface WorkspaceHealthSummary {
  total_blocks: number;
  belief_open: number;
  belief_leading: number;
  belief_confirmed: number;
  belief_rejected: number;
  needs_review: number;
  reasoning_debt: number;
  stale: number;
  orphans: number;
  dead_ends: number;
  still_open: number;
  decisions: number;
}

export interface WorkspaceHygieneReport {
  summary: WorkspaceHealthSummary;
  orphans: HygieneItem[];
  stale: HygieneItem[];
  still_open: HygieneItem[];
  dead_ends: HygieneItem[];
  decisions: HygieneItem[];
}

export type WorkspaceTemplate = "blank" | "security_hunt" | "product_research";

export interface Workspace {
  id: string;
  name: string;
  goal: string;
  template: WorkspaceTemplate;
  created_at: string;
  updated_at: string;
}

export type BeliefState =
  | "open"
  | "leaning_true"
  | "leaning_false"
  | "confirmed"
  | "rejected";

export type BlockSystemTag =
  | "none"
  | "needs_review"
  | "ruled_out"
  | "reportable"
  | "reasoning_debt"
  | "stale";

export type ConfidenceLevel = "low" | "medium" | "high";

export interface BlockField {
  id: string;
  text: string;
  source?: string;
}

export interface BlockConclusionField {
  id: string;
  text: string;
  outcome: string;
  tag: string;
  confidence_level?: string;
}

export interface BlockEntry {
  id: string;
  workspace_id: string;
  belief_state: BeliefState;
  system_tag: BlockSystemTag;
  user_tag?: string;
  created_at: string;
  updated_at: string;
  hypothesis?: BlockField;
  action?: BlockField;
  evidence?: BlockField;
  conclusion?: BlockConclusionField;
  linked_block_ids: string[];
  incomplete: boolean;
}

export const TEMPLATES: { value: WorkspaceTemplate; label: string }[] = [
  { value: "blank", label: "Blank" },
  { value: "security_hunt", label: "Security hunt" },
  { value: "product_research", label: "Product research" },
];

export const BELIEF_STATES: { value: BeliefState; label: string }[] = [
  { value: "open", label: "Open" },
  { value: "leaning_true", label: "Leaning True" },
  { value: "leaning_false", label: "Leaning False" },
  { value: "confirmed", label: "Confirmed" },
  { value: "rejected", label: "Rejected" },
];

export const SYSTEM_TAGS: { value: BlockSystemTag; label: string }[] = [
  { value: "none", label: "None" },
  { value: "needs_review", label: "Needs Review" },
  { value: "ruled_out", label: "Ruled Out" },
  { value: "reportable", label: "Reportable" },
  { value: "reasoning_debt", label: "Reasoning Debt" },
  { value: "stale", label: "Stale" },
];

export const CONFIDENCE_LEVELS: { value: ConfidenceLevel; label: string }[] = [
  { value: "low", label: "Low" },
  { value: "medium", label: "Medium" },
  { value: "high", label: "High" },
];

export function beliefStateLabel(state: BeliefState): string {
  return BELIEF_STATES.find((s) => s.value === state)?.label ?? state;
}

export function systemTagLabel(tag: BlockSystemTag): string {
  return SYSTEM_TAGS.find((t) => t.value === tag)?.label ?? tag;
}

export const PLACEHOLDERS: Record<
  WorkspaceTemplate,
  { hypothesis: string; action: string; evidence: string; conclusion: string }
> = {
  blank: {
    hypothesis: "What uncertain claim are you testing?",
    action: "What did you do to test it?",
    evidence: "What did you observe?",
    conclusion: "What did you conclude from the evidence?",
  },
  security_hunt: {
    hypothesis: "This target may be vulnerable to…",
    action: "Ran scan / tested endpoint / reviewed config…",
    evidence: "Response code, header, log line, screenshot note…",
    conclusion: "Finding confirmed / ruled out / needs more testing",
  },
  product_research: {
    hypothesis: "Users may need X because…",
    action: "Interviewed / surveyed / reviewed competitor…",
    evidence: "Quote, metric, observation from session…",
    conclusion: "Validated / invalidated / pivot recommendation",
  },
};
