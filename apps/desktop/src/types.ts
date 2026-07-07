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

export type WorkspaceTemplate =
  | "blank"
  | "agent_devops"
  | "security_hunt"
  | "product_research"
  | "decision_strategy";

export interface Workspace {
  id: string;
  name: string;
  goal: string;
  template: WorkspaceTemplate | string;
  created_at: string;
  updated_at: string;
  archived_at?: string | null;
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
  title: string;
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

/** Labels for all templates (including legacy workspaces not shown in create dropdown). */
const TEMPLATE_LABELS: Record<string, string> = {
  agent_devops: "Agent DevOps",
  agent_dev_ops: "Agent DevOps",
  blank: "Blank",
  security_hunt: "Penetration Testing",
  product_research: "Product Research",
  decision_strategy: "Decision & Strategy",
};

/** Templates offered when creating a new workspace. */
export const CREATE_WORKSPACE_TEMPLATES: { value: WorkspaceTemplate; label: string }[] = [
  { value: "agent_devops", label: "Agent DevOps" },
  { value: "blank", label: "Blank" },
  { value: "product_research", label: "Product Research" },
  { value: "decision_strategy", label: "Decision & Strategy" },
];

/** @deprecated use CREATE_WORKSPACE_TEMPLATES for create flows */
export const TEMPLATES = CREATE_WORKSPACE_TEMPLATES;

export function normalizeTemplate(template: WorkspaceTemplate | string): WorkspaceTemplate {
  if (template === "agent_dev_ops") return "agent_devops";
  return template as WorkspaceTemplate;
}

/** First reasoning field label — Assumption for Agent DevOps, Hypothesis for security hunt. */
export function hypothesisFieldLabel(template: WorkspaceTemplate | string): string {
  const t = normalizeTemplate(template);
  if (t === "agent_devops") return "Assumption";
  if (t === "security_hunt") return "Hypothesis";
  return "Hypothesis";
}

export function templateLabel(template: WorkspaceTemplate | string): string {
  const key = normalizeTemplate(template);
  return TEMPLATE_LABELS[key] ?? key;
}

export function placeholdersForTemplate(template: WorkspaceTemplate | string) {
  const key = normalizeTemplate(template);
  return PLACEHOLDERS[key] ?? PLACEHOLDERS.blank;
}

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

export const HYGIENE_CATEGORIES = [
  { id: "orphans", label: "Orphans", key: "orphans" as const },
  { id: "stale", label: "Stale", key: "stale" as const },
  { id: "still_open", label: "Still Open", key: "still_open" as const },
  { id: "dead_ends", label: "Dead Ends", key: "dead_ends" as const },
  { id: "decisions", label: "Decisions", key: "decisions" as const },
] as const;

export function hygieneCategoryLabel(key: string): string {
  const cat = HYGIENE_CATEGORIES.find((c) => c.key === key);
  return cat?.label ?? key;
}

export const PLACEHOLDERS: Record<
  WorkspaceTemplate,
  {
    goal: string;
    title: string;
    userTag: string;
    hypothesis: string;
    action: string;
    evidence: string;
    conclusion: string;
  }
> = {
  agent_devops: {
    goal: "What change or PR are you reasoning through?",
    title: "e.g. Refresh token rotation fix",
    userTag: "e.g. auth, api, regression",
    hypothesis: "We assume this approach works because…",
    action: "Implemented / tested / reviewed diff…",
    evidence: "Test output, log line, reviewer note…",
    conclusion: "Ship / revise / need more validation",
  },
  blank: {
    goal: "What question are you trying to answer in this workspace?",
    title: "Short label for this step",
    userTag: "e.g. pricing, onboarding, open-question",
    hypothesis: "What uncertain claim are you testing?",
    action: "What did you do to test it?",
    evidence: "What did you observe?",
    conclusion: "What did you conclude from the evidence?",
  },
  security_hunt: {
    goal: "What scope, asset, or finding are you hunting?",
    title: "e.g. IDOR on /api/user",
    userTag: "e.g. idor, auth-bypass",
    hypothesis: "This target may be vulnerable to…",
    action: "Ran scan / tested endpoint / reviewed config…",
    evidence: "Response code, header, log line, screenshot note…",
    conclusion: "Finding confirmed / ruled out / needs more testing",
  },
  product_research: {
    goal: "What product or user question are you investigating?",
    title: "e.g. Willingness to pay for feature X",
    userTag: "e.g. interview, competitor, persona-a",
    hypothesis: "Users may need X because…",
    action: "Interviewed / surveyed / reviewed competitor…",
    evidence: "Quote, metric, observation from session…",
    conclusion: "Validated / invalidated / pivot recommendation",
  },
  decision_strategy: {
    goal: "What decision are you trying to make, and what would change your mind?",
    title: "e.g. Build vs buy for analytics",
    userTag: "e.g. option-a, risk, stakeholder",
    hypothesis: "Option A may be better because…",
    action: "Compared costs / ran pilot / consulted advisor…",
    evidence: "Data point, constraint, stakeholder input…",
    conclusion: "Choose A / defer / need more info",
  },
};
