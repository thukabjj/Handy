//! RAG (Retrieval-Augmented Generation) Manager
//!
//! Provides local knowledge base functionality for context-aware AI responses.
//! Uses SQLite for vector storage and Ollama for embedding generation.

use crate::ollama_client::OllamaClient;
use log::{debug, info, warn};
use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Database migrations for RAG tables
static MIGRATIONS: &[M] = &[
    M::up(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id TEXT,
            title TEXT,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS embeddings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            document_id INTEGER NOT NULL,
            chunk_index INTEGER NOT NULL DEFAULT 0,
            chunk_text TEXT NOT NULL,
            embedding BLOB NOT NULL,
            dimensions INTEGER NOT NULL,
            model TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_embeddings_document ON embeddings(document_id);
        CREATE INDEX IF NOT EXISTS idx_documents_source ON documents(source_type, source_id);
        "#,
    ),
    M::up(
        r#"
        CREATE TABLE IF NOT EXISTS rag_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO rag_settings (key, value) VALUES
            ('embedding_model', 'nomic-embed-text'),
            ('chunk_size', '500'),
            ('chunk_overlap', '50'),
            ('top_k', '3');
        "#,
    ),
];

/// Document metadata for RAG storage
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct DocMetadata {
    /// Type of source: "transcription", "upload", "note"
    pub source_type: String,
    /// Optional reference ID (e.g., history entry ID, file path)
    pub source_id: Option<String>,
    /// Document title or description
    pub title: Option<String>,
    /// Additional metadata as JSON (stored as string for TypeScript compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[specta(skip)]
    pub extra: Option<serde_json::Value>,
}

/// Search result from RAG query
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SearchResult {
    /// Document ID
    pub document_id: i64,
    /// Chunk text that matched
    pub chunk_text: String,
    /// Similarity score (0-1, higher is better)
    pub similarity: f32,
    /// Document metadata
    pub metadata: DocMetadata,
    /// Original document title
    pub title: Option<String>,
}

/// Stored document representation
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct StoredDocument {
    pub id: i64,
    pub content: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub title: Option<String>,
    pub created_at: i64,
}

/// RAG Manager for knowledge base operations
pub struct RagManager {
    db_path: PathBuf,
    ollama_client: Arc<OllamaClient>,
    embedding_model: Mutex<String>,
}

impl RagManager {
    /// Create a new RAG manager
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    /// * `ollama_client` - Shared Ollama client for embedding generation
    pub fn new(db_path: PathBuf, ollama_client: Arc<OllamaClient>) -> Result<Self, String> {
        let manager = Self {
            db_path,
            ollama_client,
            embedding_model: Mutex::new("nomic-embed-text".to_string()),
        };

        // Initialize database
        manager.init_database()?;

        // Load settings
        if let Ok(conn) = manager.get_connection() {
            if let Ok(model) = conn.query_row(
                "SELECT value FROM rag_settings WHERE key = 'embedding_model'",
                [],
                |row| row.get::<_, String>(0),
            ) {
                *manager.embedding_model.blocking_lock() = model;
            }
        }

        Ok(manager)
    }

    /// Get a database connection
    fn get_connection(&self) -> Result<Connection, String> {
        Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open RAG database: {}", e))
    }

    /// Initialize the database with migrations
    fn init_database(&self) -> Result<(), String> {
        let mut conn = self.get_connection()?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

        // Run migrations
        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations
            .to_latest(&mut conn)
            .map_err(|e| format!("Failed to run RAG migrations: {}", e))?;

        info!("RAG database initialized at {:?}", self.db_path);
        Ok(())
    }

    /// Add a document to the knowledge base
    ///
    /// # Arguments
    /// * `content` - The document text content
    /// * `metadata` - Document metadata
    ///
    /// # Returns
    /// The document ID
    pub async fn add_document(
        &self,
        content: &str,
        metadata: DocMetadata,
    ) -> Result<i64, String> {
        let conn = self.get_connection()?;

        // Insert document
        let metadata_json = metadata
            .extra
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();

        conn.execute(
            "INSERT INTO documents (content, source_type, source_id, title, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                content,
                metadata.source_type,
                metadata.source_id,
                metadata.title,
                metadata_json
            ],
        )
        .map_err(|e| format!("Failed to insert document: {}", e))?;

        let document_id = conn.last_insert_rowid();
        debug!("Added document {} to knowledge base", document_id);

        // Generate and store embeddings for the document
        self.index_document(document_id, content).await?;

        Ok(document_id)
    }

    /// Index a document by generating embeddings for its chunks
    async fn index_document(&self, document_id: i64, content: &str) -> Result<(), String> {
        let chunks = self.chunk_text(content);
        let model = self.embedding_model.lock().await.clone();

        debug!(
            "Indexing document {} with {} chunks using model {}",
            document_id,
            chunks.len(),
            model
        );

        for (index, chunk) in chunks.iter().enumerate() {
            if chunk.trim().is_empty() {
                continue;
            }

            // Generate embedding
            let embedding = self
                .ollama_client
                .generate_embeddings(&model, chunk)
                .await?;

            // Store embedding
            let conn = self.get_connection()?;
            let embedding_blob = Self::vec_to_blob(&embedding);

            conn.execute(
                "INSERT INTO embeddings (document_id, chunk_index, chunk_text, embedding, dimensions, model) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    document_id,
                    index as i64,
                    chunk,
                    embedding_blob,
                    embedding.len() as i64,
                    model
                ],
            )
            .map_err(|e| format!("Failed to store embedding: {}", e))?;
        }

        info!(
            "Indexed document {} with {} embedding chunks",
            document_id,
            chunks.len()
        );
        Ok(())
    }

    /// Chunk text into smaller pieces for embedding
    fn chunk_text(&self, text: &str) -> Vec<String> {
        let chunk_size = 500; // characters
        let overlap = 50;

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= chunk_size {
            return vec![text.to_string()];
        }

        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();

            // Try to break at sentence or word boundary
            let chunk = if end < chars.len() {
                if let Some(pos) = chunk.rfind(|c| c == '.' || c == '!' || c == '?') {
                    chunk[..=pos].to_string()
                } else if let Some(pos) = chunk.rfind(' ') {
                    chunk[..pos].to_string()
                } else {
                    chunk
                }
            } else {
                chunk
            };

            if !chunk.trim().is_empty() {
                chunks.push(chunk.clone());
            }

            // Move start position with overlap
            start = start + chunk.len().saturating_sub(overlap);
            if start >= chars.len() {
                break;
            }
        }

        chunks
    }

    /// Search the knowledge base for relevant context
    ///
    /// # Arguments
    /// * `query` - The search query
    /// * `top_k` - Number of results to return
    ///
    /// # Returns
    /// Vector of search results sorted by similarity
    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, String> {
        let model = self.embedding_model.lock().await.clone();

        // Generate query embedding
        let query_embedding = self
            .ollama_client
            .generate_embeddings(&model, query)
            .await?;

        // Search for similar embeddings
        let conn = self.get_connection()?;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    e.document_id,
                    e.chunk_text,
                    e.embedding,
                    d.source_type,
                    d.source_id,
                    d.title,
                    d.metadata
                FROM embeddings e
                JOIN documents d ON e.document_id = d.id
                WHERE e.model = ?1
                "#,
            )
            .map_err(|e| format!("Failed to prepare search query: {}", e))?;

        let mut results: Vec<SearchResult> = stmt
            .query_map(params![model], |row| {
                let document_id: i64 = row.get(0)?;
                let chunk_text: String = row.get(1)?;
                let embedding_blob: Vec<u8> = row.get(2)?;
                let source_type: String = row.get(3)?;
                let source_id: Option<String> = row.get(4)?;
                let title: Option<String> = row.get(5)?;
                let metadata_json: String = row.get(6)?;

                let stored_embedding = Self::blob_to_vec(&embedding_blob);
                let similarity = Self::cosine_similarity(&query_embedding, &stored_embedding);

                let extra: Option<serde_json::Value> = if metadata_json.is_empty() {
                    None
                } else {
                    serde_json::from_str(&metadata_json).ok()
                };

                Ok(SearchResult {
                    document_id,
                    chunk_text,
                    similarity,
                    metadata: DocMetadata {
                        source_type,
                        source_id,
                        title: title.clone(),
                        extra,
                    },
                    title,
                })
            })
            .map_err(|e| format!("Failed to execute search: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        // Sort by similarity (highest first) and take top_k
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        debug!(
            "Search for '{}' returned {} results",
            query.chars().take(50).collect::<String>(),
            results.len()
        );

        Ok(results)
    }

    /// Generate embeddings for text (utility method)
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, String> {
        let model = self.embedding_model.lock().await.clone();
        self.ollama_client.generate_embeddings(&model, text).await
    }

    /// Delete a document and its embeddings
    pub fn delete_document(&self, document_id: i64) -> Result<(), String> {
        let conn = self.get_connection()?;

        // Foreign key cascade will delete embeddings
        conn.execute("DELETE FROM documents WHERE id = ?1", params![document_id])
            .map_err(|e| format!("Failed to delete document: {}", e))?;

        info!("Deleted document {} from knowledge base", document_id);
        Ok(())
    }

    /// List all documents in the knowledge base
    pub fn list_documents(&self) -> Result<Vec<StoredDocument>, String> {
        let conn = self.get_connection()?;

        let mut stmt = conn
            .prepare("SELECT id, content, source_type, source_id, title, created_at FROM documents ORDER BY created_at DESC")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let documents = stmt
            .query_map([], |row| {
                Ok(StoredDocument {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    source_type: row.get(2)?,
                    source_id: row.get(3)?,
                    title: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query documents: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(documents)
    }

    /// Get document count
    pub fn document_count(&self) -> Result<i64, String> {
        let conn = self.get_connection()?;
        conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count documents: {}", e))
    }

    /// Get embedding count
    pub fn embedding_count(&self) -> Result<i64, String> {
        let conn = self.get_connection()?;
        conn.query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count embeddings: {}", e))
    }

    /// Set the embedding model
    pub async fn set_embedding_model(&self, model: &str) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE rag_settings SET value = ?1 WHERE key = 'embedding_model'",
            params![model],
        )
        .map_err(|e| format!("Failed to update embedding model: {}", e))?;

        *self.embedding_model.lock().await = model.to_string();
        info!("Updated embedding model to: {}", model);
        Ok(())
    }

    /// Get the current embedding model
    pub async fn get_embedding_model(&self) -> String {
        self.embedding_model.lock().await.clone()
    }

    /// Clear all documents and embeddings
    pub fn clear_all(&self) -> Result<(), String> {
        let conn = self.get_connection()?;

        conn.execute("DELETE FROM embeddings", [])
            .map_err(|e| format!("Failed to clear embeddings: {}", e))?;

        conn.execute("DELETE FROM documents", [])
            .map_err(|e| format!("Failed to clear documents: {}", e))?;

        warn!("Cleared all documents from knowledge base");
        Ok(())
    }

    /// Convert f32 vector to bytes for storage
    fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
        vec.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    /// Convert bytes back to f32 vector
    fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
        blob.chunks(4)
            .map(|chunk| {
                let bytes: [u8; 4] = chunk.try_into().unwrap_or([0; 4]);
                f32::from_le_bytes(bytes)
            })
            .collect()
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((RagManager::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((RagManager::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);

        let d = vec![-1.0, 0.0, 0.0];
        assert!((RagManager::cosine_similarity(&a, &d) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_vec_blob_roundtrip() {
        let original = vec![1.5, -2.3, 0.0, 4.2];
        let blob = RagManager::vec_to_blob(&original);
        let recovered = RagManager::blob_to_vec(&blob);

        assert_eq!(original.len(), recovered.len());
        for (a, b) in original.iter().zip(recovered.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[test]
    fn test_chunk_text_short() {
        // Create a mock manager for testing (we just need the method)
        let chunks = chunk_text_helper("Short text");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short text");
    }

    // Helper for testing chunking without needing full manager
    fn chunk_text_helper(text: &str) -> Vec<String> {
        let chunk_size = 500;
        let overlap = 50;

        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();

        if chars.len() <= chunk_size {
            return vec![text.to_string()];
        }

        let mut start = 0;
        while start < chars.len() {
            let end = (start + chunk_size).min(chars.len());
            let chunk: String = chars[start..end].iter().collect();

            let chunk = if end < chars.len() {
                if let Some(pos) = chunk.rfind(|c| c == '.' || c == '!' || c == '?') {
                    chunk[..=pos].to_string()
                } else if let Some(pos) = chunk.rfind(' ') {
                    chunk[..pos].to_string()
                } else {
                    chunk
                }
            } else {
                chunk
            };

            if !chunk.trim().is_empty() {
                chunks.push(chunk.clone());
            }

            start = start + chunk.len().saturating_sub(overlap);
            if start >= chars.len() {
                break;
            }
        }

        chunks
    }
}
