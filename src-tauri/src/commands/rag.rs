//! Tauri commands for RAG (Knowledge Base) functionality

use crate::managers::rag::{DocMetadata, RagManager, SearchResult, StoredDocument};
use crate::ollama_client::OllamaClient;
use crate::settings::{get_settings, write_settings, KnowledgeBaseSettings};
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Add a document to the knowledge base
#[tauri::command]
#[specta::specta]
pub async fn rag_add_document(
    rag_manager: State<'_, Arc<RagManager>>,
    content: String,
    source_type: String,
    source_id: Option<String>,
    title: Option<String>,
) -> Result<i64, String> {
    let metadata = DocMetadata {
        source_type,
        source_id,
        title,
        extra: None,
    };

    rag_manager.add_document(&content, metadata).await
}

/// Add a document from a file path (.txt/.md/.pdf)
#[tauri::command]
#[specta::specta]
pub async fn rag_add_document_from_file(
    rag_manager: State<'_, Arc<RagManager>>,
    path: String,
) -> Result<i64, String> {
    let content = read_document_text(Path::new(&path))?;
    if content.trim().is_empty() {
        return Err("No extractable text found in document".to_string());
    }

    let title = Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string());

    let metadata = DocMetadata {
        source_type: "file".to_string(),
        source_id: Some(path),
        title,
        extra: None,
    };

    rag_manager.add_document(&content, metadata).await
}

/// Search the knowledge base for relevant context
#[tauri::command]
#[specta::specta]
pub async fn rag_search(
    rag_manager: State<'_, Arc<RagManager>>,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<SearchResult>, String> {
    let k = top_k.unwrap_or(3);
    rag_manager.search(&query, k).await
}

/// Delete a document from the knowledge base
#[tauri::command]
#[specta::specta]
pub fn rag_delete_document(
    rag_manager: State<'_, Arc<RagManager>>,
    document_id: i64,
) -> Result<(), String> {
    rag_manager.delete_document(document_id)
}

/// List all documents in the knowledge base
#[tauri::command]
#[specta::specta]
pub fn rag_list_documents(
    rag_manager: State<'_, Arc<RagManager>>,
) -> Result<Vec<StoredDocument>, String> {
    rag_manager.list_documents()
}

/// Get knowledge base statistics
#[tauri::command]
#[specta::specta]
pub fn rag_get_stats(rag_manager: State<'_, Arc<RagManager>>) -> Result<RagStats, String> {
    let document_count = rag_manager.document_count()?;
    let embedding_count = rag_manager.embedding_count()?;

    Ok(RagStats {
        document_count,
        embedding_count,
    })
}

/// Get the current embedding model
#[tauri::command]
#[specta::specta]
pub async fn rag_get_embedding_model(
    rag_manager: State<'_, Arc<RagManager>>,
) -> Result<String, String> {
    Ok(rag_manager.get_embedding_model().await)
}

/// Set the embedding model
#[tauri::command]
#[specta::specta]
pub async fn rag_set_embedding_model(
    rag_manager: State<'_, Arc<RagManager>>,
    model: String,
) -> Result<(), String> {
    rag_manager.set_embedding_model(&model).await
}

/// Clear all documents from the knowledge base
#[tauri::command]
#[specta::specta]
pub fn rag_clear_all(rag_manager: State<'_, Arc<RagManager>>) -> Result<(), String> {
    rag_manager.clear_all()
}

/// Chat with the knowledge base using RAG context
#[tauri::command]
#[specta::specta]
pub async fn rag_chat(
    app: AppHandle,
    rag_manager: State<'_, Arc<RagManager>>,
    ollama_client: State<'_, Arc<OllamaClient>>,
    question: String,
    context: Option<String>,
) -> Result<String, String> {
    let settings = get_settings(&app);

    // Search for relevant context
    let search_results = rag_manager.search(&question, 3).await?;

    // Build context from search results
    let mut rag_context = String::new();
    for result in &search_results {
        rag_context.push_str(&result.chunk_text);
        rag_context.push_str("\n\n");
    }

    // Append any additional context provided
    if let Some(ctx) = context {
        rag_context.push_str(&ctx);
    }

    if rag_context.is_empty() {
        return Err("No relevant context found in knowledge base".to_string());
    }

    // Use the Ollama client to generate a response
    let prompt = format!(
        "Based on the following context, answer the question.\n\nContext:\n{}\n\nQuestion: {}",
        rag_context, question
    );

    let model = &settings.active_listening.ollama_model;
    ollama_client.generate(model, prompt).await
}

/// Knowledge base statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct RagStats {
    pub document_count: i64,
    pub embedding_count: i64,
}

/// Get knowledge base settings
#[tauri::command]
#[specta::specta]
pub fn get_knowledge_base_settings(app: AppHandle) -> Result<KnowledgeBaseSettings, String> {
    let settings = get_settings(&app);
    Ok(settings.knowledge_base)
}

/// Update knowledge base enabled setting
#[tauri::command]
#[specta::specta]
pub fn change_knowledge_base_enabled_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.knowledge_base.enabled = enabled;
    write_settings(&app, settings);
    Ok(())
}

/// Update auto-index transcriptions setting
#[tauri::command]
#[specta::specta]
pub fn change_auto_index_transcriptions_setting(
    app: AppHandle,
    auto_index: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.knowledge_base.auto_index_transcriptions = auto_index;
    write_settings(&app, settings);
    Ok(())
}

/// Update embedding model setting
#[tauri::command]
#[specta::specta]
pub async fn change_kb_embedding_model_setting(
    app: AppHandle,
    rag_manager: State<'_, Arc<RagManager>>,
    model: String,
) -> Result<(), String> {
    // Update both settings and RAG manager
    let mut settings = get_settings(&app);
    settings.knowledge_base.embedding_model = model.clone();
    write_settings(&app, settings);

    // Also update the RAG manager's model
    rag_manager.set_embedding_model(&model).await
}

/// Update top_k setting
#[tauri::command]
#[specta::specta]
pub fn change_kb_top_k_setting(app: AppHandle, top_k: usize) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.knowledge_base.top_k = top_k;
    write_settings(&app, settings);
    Ok(())
}

/// Update similarity threshold setting
#[tauri::command]
#[specta::specta]
pub fn change_kb_similarity_threshold_setting(
    app: AppHandle,
    threshold: f32,
) -> Result<(), String> {
    if !(0.0..=1.0).contains(&threshold) {
        return Err("Similarity threshold must be between 0.0 and 1.0".to_string());
    }
    let mut settings = get_settings(&app);
    settings.knowledge_base.similarity_threshold = threshold;
    write_settings(&app, settings);
    Ok(())
}

/// Update use in active listening setting
#[tauri::command]
#[specta::specta]
pub fn change_kb_use_in_active_listening_setting(
    app: AppHandle,
    use_in_al: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.knowledge_base.use_in_active_listening = use_in_al;
    write_settings(&app, settings);
    Ok(())
}

fn read_document_text(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" | "md" | "markdown" => std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read text document: {}", e)),
        "pdf" => {
            let bytes =
                std::fs::read(path).map_err(|e| format!("Failed to read PDF file: {}", e))?;
            Ok(extract_pdf_text_lossy(&bytes))
        }
        _ => Err("Unsupported file type. Supported: .txt, .md, .pdf".to_string()),
    }
}

fn extract_pdf_text_lossy(bytes: &[u8]) -> String {
    // Lightweight extraction fallback that captures literal strings in PDF content streams.
    // This is intentionally dependency-free and works well enough for many simple PDFs.
    let mut out = String::new();
    let mut in_text = false;
    let mut escaped = false;
    let mut buf = String::new();

    for &b in bytes {
        let c = b as char;
        if !in_text {
            if c == '(' {
                in_text = true;
                buf.clear();
            }
            continue;
        }

        if escaped {
            buf.push(c);
            escaped = false;
            continue;
        }

        match c {
            '\\' => escaped = true,
            ')' => {
                in_text = false;
                if buf.len() >= 3 && buf.chars().any(|ch| ch.is_alphabetic()) {
                    out.push_str(buf.trim());
                    out.push('\n');
                }
            }
            _ => {
                if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                    buf.push(c);
                }
            }
        }
    }

    if out.trim().is_empty() {
        // Fallback for PDFs where literals are not extracted by the simple parser.
        let raw = String::from_utf8_lossy(bytes);
        raw.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.len() > 20 && trimmed.chars().filter(|c| c.is_alphabetic()).count() > 8
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        out
    }
}
