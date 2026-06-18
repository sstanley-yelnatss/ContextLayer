-- Human-readable block titles for natural-language reference within a workspace.

ALTER TABLE blocks ADD COLUMN title TEXT;

-- Backfill from hypothesis text, then action, then a short id fallback.
UPDATE blocks SET title = (
  SELECT substr(h.text, 1, 120)
  FROM hypotheses h
  WHERE h.block_id = blocks.id AND h.deleted_at IS NULL
  LIMIT 1
) WHERE title IS NULL;

UPDATE blocks SET title = (
  SELECT substr(a.text, 1, 120)
  FROM actions a
  WHERE a.block_id = blocks.id AND a.deleted_at IS NULL
  LIMIT 1
) WHERE title IS NULL;

UPDATE blocks SET title = 'Block ' || substr(id, 1, 8) WHERE title IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_blocks_workspace_title
  ON blocks(workspace_id, lower(title))
  WHERE deleted_at IS NULL AND title IS NOT NULL;
