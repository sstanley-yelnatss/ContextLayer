use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::DbError;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_initial", include_str!("../../../migrations/001_initial.sql")),
    (
        "002_timeline_indexes",
        include_str!("../../../migrations/002_timeline_indexes.sql"),
    ),
    ("003_blocks", include_str!("../../../migrations/003_blocks.sql")),
    (
        "004_block_title",
        include_str!("../../../migrations/004_block_title.sql"),
    ),
    (
        "005_workspace_archived",
        include_str!("../../../migrations/005_workspace_archived.sql"),
    ),
];

const SCHEMA_BOOTSTRAP: &str = "
CREATE TABLE IF NOT EXISTS schema_migrations (
  version TEXT PRIMARY KEY NOT NULL,
  applied_at TEXT NOT NULL
);
";

pub fn default_db_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".contextlayer")
        .join("graph.db")
}

pub fn open(db_path: &Path) -> Result<Connection, DbError> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
    Ok(conn)
}

pub fn migrate(db_path: &Path) -> Result<(), DbError> {
    let conn = open(db_path)?;
    run_migrations(&conn)
}

pub fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(SCHEMA_BOOTSTRAP)?;

    for (version, sql) in MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                [*version],
                |row| row.get::<_, i32>(0),
            )
            .optional()?
            .is_some();

        if applied {
            continue;
        }

        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            (*version, Utc::now().to_rfc3339()),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(Result::ok)
            .collect()
    }

    #[test]
    fn migrations_create_prd_tables_only() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        run_migrations(&conn).unwrap();

        let tables = table_names(&conn);
        for required in [
            "workspaces",
            "blocks",
            "block_links",
            "belief_state_history",
            "hypotheses",
            "actions",
            "evidence",
            "conclusions",
            "node_links",
            "entity_versions",
            "events",
            "schema_migrations",
        ] {
            assert!(tables.contains(&required.to_string()), "missing {required}");
        }
        assert!(!tables.contains(&"notes".to_string()));
        assert!(!tables.contains(&"constraints".to_string()));
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }
}
