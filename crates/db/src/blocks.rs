//! Block CRUD — primary UX unit (Phase 1.1)

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use contextlayer_core::{
    validate_action_text, validate_conclusion_fields, validate_conclusion_links,
    validate_evidence_text, validate_hypothesis_text, BeliefState, BlockSystemTag,
    ConfidenceLevel, ConclusionOutcome, ConclusionTag, EntityType, EventKind, NodeType,
};

use crate::graph::GraphStore;
use crate::DbError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockField {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockConclusionField {
    pub id: String,
    pub text: String,
    pub outcome: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_level: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockEntry {
    pub id: String,
    pub workspace_id: String,
    pub title: String,
    pub belief_state: String,
    pub system_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_tag: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hypothesis: Option<BlockField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<BlockField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<BlockField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion: Option<BlockConclusionField>,
    pub linked_block_ids: Vec<String>,
    pub incomplete: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SaveBlockInput {
    pub workspace_id: String,
    pub block_id: Option<String>,
    /// Resolve block by title (case-insensitive) within workspace when block_id is absent.
    pub block_title: Option<String>,
    pub title: Option<String>,
    pub hypothesis_text: Option<String>,
    pub action_text: Option<String>,
    pub evidence_text: Option<String>,
    pub evidence_source: Option<String>,
    pub conclusion_text: Option<String>,
    pub conclusion_outcome: Option<String>,
    pub conclusion_tag: Option<String>,
    pub confidence_level: Option<String>,
    pub belief_state: Option<String>,
    pub system_tag: Option<String>,
    pub user_tag: Option<String>,
    pub link_to_block_ids: Vec<String>,
}

impl GraphStore {
    pub fn migrate_legacy_nodes_to_blocks(&self) -> Result<u32, DbError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM blocks", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(0);
        }

        let mut migrated = 0u32;
        for (table, node_type) in [
            ("hypotheses", "hypothesis"),
            ("actions", "action"),
            ("evidence", "evidence"),
            ("conclusions", "conclusion"),
        ] {
            let sql = format!(
                "SELECT id, workspace_id, created_at FROM {table} WHERE block_id IS NULL"
            );
            let mut stmt = self.conn.prepare(&sql)?;
            let rows: Vec<(String, String, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .filter_map(Result::ok)
                .collect();

            for (node_id, workspace_id, created_at) in rows {
                let block_id = Uuid::new_v4().to_string();
                self.conn.execute(
                    "INSERT INTO blocks (id, workspace_id, title, belief_state, system_tag, created_at, updated_at)
                     VALUES (?1, ?2, ?3, 'open', 'none', ?4, ?4)",
                    params![block_id, workspace_id, format!("Block {}", &block_id[..8.min(block_id.len())]), created_at],
                )?;
                self.conn.execute(
                    &format!("UPDATE {table} SET block_id = ?1 WHERE id = ?2"),
                    params![block_id, node_id],
                )?;
                let _ = node_type;
                migrated += 1;
            }
        }
        Ok(migrated)
    }

    pub fn save_block(&self, input: SaveBlockInput) -> Result<BlockEntry, DbError> {
        self.ensure_workspace(&input.workspace_id)?;

        let existing = if let Some(block_id) = self.resolve_block_ref(&input)? {
            Some(self.get_block(&block_id)?)
        } else {
            None
        };

        let is_update = existing.is_some();

        let evidence_text_provided = input.evidence_text.is_some();
        let h_text = merge_text_input(input.hypothesis_text, existing.as_ref().and_then(|b| b.hypothesis.as_ref().map(|h| h.text.clone())));
        let a_text = merge_text_input(input.action_text, existing.as_ref().and_then(|b| b.action.as_ref().map(|a| a.text.clone())));
        let e_text = merge_text_input(input.evidence_text, existing.as_ref().and_then(|b| b.evidence.as_ref().map(|e| e.text.clone())));
        let c_text = merge_text_input(input.conclusion_text, existing.as_ref().and_then(|b| b.conclusion.as_ref().map(|c| c.text.clone())));

        if c_text.is_some() && (h_text.is_none() || e_text.is_none()) {
            return Err(DbError::Admission(
                contextlayer_core::CONCLUSION_LINK_ERROR.to_string(),
            ));
        }

        let title = if let Some(explicit) = trim_opt(input.title) {
            explicit
        } else if let Some(ref ex) = existing {
            ex.title.clone()
        } else {
            derive_title(&h_text, &a_text, &e_text, &c_text)
        };
        self.ensure_unique_title(&input.workspace_id, &title, existing.as_ref().map(|b| b.id.as_str()))?;

        let belief = input
            .belief_state
            .as_deref()
            .and_then(BeliefState::parse)
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|b| BeliefState::parse(&b.belief_state))
            })
            .unwrap_or(BeliefState::Open);
        let system_tag = input
            .system_tag
            .as_deref()
            .and_then(BlockSystemTag::parse)
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|b| BlockSystemTag::parse(&b.system_tag))
            })
            .unwrap_or(BlockSystemTag::None);
        let user_tag = match input.user_tag {
            Some(ref s) => {
                let t = s.trim();
                if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                }
            }
            None => existing.as_ref().and_then(|b| b.user_tag.clone()),
        };

        let e_source = merge_evidence_source(
            evidence_text_provided,
            input.evidence_source,
            existing.as_ref().and_then(|b| b.evidence.as_ref()),
        );

        let c_outcome = input
            .conclusion_outcome
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|b| b.conclusion.as_ref().map(|c| c.outcome.clone()))
            })
            .unwrap_or_else(|| "uncertain".to_string());
        let c_tag = input
            .conclusion_tag
            .or_else(|| {
                existing
                    .as_ref()
                    .and_then(|b| b.conclusion.as_ref().map(|c| c.tag.clone()))
            })
            .unwrap_or_else(|| "none".to_string());
        let conf_level = input.confidence_level.or_else(|| {
            existing
                .as_ref()
                .and_then(|b| b.conclusion.as_ref().and_then(|c| c.confidence_level.clone()))
        });

        let now = Utc::now();
        let block_id = if let Some(ref ex) = existing {
            ex.id.clone()
        } else {
            Uuid::new_v4().to_string()
        };

        if is_update {
            self.update_block_metadata(
                &block_id,
                &title,
                belief,
                system_tag,
                user_tag.as_deref(),
                now,
            )?;
            self.clear_block_nodes(&block_id)?;
        } else {
            self.conn.execute(
                "INSERT INTO blocks (id, workspace_id, title, belief_state, system_tag, user_tag, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    block_id,
                    input.workspace_id,
                    title,
                    belief.as_str(),
                    system_tag.as_str(),
                    user_tag,
                    now.to_rfc3339(),
                    now.to_rfc3339()
                ],
            )?;
            self.emit_block_event(EventKind::Created, &block_id, serde_json::json!({}))?;
        }

        let mut hypothesis_id: Option<String> = None;
        let mut action_id: Option<String> = None;
        let mut evidence_id: Option<String> = None;

        if let Some(text) = h_text {
            validate_hypothesis_text(&text).map_err(|e| DbError::Admission(e.to_string()))?;
            let id = self.insert_hypothesis_in_block(&input.workspace_id, &block_id, &text, belief)?;
            hypothesis_id = Some(id);
        }

        if let Some(text) = a_text {
            validate_action_text(&text).map_err(|e| DbError::Admission(e.to_string()))?;
            let id = self.insert_action_in_block(&input.workspace_id, &block_id, &text)?;
            action_id = Some(id);
        }

        if let Some(text) = e_text {
            validate_evidence_text(&text).map_err(|e| DbError::Admission(e.to_string()))?;
            let src = e_source.as_deref().filter(|s| !s.trim().is_empty());
            let id = self.insert_evidence_in_block(&input.workspace_id, &block_id, &text, src)?;
            evidence_id = Some(id);
        }

        if let (Some(h_id), Some(a_id)) = (&hypothesis_id, &action_id) {
            let _ = self.add_link_internal(
                &input.workspace_id,
                NodeType::Hypothesis,
                h_id,
                NodeType::Action,
                a_id,
            );
        }
        if let (Some(a_id), Some(e_id)) = (&action_id, &evidence_id) {
            let _ = self.add_link_internal(
                &input.workspace_id,
                NodeType::Action,
                a_id,
                NodeType::Evidence,
                e_id,
            );
        }

        if let Some(text) = c_text {
            let outcome = c_outcome.as_str();
            let tag = c_tag.as_str();
            let conf_level_ref = conf_level.as_deref();
            let h_ids: Vec<String> = hypothesis_id.clone().into_iter().collect();
            let e_ids: Vec<String> = evidence_id.clone().into_iter().collect();
            validate_conclusion_links(h_ids.len(), e_ids.len())
                .map_err(|e| DbError::Admission(e.to_string()))?;
            let (outcome_enum, tag_enum) = validate_conclusion_fields(&text, outcome, tag, None)
                .map_err(|e| DbError::Admission(e.to_string()))?;
            let conf_parsed = conf_level_ref.and_then(ConfidenceLevel::parse);
            let c_id = self.insert_conclusion_in_block(
                &input.workspace_id,
                &block_id,
                &text,
                outcome_enum,
                tag_enum,
                conf_parsed,
            )?;
            if let Some(h_id) = &hypothesis_id {
                let _ = self.add_link_internal(
                    &input.workspace_id,
                    NodeType::Conclusion,
                    &c_id,
                    NodeType::Hypothesis,
                    h_id,
                );
            }
            if let Some(e_id) = &evidence_id {
                let _ = self.add_link_internal(
                    &input.workspace_id,
                    NodeType::Conclusion,
                    &c_id,
                    NodeType::Evidence,
                    e_id,
                );
            }
            if let Some(h_id) = &hypothesis_id {
                let _ = self.refresh_hypothesis_statuses_for(&input.workspace_id, &[h_id.clone()]);
            }
        }

        for to_id in &input.link_to_block_ids {
            if to_id != &block_id {
                let _ = self.add_block_link(&input.workspace_id, &block_id, to_id);
            }
        }

        self.get_block(&block_id)
    }

    /// Resolve block id from block_id or block_title (case-insensitive, exact match).
    pub fn resolve_block_ref(&self, input: &SaveBlockInput) -> Result<Option<String>, DbError> {
        if let Some(ref id) = input.block_id {
            if !id.trim().is_empty() {
                self.ensure_block_in_workspace(&input.workspace_id, id)?;
                return Ok(Some(id.clone()));
            }
        }
        if let Some(ref title) = input.block_title {
            let t = title.trim();
            if !t.is_empty() {
                return self.find_block_id_by_title(&input.workspace_id, t).map(Some);
            }
        }
        Ok(None)
    }

    pub fn find_block_id_by_title(
        &self,
        workspace_id: &str,
        title: &str,
    ) -> Result<String, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM blocks
             WHERE workspace_id = ?1 AND deleted_at IS NULL AND lower(title) = lower(?2)",
        )?;
        let ids: Vec<String> = stmt
            .query_map(params![workspace_id, title.trim()], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        match ids.len() {
            0 => Err(DbError::NotFound),
            1 => Ok(ids[0].clone()),
            _ => Err(DbError::InvalidInput(format!(
                "ambiguous block title \"{title}\" — use block_id instead"
            ))),
        }
    }

    pub fn list_block_titles(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<(String, String)>, DbError> {
        let blocks = self.fetch_blocks(workspace_id, false)?;
        Ok(blocks.into_iter().map(|b| (b.id, b.title)).collect())
    }

    pub fn fetch_blocks(
        &self,
        workspace_id: &str,
        ascending: bool,
    ) -> Result<Vec<BlockEntry>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, title, belief_state, system_tag, user_tag, created_at, updated_at
             FROM blocks WHERE workspace_id = ?1 AND deleted_at IS NULL",
        )?;
        let mut blocks: Vec<BlockEntry> = stmt
            .query_map([workspace_id], |row| {
                Ok(BlockEntry {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    title: row.get(2)?,
                    belief_state: row.get(3)?,
                    system_tag: row.get(4)?,
                    user_tag: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    hypothesis: None,
                    action: None,
                    evidence: None,
                    conclusion: None,
                    linked_block_ids: vec![],
                    incomplete: false,
                })
            })?
            .filter_map(Result::ok)
            .collect();

        for block in &mut blocks {
            block.hypothesis = self.load_block_hypothesis(&block.id)?;
            block.action = self.load_block_action(&block.id)?;
            block.evidence = self.load_block_evidence(&block.id)?;
            block.conclusion = self.load_block_conclusion(&block.id)?;
            block.linked_block_ids = self.list_block_link_targets(&block.id)?;
            block.incomplete = block_has_gaps(block);
        }

        blocks.sort_by(|a, b| {
            let ord = a.updated_at.cmp(&b.updated_at);
            if ascending {
                ord
            } else {
                ord.reverse()
            }
        });

        Ok(blocks)
    }

    pub fn list_blocks_for_picker(&self, workspace_id: &str) -> Result<Vec<(String, String)>, DbError> {
        let blocks = self.fetch_blocks(workspace_id, false)?;
        Ok(blocks.into_iter().map(|b| (b.id, b.title)).collect())
    }

    pub fn add_block_link(
        &self,
        workspace_id: &str,
        from_block_id: &str,
        to_block_id: &str,
    ) -> Result<contextlayer_core::BlockLink, DbError> {
        if from_block_id == to_block_id {
            return Err(DbError::InvalidInput("cannot link block to itself".into()));
        }
        self.ensure_block_in_workspace(workspace_id, from_block_id)?;
        self.ensure_block_in_workspace(workspace_id, to_block_id)?;

        let exists: Option<i64> = self
            .conn
            .query_row(
                "SELECT 1 FROM block_links WHERE from_block_id = ?1 AND to_block_id = ?2",
                params![from_block_id, to_block_id],
                |row| row.get(0),
            )
            .optional()?;
        if exists.is_some() {
            return Err(DbError::InvalidInput("block link already exists".into()));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO block_links (id, workspace_id, from_block_id, to_block_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                workspace_id,
                from_block_id,
                to_block_id,
                now.to_rfc3339()
            ],
        )?;
        Ok(contextlayer_core::BlockLink {
            id,
            workspace_id: workspace_id.to_string(),
            from_block_id: from_block_id.to_string(),
            to_block_id: to_block_id.to_string(),
            created_at: now,
        })
    }

    pub fn list_block_links(
        &self,
        workspace_id: &str,
    ) -> Result<Vec<contextlayer_core::BlockLink>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, from_block_id, to_block_id, created_at
             FROM block_links WHERE workspace_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([workspace_id], |row| {
            Ok(contextlayer_core::BlockLink {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                from_block_id: row.get(2)?,
                to_block_id: row.get(3)?,
                created_at: parse_ts(&row.get::<_, String>(4)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn remove_block_link(&self, link_id: &str) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM block_links WHERE id = ?1", [link_id])?;
        if rows == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    pub fn get_block_in_workspace(
        &self,
        workspace_id: &str,
        block_id: Option<String>,
        block_title: Option<String>,
    ) -> Result<BlockEntry, DbError> {
        let id = self
            .resolve_block_ref(&SaveBlockInput {
                workspace_id: workspace_id.to_string(),
                block_id,
                block_title,
                ..Default::default()
            })?
            .ok_or(DbError::NotFound)?;
        self.get_block(&id)
    }

    pub fn soft_delete_block(&self, block_id: &str) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        let rows = self.conn.execute(
            "UPDATE blocks SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            params![now, block_id],
        )?;
        if rows == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    /// Delete block by id or title (case-insensitive title match within workspace).
    pub fn delete_block(
        &self,
        workspace_id: &str,
        block_id: Option<String>,
        block_title: Option<String>,
    ) -> Result<(), DbError> {
        let id = self
            .resolve_block_ref(&SaveBlockInput {
                workspace_id: workspace_id.to_string(),
                block_id,
                block_title,
                ..Default::default()
            })?
            .ok_or(DbError::NotFound)?;
        self.soft_delete_block(&id)
    }

    pub fn get_block(&self, block_id: &str) -> Result<BlockEntry, DbError> {
        self.conn
            .query_row(
                "SELECT id, workspace_id, title, belief_state, system_tag, user_tag, created_at, updated_at
                 FROM blocks WHERE id = ?1 AND deleted_at IS NULL",
                [block_id],
                |row| {
                    Ok(BlockEntry {
                        id: row.get(0)?,
                        workspace_id: row.get(1)?,
                        title: row.get(2)?,
                        belief_state: row.get(3)?,
                        system_tag: row.get(4)?,
                        user_tag: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                        hypothesis: None,
                        action: None,
                        evidence: None,
                        conclusion: None,
                        linked_block_ids: vec![],
                        incomplete: false,
                    })
                },
            )
            .map_err(DbError::from)
            .and_then(|mut b| {
                b.hypothesis = self.load_block_hypothesis(block_id)?;
                b.action = self.load_block_action(block_id)?;
                b.evidence = self.load_block_evidence(block_id)?;
                b.conclusion = self.load_block_conclusion(block_id)?;
                b.linked_block_ids = self.list_block_link_targets(block_id)?;
                b.incomplete = block_has_gaps(&b);
                Ok(b)
            })
    }

    fn update_block_metadata(
        &self,
        block_id: &str,
        title: &str,
        belief: BeliefState,
        system_tag: BlockSystemTag,
        user_tag: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<(), DbError> {
        let prev: Option<String> = self
            .conn
            .query_row(
                "SELECT belief_state FROM blocks WHERE id = ?1",
                [block_id],
                |row| row.get(0),
            )
            .optional()?;

        self.conn.execute(
            "UPDATE blocks SET title = ?1, belief_state = ?2, system_tag = ?3, user_tag = ?4, updated_at = ?5 WHERE id = ?6",
            params![
                title,
                belief.as_str(),
                system_tag.as_str(),
                user_tag,
                now.to_rfc3339(),
                block_id
            ],
        )?;

        if let Some(from) = prev {
            if from != belief.as_str() {
                self.conn.execute(
                    "INSERT INTO belief_state_history (id, block_id, from_state, to_state, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        Uuid::new_v4().to_string(),
                        block_id,
                        from,
                        belief.as_str(),
                        now.to_rfc3339()
                    ],
                )?;
            }
        }
        Ok(())
    }

    fn clear_block_nodes(&self, block_id: &str) -> Result<(), DbError> {
        for table in ["hypotheses", "actions", "evidence"] {
            self.conn.execute(
                &format!("UPDATE {table} SET deleted_at = ?1 WHERE block_id = ?2 AND deleted_at IS NULL"),
                params![Utc::now().to_rfc3339(), block_id],
            )?;
        }
        self.conn.execute(
            "UPDATE conclusions SET superseded_at = ?1 WHERE block_id = ?2 AND superseded_at IS NULL",
            params![Utc::now().to_rfc3339(), block_id],
        )?;
        Ok(())
    }

    fn insert_hypothesis_in_block(
        &self,
        workspace_id: &str,
        block_id: &str,
        text: &str,
        belief: BeliefState,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO hypotheses (id, workspace_id, block_id, text, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, workspace_id, block_id, text, belief.as_str(), now.to_rfc3339()],
        )?;
        Ok(id)
    }

    fn insert_action_in_block(
        &self,
        workspace_id: &str,
        block_id: &str,
        text: &str,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO actions (id, workspace_id, block_id, text, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, workspace_id, block_id, text, now.to_rfc3339()],
        )?;
        Ok(id)
    }

    fn insert_evidence_in_block(
        &self,
        workspace_id: &str,
        block_id: &str,
        text: &str,
        source: Option<&str>,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO evidence (id, workspace_id, block_id, text, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, workspace_id, block_id, text, source, now.to_rfc3339()],
        )?;
        Ok(id)
    }

    fn insert_conclusion_in_block(
        &self,
        workspace_id: &str,
        block_id: &str,
        text: &str,
        outcome: ConclusionOutcome,
        tag: ConclusionTag,
        confidence_level: Option<ConfidenceLevel>,
    ) -> Result<String, DbError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO conclusions (id, workspace_id, block_id, text, outcome, tag, confidence_level, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                workspace_id,
                block_id,
                text,
                outcome.as_str(),
                tag.as_str(),
                confidence_level.map(|c| c.as_str()),
                now.to_rfc3339()
            ],
        )?;
        Ok(id)
    }

    fn load_block_hypothesis(&self, block_id: &str) -> Result<Option<BlockField>, DbError> {
        self.conn
            .query_row(
                "SELECT id, text FROM hypotheses WHERE block_id = ?1 AND deleted_at IS NULL LIMIT 1",
                [block_id],
                |row| Ok(BlockField { id: row.get(0)?, text: row.get(1)?, source: None }),
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_block_action(&self, block_id: &str) -> Result<Option<BlockField>, DbError> {
        self.conn
            .query_row(
                "SELECT id, text FROM actions WHERE block_id = ?1 AND deleted_at IS NULL LIMIT 1",
                [block_id],
                |row| Ok(BlockField { id: row.get(0)?, text: row.get(1)?, source: None }),
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_block_evidence(&self, block_id: &str) -> Result<Option<BlockField>, DbError> {
        self.conn
            .query_row(
                "SELECT id, text, source FROM evidence WHERE block_id = ?1 AND deleted_at IS NULL LIMIT 1",
                [block_id],
                |row| {
                    Ok(BlockField {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        source: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn load_block_conclusion(&self, block_id: &str) -> Result<Option<BlockConclusionField>, DbError> {
        self.conn
            .query_row(
                "SELECT id, text, outcome, tag, confidence_level FROM conclusions
                 WHERE block_id = ?1 AND superseded_at IS NULL LIMIT 1",
                [block_id],
                |row| {
                    Ok(BlockConclusionField {
                        id: row.get(0)?,
                        text: row.get(1)?,
                        outcome: row.get(2)?,
                        tag: row.get(3)?,
                        confidence_level: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    fn list_block_link_targets(&self, from_block_id: &str) -> Result<Vec<String>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT to_block_id FROM block_links WHERE from_block_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map([from_block_id], |row| row.get(0))?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    fn ensure_block_in_workspace(&self, workspace_id: &str, block_id: &str) -> Result<(), DbError> {
        let found: String = self.conn.query_row(
            "SELECT workspace_id FROM blocks WHERE id = ?1 AND deleted_at IS NULL",
            [block_id],
            |row| row.get(0),
        )?;
        if found != workspace_id {
            return Err(DbError::InvalidInput("cross-workspace block link".into()));
        }
        Ok(())
    }

    fn emit_block_event(
        &self,
        kind: EventKind,
        block_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO events (id, type, entity_type, entity_id, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                kind.as_str(),
                EntityType::Block.as_str(),
                block_id,
                payload.to_string(),
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn ensure_unique_title(
        &self,
        workspace_id: &str,
        title: &str,
        except_block_id: Option<&str>,
    ) -> Result<(), DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM blocks
             WHERE workspace_id = ?1 AND deleted_at IS NULL AND lower(title) = lower(?2)",
        )?;
        let conflicts: Vec<String> = stmt
            .query_map(params![workspace_id, title.trim()], |row| row.get(0))?
            .filter_map(Result::ok)
            .filter(|id| except_block_id.map(|ex| ex != id).unwrap_or(true))
            .collect();
        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(DbError::InvalidInput(format!(
                "block title \"{title}\" already exists in this workspace"
            )))
        }
    }
}

fn trim_opt(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// None in input = keep existing; Some("") = clear; Some(text) = set.
fn merge_text_input(input: Option<String>, existing: Option<String>) -> Option<String> {
    match input {
        None => existing.and_then(|s| trim_opt(Some(s))),
        Some(s) => trim_opt(Some(s)),
    }
}

fn merge_evidence_source(
    evidence_text_provided: bool,
    input_source: Option<String>,
    existing: Option<&BlockField>,
) -> Option<String> {
    if !evidence_text_provided {
        return existing.and_then(|e| e.source.clone());
    }
    match input_source {
        Some(s) => trim_opt(Some(s)),
        None => existing.and_then(|e| e.source.clone()),
    }
}

fn derive_title(
    h: &Option<String>,
    a: &Option<String>,
    e: &Option<String>,
    c: &Option<String>,
) -> String {
    for text in [h, a, e, c].into_iter().flatten() {
        if text.len() > 120 {
            return format!("{}…", &text[..117]);
        }
        return text.clone();
    }
    "Untitled block".to_string()
}

fn block_has_gaps(block: &BlockEntry) -> bool {
    let has_h = block.hypothesis.is_some();
    let has_a = block.action.is_some();
    let has_e = block.evidence.is_some();
    let has_c = block.conclusion.is_some();
    if has_c && (!has_h || !has_e) {
        return true;
    }
    if has_a && !has_e {
        return true;
    }
    if has_h && !has_a && !has_e && !has_c {
        return true;
    }
    false
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphStore;

    #[test]
    fn partial_update_preserves_unmentioned_fields() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let ws = store.create_workspace("T", "G", "blank").unwrap();
        let block = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("IDOR test".into()),
                hypothesis_text: Some("User IDs may be enumerable".into()),
                action_text: Some("curl /api/user/123".into()),
                ..Default::default()
            })
            .unwrap();

        let updated = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                block_id: Some(block.id.clone()),
                evidence_text: Some("HTTP 200 with other user's data".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(updated.title, "IDOR test");
        assert!(updated.hypothesis.is_some());
        assert!(updated.action.is_some());
        assert_eq!(
            updated.evidence.as_ref().unwrap().text,
            "HTTP 200 with other user's data"
        );
    }

    #[test]
    fn title_only_block_create() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let ws = store.create_workspace("T", "G", "blank").unwrap();
        let block = store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("Placeholder — hypothesis TBD".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(block.title, "Placeholder — hypothesis TBD");
        assert!(block.hypothesis.is_none());
        assert!(block.action.is_none());
        assert!(block.evidence.is_none());
        assert!(block.conclusion.is_none());
    }

    #[test]
    fn resolve_block_by_title() {
        let dir = tempfile::tempdir().unwrap();
        let store = GraphStore::open(&dir.path().join("t.db")).unwrap();
        let ws = store.create_workspace("T", "G", "blank").unwrap();
        store
            .save_block(SaveBlockInput {
                workspace_id: ws.id.clone(),
                title: Some("Auth bypass".into()),
                hypothesis_text: Some("JWT alg none".into()),
                ..Default::default()
            })
            .unwrap();

        let id = store
            .find_block_id_by_title(&ws.id, "auth bypass")
            .unwrap();
        let block = store.get_block(&id).unwrap();
        assert_eq!(block.title, "Auth bypass");
    }
}
