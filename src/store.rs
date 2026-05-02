// Persistance SQLite — objets suivis + journal des transitions

use rusqlite::{Connection, params};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

/// Un objet chargé depuis la base
#[derive(Debug, Clone)]
pub struct StoredObject {
    pub object_id: String,
    pub workflow_id: String,
    pub current_state: String,
    pub frozen: bool,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(path)?;

        conn.execute_batch("
            CREATE TABLE IF NOT EXISTS objects (
                object_id    TEXT NOT NULL,
                workflow_id  TEXT NOT NULL,
                state        TEXT NOT NULL DEFAULT 'idéation',
                frozen       INTEGER NOT NULL DEFAULT 0,
                created_at   TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (object_id, workflow_id)
            );

            CREATE TABLE IF NOT EXISTS journal (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                workflow_id  TEXT NOT NULL,
                object_id    TEXT NOT NULL,
                from_state   TEXT NOT NULL,
                to_state     TEXT NOT NULL,
                transition   TEXT NOT NULL,
                timestamp    TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_journal_object
                ON journal(workflow_id, object_id);
        ")?;

        Ok(Self { conn })
    }

    /// Enregistre ou met à jour un objet (UPSERT)
    pub fn upsert_object(
        &self,
        workflow_id: &str,
        object_id: &str,
        state: &str,
        frozen: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO objects (object_id, workflow_id, state, frozen, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(object_id, workflow_id) DO UPDATE SET
               state = excluded.state,
               frozen = excluded.frozen,
               updated_at = excluded.updated_at",
            params![object_id, workflow_id, state, frozen as i32],
        )?;
        Ok(())
    }

    /// Charge tous les objets d'un workflow
    pub fn load_objects(
        &self,
        workflow_id: &str,
    ) -> Result<Vec<StoredObject>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT object_id, workflow_id, state, frozen
             FROM objects WHERE workflow_id = ?1"
        )?;
        let rows = stmt.query_map(params![workflow_id], |row| {
            Ok(StoredObject {
                object_id: row.get(0)?,
                workflow_id: row.get(1)?,
                current_state: row.get(2)?,
                frozen: row.get::<_, i32>(3)? != 0,
            })
        })?;
        let mut objects = Vec::new();
        for row in rows {
            objects.push(row?);
        }
        Ok(objects)
    }

    /// Enregistre une transition dans le journal
    pub fn log_transition(
        &self,
        workflow_id: &str,
        object_id: &str,
        from_state: &str,
        to_state: &str,
        transition_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO journal (workflow_id, object_id, from_state, to_state, transition)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![workflow_id, object_id, from_state, to_state, transition_id],
        )?;
        log::debug!("[store] {}: {} → {} via {}", object_id, from_state, to_state, transition_id);
        Ok(())
    }

    /// Compte le nombre de transitions récentes (pour rate_limit)
    pub fn recent_transition_count(
        &self,
        workflow_id: &str,
        object_id: &str,
        transition_id: &str,
        window_minutes: i64,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM journal
             WHERE workflow_id = ?1
               AND object_id = ?2
               AND transition = ?3
               AND timestamp > datetime('now', ?4)",
            params![
                workflow_id,
                object_id,
                transition_id,
                format!("-{} minutes", window_minutes)
            ],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}
