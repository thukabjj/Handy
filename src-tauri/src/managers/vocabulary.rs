use log::{debug, info};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VocabularyEntry {
    pub id: i64,
    pub term: String,
    pub frequency: i64,
    pub source: String,
    pub category: Option<String>,
    pub created_at: String,
}

pub struct VocabularyManager {
    db_path: PathBuf,
}

impl VocabularyManager {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self, String> {
        let db_path = app_data_dir.join("vocabulary.db");
        let manager = Self { db_path };
        manager.initialize_db()?;
        Ok(manager)
    }

    fn initialize_db(&self) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS vocabulary (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                term TEXT NOT NULL UNIQUE,
                frequency INTEGER NOT NULL DEFAULT 0,
                source TEXT NOT NULL DEFAULT 'manual',
                category TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .map_err(|e| format!("Failed to create vocabulary table: {}", e))?;
        Ok(())
    }

    fn get_connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open vocabulary DB: {}", e))
    }

    pub fn get_vocabulary(&self) -> Result<Vec<VocabularyEntry>, String> {
        let conn = self.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, term, frequency, source, category, created_at
                 FROM vocabulary ORDER BY frequency DESC, term ASC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(VocabularyEntry {
                    id: row.get(0)?,
                    term: row.get(1)?,
                    frequency: row.get(2)?,
                    source: row.get(3)?,
                    category: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query vocabulary: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    pub fn add_term(
        &self,
        term: &str,
        category: Option<&str>,
    ) -> Result<VocabularyEntry, String> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO vocabulary (term, source, category) VALUES (?1, 'manual', ?2)
             ON CONFLICT(term) DO UPDATE SET frequency = frequency + 1",
            params![term, category],
        )
        .map_err(|e| format!("Failed to add term: {}", e))?;

        let id = conn.last_insert_rowid();
        debug!("Added vocabulary term: {} (id={})", term, id);

        // Fetch the inserted/updated entry
        let mut stmt = conn
            .prepare(
                "SELECT id, term, frequency, source, category, created_at
                 FROM vocabulary WHERE term = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row(params![term], |row| {
            Ok(VocabularyEntry {
                id: row.get(0)?,
                term: row.get(1)?,
                frequency: row.get(2)?,
                source: row.get(3)?,
                category: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to fetch inserted term: {}", e))
    }

    pub fn remove_term(&self, id: i64) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute("DELETE FROM vocabulary WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete term: {}", e))?;
        Ok(())
    }

    pub fn import_vocabulary(&self, json: &str) -> Result<usize, String> {
        let entries: Vec<VocabularyEntry> =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let conn = self.get_connection()?;
        let mut count = 0;

        for entry in &entries {
            match conn.execute(
                "INSERT INTO vocabulary (term, source, category) VALUES (?1, 'imported', ?2)
                 ON CONFLICT(term) DO NOTHING",
                params![entry.term, entry.category],
            ) {
                Ok(n) => count += n,
                Err(e) => debug!("Skipping term '{}': {}", entry.term, e),
            }
        }

        info!("Imported {} vocabulary terms", count);
        Ok(count)
    }

    pub fn export_vocabulary(&self) -> Result<String, String> {
        let entries = self.get_vocabulary()?;
        serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Failed to serialize vocabulary: {}", e))
    }
}
