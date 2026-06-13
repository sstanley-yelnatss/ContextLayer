-- ContextLayer Phase 1 schema. Four node types only — no notes, no constraints table.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS workspaces (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  goal TEXT NOT NULL,
  template TEXT NOT NULL DEFAULT 'blank',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS hypotheses (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  text TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'open',
  created_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS actions (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  text TEXT NOT NULL,
  created_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS evidence (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  text TEXT NOT NULL,
  source TEXT,
  created_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS conclusions (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  text TEXT NOT NULL,
  outcome TEXT NOT NULL,
  tag TEXT NOT NULL DEFAULT 'none',
  confidence REAL,
  created_at TEXT NOT NULL,
  superseded_at TEXT
);

CREATE TABLE IF NOT EXISTS node_links (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  from_type TEXT NOT NULL,
  from_id TEXT NOT NULL,
  to_type TEXT NOT NULL,
  to_id TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS entity_versions (
  id TEXT PRIMARY KEY NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  body_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
  id TEXT PRIMARY KEY NOT NULL,
  type TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_hypotheses_workspace ON hypotheses(workspace_id);
CREATE INDEX IF NOT EXISTS idx_actions_workspace ON actions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_evidence_workspace ON evidence(workspace_id);
CREATE INDEX IF NOT EXISTS idx_conclusions_workspace ON conclusions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_node_links_from ON node_links(from_type, from_id);
CREATE INDEX IF NOT EXISTS idx_node_links_to ON node_links(to_type, to_id);
