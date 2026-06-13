-- Phase 1.1: Blocks as primary UX unit; belief states; block links.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS blocks (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  belief_state TEXT NOT NULL DEFAULT 'open',
  system_tag TEXT NOT NULL DEFAULT 'none',
  user_tag TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS block_links (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  from_block_id TEXT NOT NULL REFERENCES blocks(id),
  to_block_id TEXT NOT NULL REFERENCES blocks(id),
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS belief_state_history (
  id TEXT PRIMARY KEY NOT NULL,
  block_id TEXT NOT NULL REFERENCES blocks(id),
  from_state TEXT,
  to_state TEXT NOT NULL,
  note TEXT,
  created_at TEXT NOT NULL
);

ALTER TABLE hypotheses ADD COLUMN block_id TEXT REFERENCES blocks(id);
ALTER TABLE actions ADD COLUMN block_id TEXT REFERENCES blocks(id);
ALTER TABLE evidence ADD COLUMN block_id TEXT REFERENCES blocks(id);
ALTER TABLE conclusions ADD COLUMN block_id TEXT REFERENCES blocks(id);
ALTER TABLE conclusions ADD COLUMN confidence_level TEXT;

CREATE INDEX IF NOT EXISTS idx_blocks_workspace ON blocks(workspace_id);
CREATE INDEX IF NOT EXISTS idx_blocks_updated ON blocks(workspace_id, updated_at);
CREATE INDEX IF NOT EXISTS idx_block_links_from ON block_links(from_block_id);
CREATE INDEX IF NOT EXISTS idx_block_links_to ON block_links(to_block_id);
CREATE INDEX IF NOT EXISTS idx_hypotheses_block ON hypotheses(block_id);
CREATE INDEX IF NOT EXISTS idx_actions_block ON actions(block_id);
CREATE INDEX IF NOT EXISTS idx_evidence_block ON evidence(block_id);
CREATE INDEX IF NOT EXISTS idx_conclusions_block ON conclusions(block_id);

-- Map legacy hypothesis statuses to belief states
UPDATE hypotheses SET status = 'leaning_true' WHERE status = 'testing';
UPDATE hypotheses SET status = 'confirmed' WHERE status = 'supported';
