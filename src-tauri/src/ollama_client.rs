//! Ollama API client for Active Listening feature
//!
//! Provides HTTP client for communicating with local Ollama server,
//! supporting streaming responses for real-time insight generation.

use futures_util::StreamExt;
use log::{debug, error, warn};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Ollama generate request payload
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Ollama embeddings request payload
#[derive(Debug, Serialize)]
struct OllamaEmbeddingsRequest {
    model: String,
    prompt: String,
}

/// Ollama embeddings response
#[derive(Debug, Deserialize)]
struct OllamaEmbeddingsResponse {
    embedding: Vec<f32>,
}

/// Ollama model options
#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
}

/// Ollama streaming response chunk
#[derive(Debug, Deserialize)]
struct OllamaStreamResponse {
    response: String,
    done: bool,
    // The following fields are part of Ollama's API response but not currently used.
    // They're kept for API completeness and potential future use (metrics, multi-turn context).
    #[serde(default)]
    #[allow(dead_code)]
    context: Option<Vec<i64>>,
    #[serde(default)]
    #[allow(dead_code)]
    total_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    eval_count: Option<u64>,
}

/// Ollama model info from /api/tags
#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

/// Individual model info
#[derive(Debug, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    // The following fields are part of Ollama's API response but not currently used.
    // They're kept for API completeness and potential future use (model info display).
    #[serde(default)]
    #[allow(dead_code)]
    pub size: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub digest: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub modified_at: String,
}

/// Ollama API client
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

/// Default timeout for Ollama API requests (5 minutes for long-running generation)
const DEFAULT_TIMEOUT_SECS: u64 = 300;
/// Connection timeout (10 seconds)
const CONNECT_TIMEOUT_SECS: u64 = 10;

impl OllamaClient {
    /// Create a new Ollama client with the given base URL
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(base_url: &str) -> Result<Self, String> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Check if Ollama server is available
    pub async fn health_check(&self) -> Result<bool, String> {
        let url = format!("{}/api/tags", self.base_url);
        debug!("Checking Ollama health at: {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!("Ollama health check passed");
                    Ok(true)
                } else {
                    let status = response.status();
                    warn!("Ollama health check failed with status: {}", status);
                    Ok(false)
                }
            }
            Err(e) => {
                debug!("Ollama health check failed: {}", e);
                // Return Ok(false) for connection errors - server is just not available
                if e.is_connect() {
                    Ok(false)
                } else {
                    Err(format!("Failed to connect to Ollama: {}", e))
                }
            }
        }
    }

    /// List available models from Ollama
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);
        debug!("Fetching Ollama models from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Failed to list models ({}): {}",
                status, error_text
            ));
        }

        let tags: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse models response: {}", e))?;

        let model_names: Vec<String> = tags.models.into_iter().map(|m| m.name).collect();
        debug!("Found {} Ollama models", model_names.len());

        Ok(model_names)
    }

    /// Generate text with streaming response
    ///
    /// Sends chunks through the provided channel as they arrive.
    /// Returns the complete response text when done.
    pub async fn generate_stream(
        &self,
        model: &str,
        prompt: String,
        tx: mpsc::Sender<String>,
    ) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        debug!(
            "Starting Ollama streaming generate to: {} with model: {}",
            url, model
        );

        let request_body = OllamaGenerateRequest {
            model: model.to_string(),
            prompt,
            stream: true,
            options: Some(OllamaOptions {
                temperature: 0.7,
                num_ctx: Some(4096),
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send generate request: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Generate request failed ({}): {}",
                status, error_text
            ));
        }

        let mut complete_response = String::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    // Ollama sends newline-delimited JSON
                    let chunk_str = String::from_utf8_lossy(&bytes);
                    for line in chunk_str.lines() {
                        if line.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<OllamaStreamResponse>(line) {
                            Ok(stream_response) => {
                                if !stream_response.response.is_empty() {
                                    complete_response.push_str(&stream_response.response);

                                    // Send chunk through channel
                                    if tx.send(stream_response.response).await.is_err() {
                                        debug!("Receiver dropped, stopping stream");
                                        return Ok(complete_response);
                                    }
                                }

                                if stream_response.done {
                                    debug!("Ollama stream completed");
                                    return Ok(complete_response);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse stream chunk: {} - line: {}", e, line);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading stream chunk: {}", e);
                    return Err(format!("Stream read error: {}", e));
                }
            }
        }

        Ok(complete_response)
    }

    /// Generate text without streaming (blocking call)
    /// Currently unused but kept as a utility for batch processing or testing.
    #[allow(dead_code)]
    pub async fn generate(&self, model: &str, prompt: String) -> Result<String, String> {
        let url = format!("{}/api/generate", self.base_url);
        debug!("Starting Ollama generate to: {} with model: {}", url, model);

        let request_body = OllamaGenerateRequest {
            model: model.to_string(),
            prompt,
            stream: false,
            options: Some(OllamaOptions {
                temperature: 0.7,
                num_ctx: Some(4096),
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send generate request: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Generate request failed ({}): {}",
                status, error_text
            ));
        }

        let stream_response: OllamaStreamResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(stream_response.response)
    }

    /// Generate embeddings for text using Ollama
    ///
    /// Uses the /api/embeddings endpoint to generate vector embeddings.
    /// Recommended models: nomic-embed-text, all-minilm
    pub async fn generate_embeddings(
        &self,
        model: &str,
        text: &str,
    ) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/embeddings", self.base_url);
        debug!(
            "Generating embeddings with model {} for text of length {}",
            model,
            text.len()
        );

        let request_body = OllamaEmbeddingsRequest {
            model: model.to_string(),
            prompt: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send embeddings request: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!(
                "Embeddings request failed ({}): {}",
                status, error_text
            ));
        }

        let embeddings_response: OllamaEmbeddingsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse embeddings response: {}", e))?;

        debug!(
            "Generated embedding with {} dimensions",
            embeddings_response.embedding.len()
        );

        Ok(embeddings_response.embedding)
    }
}

/// Apply template variables to a prompt template
///
/// Supported variables:
/// - {{transcription}} - The current segment transcription
/// - {{previous_context}} - Summary of previous segments
/// - {{session_topic}} - User-defined session topic
/// - {{retrieved_context}} - RAG-retrieved relevant context from knowledge base
pub fn apply_prompt_template(
    template: &str,
    transcription: &str,
    previous_context: &str,
    session_topic: Option<&str>,
) -> String {
    apply_prompt_template_with_rag(template, transcription, previous_context, session_topic, None)
}

/// Apply template variables including RAG context
///
/// Supported variables:
/// - {{transcription}} - The current segment transcription
/// - {{previous_context}} - Summary of previous segments
/// - {{session_topic}} - User-defined session topic
/// - {{retrieved_context}} - RAG-retrieved relevant context from knowledge base
pub fn apply_prompt_template_with_rag(
    template: &str,
    transcription: &str,
    previous_context: &str,
    session_topic: Option<&str>,
    retrieved_context: Option<&str>,
) -> String {
    let mut result = template.to_string();

    result = result.replace("{{transcription}}", transcription);
    result = result.replace("{{previous_context}}", previous_context);
    result = result.replace(
        "{{session_topic}}",
        session_topic.unwrap_or("Not specified"),
    );
    result = result.replace(
        "{{retrieved_context}}",
        retrieved_context.unwrap_or("No additional context available"),
    );

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_prompt_template() {
        let template = "Transcription: {{transcription}}\nContext: {{previous_context}}\nTopic: {{session_topic}}";
        let result = apply_prompt_template(
            template,
            "Hello world",
            "Previous discussion about AI",
            Some("AI Meeting"),
        );

        assert!(result.contains("Hello world"));
        assert!(result.contains("Previous discussion about AI"));
        assert!(result.contains("AI Meeting"));
    }

    #[test]
    fn test_apply_prompt_template_no_topic() {
        let template = "Topic: {{session_topic}}";
        let result = apply_prompt_template(template, "", "", None);

        assert!(result.contains("Not specified"));
    }

    #[test]
    fn test_apply_prompt_template_empty_template() {
        let template = "";
        let result = apply_prompt_template(template, "hello", "context", Some("topic"));
        assert_eq!(result, "");
    }

    #[test]
    fn test_apply_prompt_template_no_variables() {
        let template = "This is a static prompt with no variables.";
        let result = apply_prompt_template(template, "hello", "context", Some("topic"));
        assert_eq!(result, "This is a static prompt with no variables.");
    }

    #[test]
    fn test_apply_prompt_template_multiple_same_variable() {
        let template = "{{transcription}} - {{transcription}}";
        let result = apply_prompt_template(template, "test", "", None);
        assert_eq!(result, "test - test");
    }

    #[test]
    fn test_apply_prompt_template_multiline_transcription() {
        let template = "Text: {{transcription}}";
        let result = apply_prompt_template(template, "line1\nline2\nline3", "", None);
        assert!(result.contains("line1\nline2\nline3"));
    }

    #[test]
    fn test_apply_prompt_template_special_characters() {
        let template = "{{transcription}}";
        let result = apply_prompt_template(template, "Hello! @#$%^&*()_+", "", None);
        assert_eq!(result, "Hello! @#$%^&*()_+");
    }

    #[test]
    fn test_apply_prompt_template_unicode() {
        let template = "{{transcription}}";
        let result = apply_prompt_template(template, "你好世界 🌍", "", None);
        assert_eq!(result, "你好世界 🌍");
    }

    #[test]
    fn test_apply_prompt_template_all_variables() {
        let template = "T:{{transcription}}|C:{{previous_context}}|S:{{session_topic}}";
        let result = apply_prompt_template(template, "trans", "ctx", Some("topic"));
        assert_eq!(result, "T:trans|C:ctx|S:topic");
    }

    #[test]
    fn test_apply_prompt_template_preserves_formatting() {
        let template = "  {{transcription}}  ";
        let result = apply_prompt_template(template, "text", "", None);
        assert_eq!(result, "  text  ");
    }

    #[test]
    fn test_apply_prompt_template_with_rag_context() {
        let template = "Transcription: {{transcription}}\nKnowledge: {{retrieved_context}}";
        let result = apply_prompt_template_with_rag(
            template,
            "Hello",
            "",
            None,
            Some("Relevant facts from knowledge base"),
        );
        assert!(result.contains("Hello"));
        assert!(result.contains("Relevant facts from knowledge base"));
    }

    #[test]
    fn test_apply_prompt_template_with_rag_no_context() {
        let template = "Knowledge: {{retrieved_context}}";
        let result = apply_prompt_template_with_rag(template, "", "", None, None);
        assert!(result.contains("No additional context available"));
    }

    #[test]
    fn test_apply_prompt_template_with_rag_all_variables() {
        let template = "T:{{transcription}}|C:{{previous_context}}|S:{{session_topic}}|R:{{retrieved_context}}";
        let result = apply_prompt_template_with_rag(
            template,
            "trans",
            "ctx",
            Some("topic"),
            Some("rag"),
        );
        assert_eq!(result, "T:trans|C:ctx|S:topic|R:rag");
    }
}
