-- Timeline ordering indexes (PRD §5)

CREATE INDEX IF NOT EXISTS idx_hypotheses_created_at ON hypotheses(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_actions_created_at ON actions(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_evidence_created_at ON evidence(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_conclusions_created_at ON conclusions(workspace_id, created_at);
CREATE INDEX IF NOT EXISTS idx_events_entity ON events(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_versions_entity ON entity_versions(entity_type, entity_id);
