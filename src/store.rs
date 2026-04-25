// Persistance SQLite des transitions

use rusqlite::Connection;
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(path)?;
        // TODO: CREATE TABLE IF NOT EXISTS
        Ok(Self { conn })
    }

    pub fn log_transition(
        &self,
        _workflow_id: &str,
        _object_id: &str,
        _from_state: &str,
        _to_state: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: INSERT INTO workflow_log
        log::debug!("Store::log_transition (placeholder)");
        Ok(())
    }
}
