//! Ask AI History Manager
//!
//! Manages persistence of Ask AI conversations to the database.

use anyhow::Result;
use log::{debug, info};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use super::ask_ai::{AskAiConversation, ConversationTurn};

/// Manages Ask AI conversation persistence
pub struct AskAiHistoryManager {
    db_path: PathBuf,
    #[allow(dead_code)]
    recordings_dir: PathBuf,
}

impl AskAiHistoryManager {
    /// Create a new AskAiHistoryManager
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        let recordings_dir = app_data_dir.join("recordings");
        let db_path = app_data_dir.join("history.db");

        Ok(Self {
            db_path,
            recordings_dir,
        })
    }

    fn get_connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Save a conversation to the database
    pub fn save_conversation(&self, conversation: &AskAiConversation) -> Result<()> {
        let conn = self.get_connection()?;

        // Insert or update the conversation
        conn.execute(
            "INSERT OR REPLACE INTO ask_ai_conversations (id, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                conversation.id,
                conversation.title,
                conversation.created_at,
                conversation.updated_at
            ],
        )?;

        // Delete existing turns for this conversation (we'll re-insert them)
        conn.execute(
            "DELETE FROM ask_ai_turns WHERE conversation_id = ?1",
            params![conversation.id],
        )?;

        // Insert all turns
        for (order, turn) in conversation.turns.iter().enumerate() {
            conn.execute(
                "INSERT INTO ask_ai_turns (id, conversation_id, question, response, audio_file_name, timestamp, turn_order)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    turn.id,
                    conversation.id,
                    turn.question,
                    turn.response,
                    turn.audio_file_name,
                    turn.timestamp,
                    order as i64
                ],
            )?;
        }

        debug!("Saved conversation {} with {} turns", conversation.id, conversation.turns.len());
        Ok(())
    }

    /// Get a conversation by ID
    pub fn get_conversation(&self, id: &str) -> Result<Option<AskAiConversation>> {
        let conn = self.get_connection()?;

        // Get conversation metadata
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at FROM ask_ai_conversations WHERE id = ?1",
        )?;

        let conversation_opt = stmt
            .query_row([id], |row| {
                Ok(AskAiConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                    turns: Vec::new(),
                })
            })
            .optional()?;

        let mut conversation = match conversation_opt {
            Some(c) => c,
            None => return Ok(None),
        };

        // Get turns for this conversation
        let mut stmt = conn.prepare(
            "SELECT id, question, response, audio_file_name, timestamp
             FROM ask_ai_turns
             WHERE conversation_id = ?1
             ORDER BY turn_order ASC",
        )?;

        let turns = stmt.query_map([id], |row| {
            Ok(ConversationTurn {
                id: row.get(0)?,
                question: row.get(1)?,
                response: row.get(2)?,
                audio_file_name: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        for turn in turns {
            conversation.turns.push(turn?);
        }

        Ok(Some(conversation))
    }

    /// List recent conversations
    pub fn list_conversations(&self, limit: usize) -> Result<Vec<AskAiConversation>> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at
             FROM ask_ai_conversations
             ORDER BY updated_at DESC
             LIMIT ?1",
        )?;

        let conversations = stmt.query_map([limit as i64], |row| {
            Ok(AskAiConversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                turns: Vec::new(),
            })
        })?;

        let mut result = Vec::new();
        for conv in conversations {
            let mut conversation = conv?;

            // Load full turns for each conversation
            if let Ok(Some(full_conv)) = self.get_conversation(&conversation.id) {
                conversation = full_conv;
            }

            result.push(conversation);
        }

        Ok(result)
    }

    /// Delete a conversation and all its turns
    pub fn delete_conversation(&self, id: &str) -> Result<()> {
        let conn = self.get_connection()?;

        // Due to ON DELETE CASCADE, deleting the conversation will also delete turns
        let deleted = conn.execute(
            "DELETE FROM ask_ai_conversations WHERE id = ?1",
            params![id],
        )?;

        if deleted > 0 {
            info!("Deleted conversation {}", id);
        } else {
            debug!("Conversation {} not found for deletion", id);
        }

        Ok(())
    }

    /// Get the total count of conversations
    #[allow(dead_code)]
    pub fn get_conversation_count(&self) -> Result<i64> {
        let conn = self.get_connection()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM ask_ai_conversations",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Clean up old conversations beyond a certain limit
    #[allow(dead_code)]
    pub fn cleanup_old_conversations(&self, keep_count: usize) -> Result<usize> {
        let conn = self.get_connection()?;

        // Get IDs of conversations to delete (oldest ones beyond the limit)
        let mut stmt = conn.prepare(
            "SELECT id FROM ask_ai_conversations
             ORDER BY updated_at DESC
             LIMIT -1 OFFSET ?1",
        )?;

        let ids_to_delete: Vec<String> = stmt
            .query_map([keep_count as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let count = ids_to_delete.len();

        for id in ids_to_delete {
            self.delete_conversation(&id)?;
        }

        if count > 0 {
            info!("Cleaned up {} old conversations", count);
        }

        Ok(count)
    }
}
