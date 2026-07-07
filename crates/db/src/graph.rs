use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use contextlayer_core::{
    derive_hypothesis_status, validate_action_text, validate_conclusion_fields,
    validate_conclusion_links, validate_evidence_text, validate_hypothesis_text,
    validate_link_pair, validate_workspace, Action, AdmissionError, BeliefState, Conclusion,
    ConclusionOutcome, EntityType, EventKind, Evidence, Hypothesis, NodeLink, NodeType,
    Workspace, WorkspaceTemplate,
};

use crate::DbError;

pub struct GraphStore {
    pub(crate) conn: Connection,
}

impl GraphStore {
    pub fn open(db_path: &std::path::Path) -> Result<Self, DbError> {
        let conn = crate::open(db_path)?;
        crate::run_migrations(&conn)?;
        let store = Self { conn };
        store.migrate_legacy_nodes_to_blocks()?;
        Ok(store)
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn seed_dogfood_if_empty(&self) -> Result<bool, DbError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(false);
        }

        self.create_workspace(
            "ContextLayer product validation",
            "Validate Phase 1 reasoning graph UX and dogfood the product itself",
            WorkspaceTemplate::ProductResearch.as_str(),
        )?;
        self.create_workspace(
            "Security domain research",
            "Evaluate tools and hypotheses in security research workflows",
            WorkspaceTemplate::SecurityHunt.as_str(),
        )?;
        Ok(true)
    }

    /// When `archived_only` is true, return archived workspaces only; otherwise active (non-archived) only.
    pub fn list_workspaces(&self, archived_only: bool) -> Result<Vec<Workspace>, DbError> {
        let sql = if archived_only {
            "SELECT id, name, goal, template, created_at, updated_at, archived_at FROM workspaces WHERE archived_at IS NOT NULL ORDER BY updated_at DESC"
        } else {
            "SELECT id, name, goal, template, created_at, updated_at, archived_at FROM workspaces WHERE archived_at IS NULL ORDER BY updated_at DESC"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                goal: row.get(2)?,
                template: WorkspaceTemplate::parse(&row.get::<_, String>(3)?)
                    .unwrap_or(WorkspaceTemplate::Blank),
                created_at: parse_ts(&row.get::<_, String>(4)?),
                updated_at: parse_ts(&row.get::<_, String>(5)?),
                archived_at: row
                    .get::<_, Option<String>>(6)?
                    .map(|s| parse_ts(&s)),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn create_workspace(&self, name: &str, goal: &str, template: &str) -> Result<Workspace, DbError> {
        let template = validate_workspace(name, goal, template).map_err(map_admission)?;
        let now = Utc::now();
        let id = Uuid::new_v4().to_string();
        let now_s = now.to_rfc3339();
        self.conn.execute(
            "INSERT INTO workspaces (id, name, goal, template, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name.trim(), goal.trim(), template.as_str(), now_s, now_s],
        )?;
        self.emit_event(
            EventKind::Created,
            EntityType::Workspace,
            &id,
            serde_json::json!({ "name": name, "goal": goal }),
        )?;
        Ok(Workspace {
            id,
            name: name.trim().to_string(),
            goal: goal.trim().to_string(),
            template,
            created_at: now,
            updated_at: now,
            archived_at: None,
        })
    }

    pub fn update_workspace(
        &self,
        id: &str,
        name: &str,
        goal: &str,
        template: &str,
    ) -> Result<Workspace, DbError> {
        let template = validate_workspace(name, goal, template).map_err(map_admission)?;
        let existing = self.get_workspace(id)?;
        self.snapshot_version(
            EntityType::Workspace,
            id,
            &serde_json::to_string(&existing).unwrap(),
        )?;
        let now = Utc::now();
        self.conn.execute(
            "UPDATE workspaces SET name = ?1, goal = ?2, template = ?3, updated_at = ?4 WHERE id = ?5",
            params![name.trim(), goal.trim(), template.as_str(), now.to_rfc3339(), id],
        )?;
        self.emit_event(
            EventKind::Corrected,
            EntityType::Workspace,
            id,
            serde_json::json!({ "name": name, "goal": goal }),
        )?;
        self.get_workspace(id)
    }

    pub fn get_workspace(&self, id: &str) -> Result<Workspace, DbError> {
        self.conn.query_row(
            "SELECT id, name, goal, template, created_at, updated_at, archived_at FROM workspaces WHERE id = ?1",
            [id],
            |row| {
                Ok(Workspace {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    goal: row.get(2)?,
                    template: WorkspaceTemplate::parse(&row.get::<_, String>(3)?)
                        .unwrap_or(WorkspaceTemplate::Blank),
                    created_at: parse_ts(&row.get::<_, String>(4)?),
                    updated_at: parse_ts(&row.get::<_, String>(5)?),
                    archived_at: row
                        .get::<_, Option<String>>(6)?
                        .map(|s| parse_ts(&s)),
                })
            },
        ).map_err(Into::into)
    }

    pub fn set_workspace_archived(&self, id: &str, archived: bool) -> Result<Workspace, DbError> {
        let _existing = self.get_workspace(id)?;
        let now = Utc::now();
        let archived_at = if archived { Some(now.to_rfc3339()) } else { None };
        self.conn.execute(
            "UPDATE workspaces SET archived_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![archived_at, now.to_rfc3339(), id],
        )?;
        self.emit_event(
            EventKind::Corrected,
            EntityType::Workspace,
            id,
            serde_json::json!({ "archived": archived }),
        )?;
        self.get_workspace(id)
    }

    /// Resolve a workspace UUID or exact name (case-insensitive) to id.
    pub fn resolve_workspace_id(&self, name_or_id: &str) -> Result<String, DbError> {
        let key = name_or_id.trim();
        if key.is_empty() {
            return Err(DbError::InvalidInput("workspace is required".into()));
        }
        if self.get_workspace(key).is_ok() {
            return Ok(key.to_string());
        }
        let mut stmt = self.conn.prepare(
            "SELECT id FROM workspaces WHERE name = ?1 COLLATE NOCASE ORDER BY updated_at DESC",
        )?;
        let mut rows = stmt.query([key])?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            ids.push(row.get::<_, String>(0)?);
        }
        match ids.len() {
            0 => Err(DbError::NotFound),
            1 => Ok(ids[0].clone()),
            n => Err(DbError::InvalidInput(format!(
                "ambiguous workspace name `{key}` ({n} matches) — use workspace id or rename"
            ))),
        }
    }

    /// Permanently delete a workspace and all related data.
    pub fn delete_workspace(&self, workspace_id: &str) -> Result<(), DbError> {
        self.get_workspace(workspace_id)?;

        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM block_links WHERE workspace_id = ?1",
            [workspace_id],
        )?;
        tx.execute(
            "DELETE FROM belief_state_history WHERE block_id IN (SELECT id FROM blocks WHERE workspace_id = ?1)",
            [workspace_id],
        )?;
        for table in ["hypotheses", "actions", "evidence", "conclusions"] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE workspace_id = ?1"),
                [workspace_id],
            )?;
        }
        tx.execute("DELETE FROM blocks WHERE workspace_id = ?1", [workspace_id])?;
        tx.execute("DELETE FROM node_links WHERE workspace_id = ?1", [workspace_id])?;
        tx.execute("DELETE FROM workspaces WHERE id = ?1", [workspace_id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_hypothesis(&self, workspace_id: &str, text: &str) -> Result<Hypothesis, DbError> {
        validate_hypothesis_text(text).map_err(map_admission)?;
        self.ensure_workspace(workspace_id)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO hypotheses (id, workspace_id, text, status, created_at) VALUES (?1, ?2, ?3, 'open', ?4)",
            params![id, workspace_id, text.trim(), now.to_rfc3339()],
        )?;
        self.emit_event(
            EventKind::Created,
            EntityType::Hypothesis,
            &id,
            serde_json::json!({ "text": text }),
        )?;
        Ok(Hypothesis {
            id,
            workspace_id: workspace_id.to_string(),
            text: text.trim().to_string(),
            status: BeliefState::Open,
            block_id: None,
            created_at: now,
            deleted_at: None,
        })
    }

    pub fn create_action(&self, workspace_id: &str, text: &str) -> Result<Action, DbError> {
        validate_action_text(text).map_err(map_admission)?;
        self.ensure_workspace(workspace_id)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO actions (id, workspace_id, text, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, workspace_id, text.trim(), now.to_rfc3339()],
        )?;
        self.emit_event(
            EventKind::Created,
            EntityType::Action,
            &id,
            serde_json::json!({ "text": text }),
        )?;
        Ok(Action {
            id,
            workspace_id: workspace_id.to_string(),
            text: text.trim().to_string(),
            block_id: None,
            created_at: now,
            deleted_at: None,
        })
    }

    pub fn create_evidence(
        &self,
        workspace_id: &str,
        text: &str,
        source: Option<&str>,
    ) -> Result<Evidence, DbError> {
        validate_evidence_text(text).map_err(map_admission)?;
        self.ensure_workspace(workspace_id)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO evidence (id, workspace_id, text, source, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, workspace_id, text.trim(), source, now.to_rfc3339()],
        )?;
        self.emit_event(
            EventKind::Created,
            EntityType::Evidence,
            &id,
            serde_json::json!({ "text": text, "source": source }),
        )?;
        Ok(Evidence {
            id,
            workspace_id: workspace_id.to_string(),
            text: text.trim().to_string(),
            source: source.map(str::to_string),
            block_id: None,
            created_at: now,
            deleted_at: None,
        })
    }

    pub fn save_conclusion(
        &self,
        workspace_id: &str,
        text: &str,
        outcome: &str,
        tag: &str,
        confidence: Option<f64>,
        hypothesis_ids: &[String],
        evidence_ids: &[String],
    ) -> Result<Conclusion, DbError> {
        validate_conclusion_fields(text, outcome, tag, confidence).map_err(map_admission)?;
        validate_conclusion_links(hypothesis_ids.len(), evidence_ids.len()).map_err(map_admission)?;
        self.ensure_workspace(workspace_id)?;

        let (outcome_enum, tag_enum) =
            validate_conclusion_fields(text, outcome, tag, confidence).map_err(map_admission)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO conclusions (id, workspace_id, text, outcome, tag, confidence, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                workspace_id,
                text.trim(),
                outcome_enum.as_str(),
                tag_enum.as_str(),
                confidence,
                now.to_rfc3339()
            ],
        )?;

        for hid in hypothesis_ids {
            self.add_link_internal(workspace_id, NodeType::Conclusion, &id, NodeType::Hypothesis, hid)?;
        }
        for eid in evidence_ids {
            self.add_link_internal(workspace_id, NodeType::Conclusion, &id, NodeType::Evidence, eid)?;
        }

        self.refresh_hypothesis_statuses_for(workspace_id, hypothesis_ids)?;

        self.emit_event(
            EventKind::Created,
            EntityType::Conclusion,
            &id,
            serde_json::json!({ "text": text, "outcome": outcome }),
        )?;

        Ok(Conclusion {
            id,
            workspace_id: workspace_id.to_string(),
            text: text.trim().to_string(),
            outcome: outcome_enum,
            tag: tag_enum,
            confidence,
            confidence_level: None,
            block_id: None,
            created_at: now,
            superseded_at: None,
        })
    }

    pub fn supersede_conclusion(&self, conclusion_id: &str) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE conclusions SET superseded_at = ?1 WHERE id = ?2 AND superseded_at IS NULL",
            params![now, conclusion_id],
        )?;
        Ok(())
    }

    pub fn add_link(
        &self,
        workspace_id: &str,
        from_type: &str,
        from_id: &str,
        to_type: &str,
        to_id: &str,
    ) -> Result<NodeLink, DbError> {
        let from = NodeType::parse(from_type).ok_or(DbError::InvalidInput("invalid from_type".into()))?;
        let to = NodeType::parse(to_type).ok_or(DbError::InvalidInput("invalid to_type".into()))?;
        validate_link_pair(from, to).map_err(map_admission)?;
        self.add_link_internal(workspace_id, from, from_id, to, to_id)
    }

    pub(crate) fn add_link_internal(
        &self,
        workspace_id: &str,
        from_type: NodeType,
        from_id: &str,
        to_type: NodeType,
        to_id: &str,
    ) -> Result<NodeLink, DbError> {
        validate_link_pair(from_type, to_type).map_err(map_admission)?;
        self.ensure_same_workspace(workspace_id, from_type, from_id)?;
        self.ensure_same_workspace(workspace_id, to_type, to_id)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO node_links (id, workspace_id, from_type, from_id, to_type, to_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                workspace_id,
                from_type.as_str(),
                from_id,
                to_type.as_str(),
                to_id,
                now.to_rfc3339()
            ],
        )?;
        self.emit_event(
            EventKind::LinkAdded,
            EntityType::Workspace,
            workspace_id,
            serde_json::json!({
                "from_type": from_type.as_str(),
                "from_id": from_id,
                "to_type": to_type.as_str(),
                "to_id": to_id,
            }),
        )?;
        Ok(NodeLink {
            id,
            workspace_id: workspace_id.to_string(),
            from_type,
            from_id: from_id.to_string(),
            to_type,
            to_id: to_id.to_string(),
            created_at: now,
        })
    }

    pub fn remove_link(&self, link_id: &str) -> Result<(), DbError> {
        let (workspace_id, from_type, from_id, to_type, to_id): (String, String, String, String, String) =
            self.conn.query_row(
                "SELECT workspace_id, from_type, from_id, to_type, to_id FROM node_links WHERE id = ?1",
                [link_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )?;
        self.conn
            .execute("DELETE FROM node_links WHERE id = ?1", [link_id])?;
        self.emit_event(
            EventKind::LinkRemoved,
            EntityType::Workspace,
            &workspace_id,
            serde_json::json!({
                "from_type": from_type,
                "from_id": from_id,
                "to_type": to_type,
                "to_id": to_id,
            }),
        )?;
        Ok(())
    }

    pub fn soft_delete_node(&self, node_type: &str, node_id: &str) -> Result<(), DbError> {
        let table = node_table(node_type)?;
        let now = Utc::now().to_rfc3339();
        let entity = entity_type(node_type)?;
        let rows = self.conn.execute(
            &format!("UPDATE {table} SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL"),
            params![now, node_id],
        )?;
        if rows == 0 {
            return Err(DbError::NotFound);
        }
        self.emit_event(
            EventKind::SoftDeleted,
            entity,
            node_id,
            serde_json::json!({}),
        )?;
        Ok(())
    }

    pub fn edit_hypothesis(&self, id: &str, text: &str) -> Result<Hypothesis, DbError> {
        validate_hypothesis_text(text).map_err(map_admission)?;
        let existing = self.get_hypothesis(id)?;
        self.snapshot_version(EntityType::Hypothesis, id, &serde_json::to_string(&existing).unwrap())?;
        self.conn.execute(
            "UPDATE hypotheses SET text = ?1 WHERE id = ?2",
            params![text.trim(), id],
        )?;
        self.emit_event(
            EventKind::Corrected,
            EntityType::Hypothesis,
            id,
            serde_json::json!({ "text": text }),
        )?;
        self.get_hypothesis(id)
    }

    pub fn fetch_timeline(
        &self,
        workspace_id: &str,
        ascending: bool,
        types: Option<Vec<String>>,
    ) -> Result<Vec<TimelineEntry>, DbError> {
        let mut entries = Vec::new();
        let type_filter: Option<Vec<NodeType>> = types.map(|ts| {
            ts.iter()
                .filter_map(|t| NodeType::parse(t))
                .collect()
        });

        let include = |nt: NodeType| -> bool {
            type_filter
                .as_ref()
                .map(|f| f.contains(&nt))
                .unwrap_or(true)
        };

        if include(NodeType::Hypothesis) {
            let mut stmt = self.conn.prepare(
                "SELECT id, text, status, created_at, deleted_at FROM hypotheses WHERE workspace_id = ?1 AND deleted_at IS NULL",
            )?;
            for row in stmt.query_map([workspace_id], |row| {
                Ok(TimelineEntry {
                    node_type: NodeType::Hypothesis,
                    id: row.get(0)?,
                    text: row.get(1)?,
                    status: Some(row.get(2)?),
                    source: None,
                    outcome: None,
                    tag: None,
                    confidence: None,
                    created_at: row.get::<_, String>(3)?,
                    deleted_at: row.get(4)?,
                    link_count: 0,
                    unlinked: true,
                    rejected: false,
                    superseded: false,
                })
            })? {
                entries.push(row?);
            }
        }

        if include(NodeType::Action) {
            let mut stmt = self.conn.prepare(
                "SELECT id, text, created_at FROM actions WHERE workspace_id = ?1 AND deleted_at IS NULL",
            )?;
            for row in stmt.query_map([workspace_id], |row| {
                Ok(TimelineEntry {
                    node_type: NodeType::Action,
                    id: row.get(0)?,
                    text: row.get(1)?,
                    status: None,
                    source: None,
                    outcome: None,
                    tag: None,
                    confidence: None,
                    created_at: row.get(2)?,
                    deleted_at: None,
                    link_count: 0,
                    unlinked: true,
                    rejected: false,
                    superseded: false,
                })
            })? {
                entries.push(row?);
            }
        }

        if include(NodeType::Evidence) {
            let mut stmt = self.conn.prepare(
                "SELECT id, text, source, created_at FROM evidence WHERE workspace_id = ?1 AND deleted_at IS NULL",
            )?;
            for row in stmt.query_map([workspace_id], |row| {
                Ok(TimelineEntry {
                    node_type: NodeType::Evidence,
                    id: row.get(0)?,
                    text: row.get(1)?,
                    status: None,
                    source: row.get(2)?,
                    outcome: None,
                    tag: None,
                    confidence: None,
                    created_at: row.get(3)?,
                    deleted_at: None,
                    link_count: 0,
                    unlinked: true,
                    rejected: false,
                    superseded: false,
                })
            })? {
                entries.push(row?);
            }
        }

        if include(NodeType::Conclusion) {
            let mut stmt = self.conn.prepare(
                "SELECT id, text, outcome, tag, confidence, created_at, superseded_at FROM conclusions WHERE workspace_id = ?1",
            )?;
            for row in stmt.query_map([workspace_id], |row| {
                let superseded_at: Option<String> = row.get(6)?;
                Ok(TimelineEntry {
                    node_type: NodeType::Conclusion,
                    id: row.get(0)?,
                    text: row.get(1)?,
                    status: None,
                    source: None,
                    outcome: Some(row.get(2)?),
                    tag: Some(row.get(3)?),
                    confidence: row.get(4)?,
                    created_at: row.get(5)?,
                    deleted_at: None,
                    link_count: 0,
                    unlinked: true,
                    rejected: false,
                    superseded: superseded_at.is_some(),
                })
            })? {
                entries.push(row?);
            }
        }

        for entry in &mut entries {
            let (from_count, to_count): (i64, i64) = self.conn.query_row(
                "SELECT
                    (SELECT COUNT(*) FROM node_links WHERE from_id = ?1),
                    (SELECT COUNT(*) FROM node_links WHERE to_id = ?1)",
                [&entry.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            entry.link_count = from_count + to_count;
            entry.unlinked = entry.link_count == 0;
            if entry.node_type == NodeType::Hypothesis {
                if let Some(status) = &entry.status {
                    entry.rejected = status == "rejected";
                }
            }
        }

        entries.sort_by(|a, b| {
            let ord = a.created_at.cmp(&b.created_at);
            if ascending { ord } else { ord.reverse() }
        });

        Ok(entries)
    }

    pub fn list_links_for_workspace(&self, workspace_id: &str) -> Result<Vec<NodeLink>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, from_type, from_id, to_type, to_id, created_at FROM node_links WHERE workspace_id = ?1",
        )?;
        let rows = stmt.query_map([workspace_id], |row| {
            Ok(NodeLink {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                from_type: NodeType::parse(&row.get::<_, String>(2)?).unwrap(),
                from_id: row.get(3)?,
                to_type: NodeType::parse(&row.get::<_, String>(4)?).unwrap(),
                to_id: row.get(5)?,
                created_at: parse_ts(&row.get::<_, String>(6)?),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn list_nodes_for_picker(&self, workspace_id: &str, node_type: &str) -> Result<Vec<PickerNode>, DbError> {
        let nt = NodeType::parse(node_type).ok_or(DbError::InvalidInput("invalid node_type".into()))?;
        let sql = match nt {
            NodeType::Hypothesis => {
                "SELECT id, text FROM hypotheses WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at DESC"
            }
            NodeType::Action => {
                "SELECT id, text FROM actions WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at DESC"
            }
            NodeType::Evidence => {
                "SELECT id, text FROM evidence WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY created_at DESC"
            }
            NodeType::Conclusion => {
                "SELECT id, text FROM conclusions WHERE workspace_id = ?1 AND superseded_at IS NULL ORDER BY created_at DESC"
            }
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([workspace_id], |row| {
            Ok(PickerNode {
                id: row.get(0)?,
                text: row.get(1)?,
                node_type: nt,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    fn get_hypothesis(&self, id: &str) -> Result<Hypothesis, DbError> {
        self.conn.query_row(
            "SELECT id, workspace_id, text, status, block_id, created_at, deleted_at FROM hypotheses WHERE id = ?1",
            [id],
            |row| {
                Ok(Hypothesis {
                    id: row.get(0)?,
                    workspace_id: row.get(1)?,
                    text: row.get(2)?,
                    status: BeliefState::parse(&row.get::<_, String>(3)?)
                        .unwrap_or(BeliefState::Open),
                    block_id: row.get(4)?,
                    created_at: parse_ts(&row.get::<_, String>(5)?),
                    deleted_at: row.get::<_, Option<String>>(6)?.map(|s| parse_ts(&s)),
                })
            },
        ).map_err(Into::into)
    }

    pub(crate) fn refresh_hypothesis_statuses_for(
        &self,
        workspace_id: &str,
        hypothesis_ids: &[String],
    ) -> Result<(), DbError> {
        for hid in hypothesis_ids {
            let mut stmt = self.conn.prepare(
                "SELECT c.outcome FROM conclusions c
                 INNER JOIN node_links nl ON nl.from_id = c.id AND nl.from_type = 'conclusion' AND nl.to_type = 'hypothesis' AND nl.to_id = ?1
                 WHERE c.workspace_id = ?2 AND c.superseded_at IS NULL",
            )?;
            let rows = stmt.query_map(params![hid, workspace_id], |row| {
                Ok(ConclusionOutcome::parse(&row.get::<_, String>(0)?).unwrap())
            })?;
            let outcomes: Vec<ConclusionOutcome> = rows.filter_map(Result::ok).collect();
            let status = derive_hypothesis_status(&outcomes);
            self.conn.execute(
                "UPDATE hypotheses SET status = ?1 WHERE id = ?2",
                params![status.as_str(), hid],
            )?;
        }
        Ok(())
    }

    pub(crate) fn ensure_workspace(&self, id: &str) -> Result<(), DbError> {
        let exists: Option<i32> = self
            .conn
            .query_row("SELECT 1 FROM workspaces WHERE id = ?1", [id], |row| row.get(0))
            .optional()?;
        if exists.is_none() {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    fn ensure_same_workspace(
        &self,
        workspace_id: &str,
        node_type: NodeType,
        node_id: &str,
    ) -> Result<(), DbError> {
        let table = match node_type {
            NodeType::Hypothesis => "hypotheses",
            NodeType::Action => "actions",
            NodeType::Evidence => "evidence",
            NodeType::Conclusion => "conclusions",
        };
        let sql = format!("SELECT workspace_id FROM {table} WHERE id = ?1");
        let found: String = self.conn.query_row(&sql, [node_id], |row| row.get(0))?;
        if found != workspace_id {
            return Err(DbError::InvalidInput("cross-workspace link".into()));
        }
        Ok(())
    }

    fn snapshot_version(&self, entity_type: EntityType, entity_id: &str, body_json: &str) -> Result<(), DbError> {
        let version: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) + 1 FROM entity_versions WHERE entity_type = ?1 AND entity_id = ?2",
                params![entity_type.as_str(), entity_id],
                |row| row.get(0),
            )
            .unwrap_or(1);
        self.conn.execute(
            "INSERT INTO entity_versions (id, entity_type, entity_id, version, body_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                entity_type.as_str(),
                entity_id,
                version,
                body_json,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    fn emit_event(
        &self,
        kind: EventKind,
        entity_type: EntityType,
        entity_id: &str,
        payload: serde_json::Value,
    ) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT INTO events (id, type, entity_type, entity_id, payload_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Uuid::new_v4().to_string(),
                kind.as_str(),
                entity_type.as_str(),
                entity_id,
                payload.to_string(),
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TimelineEntry {
    pub node_type: NodeType,
    pub id: String,
    pub text: String,
    pub status: Option<String>,
    pub source: Option<String>,
    pub outcome: Option<String>,
    pub tag: Option<String>,
    pub confidence: Option<f64>,
    pub created_at: String,
    pub deleted_at: Option<String>,
    pub link_count: i64,
    pub unlinked: bool,
    pub rejected: bool,
    pub superseded: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PickerNode {
    pub id: String,
    pub text: String,
    pub node_type: NodeType,
}

fn parse_ts(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

fn node_table(node_type: &str) -> Result<&'static str, DbError> {
    match node_type {
        "hypothesis" => Ok("hypotheses"),
        "action" => Ok("actions"),
        "evidence" => Ok("evidence"),
        "conclusion" => Ok("conclusions"),
        _ => Err(DbError::InvalidInput("invalid node_type".into())),
    }
}

fn entity_type(node_type: &str) -> Result<EntityType, DbError> {
    match node_type {
        "hypothesis" => Ok(EntityType::Hypothesis),
        "action" => Ok(EntityType::Action),
        "evidence" => Ok(EntityType::Evidence),
        "conclusion" => Ok(EntityType::Conclusion),
        _ => Err(DbError::InvalidInput("invalid node_type".into())),
    }
}

fn map_admission(err: AdmissionError) -> DbError {
    DbError::Admission(err.to_string())
}
