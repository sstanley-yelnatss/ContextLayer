-- Soft-archive workspaces (hidden from default list; data retained).

ALTER TABLE workspaces ADD COLUMN archived_at TEXT;
