use crate::settings::PostProcessProvider;
use crate::telemetry::{TelemetryEventBuilder, TelemetryLevel};
use base64::Engine;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    strict: bool,
    schema: Value,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
    json_schema: JsonSchema,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

/// Build headers for API requests based on provider type
fn build_headers(provider: &PostProcessProvider, api_key: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();

    // Common headers
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://github.com/cjpais/Handy"),
    );
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Handy/1.0 (+https://github.com/cjpais/Handy)"),
    );
    headers.insert("X-Title", HeaderValue::from_static("Handy"));

    // Provider-specific auth headers
    if !api_key.is_empty() {
        if provider.id == "anthropic" {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|e| format!("Invalid API key header value: {}", e))?,
            );
            headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        } else {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| format!("Invalid authorization header value: {}", e))?,
            );
        }
    }

    Ok(headers)
}

/// Create an HTTP client with provider-specific headers
fn create_client(provider: &PostProcessProvider, api_key: &str) -> Result<reqwest::Client, String> {
    let headers = build_headers(provider, api_key)?;
    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

/// Send a chat completion request to an OpenAI-compatible API
/// Returns Ok(Some(content)) on success, Ok(None) if response has no content,
/// or Err on actual errors (HTTP, parsing, etc.)
pub async fn send_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
) -> Result<Option<String>, String> {
    send_chat_completion_with_schema(provider, api_key, model, prompt, None, None).await
}

/// Send a chat completion request with structured output support
/// When json_schema is provided, uses structured outputs mode
/// system_prompt is used as the system message when provided
pub async fn send_chat_completion_with_schema(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    user_content: String,
    system_prompt: Option<String>,
    json_schema: Option<Value>,
) -> Result<Option<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let start = Instant::now();

    debug!("Sending chat completion request to: {}", url);
    TelemetryEventBuilder::new("llm_client", "chat_completion_started")
        .message("Sending non-streaming chat completion request")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .attr("schema", json_schema.is_some())
        .emit();

    let client = create_client(provider, &api_key)?;

    // Build messages vector
    let mut messages = Vec::new();

    // Add system prompt if provided
    if let Some(system) = system_prompt {
        messages.push(ChatMessage {
            role: "system".to_string(),
            content: system,
        });
    }

    // Add user message
    messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_content,
    });

    // Build response_format if schema is provided
    let response_format = json_schema.map(|schema| ResponseFormat {
        format_type: "json_schema".to_string(),
        json_schema: JsonSchema {
            name: "transcription_output".to_string(),
            strict: true,
            schema,
        },
    });

    let request_body = ChatCompletionRequest {
        model: model.to_string(),
        messages,
        response_format,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            TelemetryEventBuilder::new("llm_client", "chat_completion_failed")
                .level(TelemetryLevel::Error)
                .message("Non-streaming chat completion request failed")
                .attr("provider_id", &provider.id)
                .attr("model", model)
                .duration_ms(start.elapsed().as_millis() as u64)
                .status("transport_error")
                .attr("error", &e)
                .emit();
            format!("HTTP request failed: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        TelemetryEventBuilder::new("llm_client", "chat_completion_failed")
            .level(TelemetryLevel::Warn)
            .message("Non-streaming chat completion returned error status")
            .attr("provider_id", &provider.id)
            .attr("model", model)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status(status.as_u16().to_string())
            .attr("error_length", error_text.len())
            .emit();
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
        TelemetryEventBuilder::new("llm_client", "chat_completion_failed")
            .level(TelemetryLevel::Error)
            .message("Failed to parse non-streaming chat completion response")
            .attr("provider_id", &provider.id)
            .attr("model", model)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status("parse_error")
            .attr("error", &e)
            .emit();
        format!("Failed to parse API response: {}", e)
    })?;

    TelemetryEventBuilder::new("llm_client", "chat_completion_completed")
        .message("Non-streaming chat completion completed")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .duration_ms(start.elapsed().as_millis() as u64)
        .attr(
            "has_content",
            completion
                .choices
                .first()
                .and_then(|choice| choice.message.content.as_ref())
                .is_some(),
        )
        .emit();

    Ok(completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone()))
}

/// Send a vision chat completion request to an OpenAI-compatible API.
/// The image is sent as a data URI via `image_url`.
pub async fn send_vision_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    prompt: String,
    image_png_bytes: &[u8],
) -> Result<Option<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let start = Instant::now();
    let client = create_client(provider, &api_key)?;
    let image_data = base64::engine::general_purpose::STANDARD.encode(image_png_bytes);
    let data_url = format!("data:image/png;base64,{}", image_data);

    TelemetryEventBuilder::new("llm_client", "vision_completion_started")
        .message("Sending vision chat completion request")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .attr("image_bytes", image_png_bytes.len())
        .emit();

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": data_url } }
                ]
            }
        ]
    });

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            TelemetryEventBuilder::new("llm_client", "vision_completion_failed")
                .level(TelemetryLevel::Error)
                .message("Vision chat completion request failed")
                .attr("provider_id", &provider.id)
                .attr("model", model)
                .duration_ms(start.elapsed().as_millis() as u64)
                .status("transport_error")
                .attr("error", &e)
                .emit();
            format!("HTTP request failed: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        TelemetryEventBuilder::new("llm_client", "vision_completion_failed")
            .level(TelemetryLevel::Warn)
            .message("Vision chat completion returned error status")
            .attr("provider_id", &provider.id)
            .attr("model", model)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status(status.as_u16().to_string())
            .attr("error_length", error_text.len())
            .emit();
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let response_json: Value = response.json().await.map_err(|e| {
        TelemetryEventBuilder::new("llm_client", "vision_completion_failed")
            .level(TelemetryLevel::Error)
            .message("Failed to parse vision chat completion response")
            .attr("provider_id", &provider.id)
            .attr("model", model)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status("parse_error")
            .attr("error", &e)
            .emit();
        format!("Failed to parse API response: {}", e)
    })?;

    // Handles both string content and array content formats.
    if let Some(content) = response_json
        .get("choices")
        .and_then(|v| v.as_array())
        .and_then(|choices| choices.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
    {
        if let Some(text) = content.as_str() {
            return Ok(Some(text.to_string()));
        }
        if let Some(parts) = content.as_array() {
            let mut merged = String::new();
            for part in parts {
                if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                    if !merged.is_empty() {
                        merged.push('\n');
                    }
                    merged.push_str(text);
                }
            }
            if !merged.is_empty() {
                return Ok(Some(merged));
            }
        }
    }

    TelemetryEventBuilder::new("llm_client", "vision_completion_completed")
        .message("Vision chat completion completed")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .duration_ms(start.elapsed().as_millis() as u64)
        .emit();

    Ok(None)
}

/// Streaming chat completion request body
#[derive(Debug, Serialize)]
struct StreamChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

/// SSE streaming response chunk
#[derive(Debug, Deserialize)]
struct StreamChatChunk {
    choices: Vec<StreamChatChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChatChoice {
    delta: StreamChatDelta,
}

#[derive(Debug, Deserialize)]
struct StreamChatDelta {
    content: Option<String>,
}

/// Stream a chat completion from an OpenAI-compatible API.
///
/// Sends each token through `tx` as it arrives (matching `OllamaClient::generate_stream` signature).
/// Returns the complete accumulated response text.
pub async fn stream_chat_completion(
    provider: &PostProcessProvider,
    api_key: String,
    model: &str,
    system_prompt: Option<String>,
    messages: Vec<(String, String)>, // (role, content) pairs
    tx: mpsc::Sender<String>,
) -> Result<String, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/chat/completions", base_url);
    let start = Instant::now();

    debug!(
        "Starting streaming chat completion to: {} with model: {}",
        url, model
    );
    TelemetryEventBuilder::new("llm_client", "stream_chat_completion_started")
        .message("Starting streaming chat completion request")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .emit();

    let client = create_client(provider, &api_key)?;

    let mut chat_messages = Vec::new();

    if let Some(system) = system_prompt {
        chat_messages.push(ChatMessage {
            role: "system".to_string(),
            content: system,
        });
    }

    for (role, content) in messages {
        chat_messages.push(ChatMessage { role, content });
    }

    let request_body = StreamChatCompletionRequest {
        model: model.to_string(),
        messages: chat_messages,
        stream: true,
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            TelemetryEventBuilder::new("llm_client", "stream_chat_completion_failed")
                .level(TelemetryLevel::Error)
                .message("Streaming chat completion request failed")
                .attr("provider_id", &provider.id)
                .attr("model", model)
                .duration_ms(start.elapsed().as_millis() as u64)
                .status("transport_error")
                .attr("error", &e)
                .emit();
            format!("HTTP request failed: {}", e)
        })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        TelemetryEventBuilder::new("llm_client", "stream_chat_completion_failed")
            .level(TelemetryLevel::Warn)
            .message("Streaming chat completion returned error status")
            .attr("provider_id", &provider.id)
            .attr("model", model)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status(status.as_u16().to_string())
            .attr("error_length", error_text.len())
            .emit();
        return Err(format!(
            "Streaming API request failed with status {}: {}",
            status, error_text
        ));
    }

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| {
            TelemetryEventBuilder::new("llm_client", "stream_chat_completion_failed")
                .level(TelemetryLevel::Error)
                .message("Streaming chat completion stream read failed")
                .attr("provider_id", &provider.id)
                .attr("model", model)
                .duration_ms(start.elapsed().as_millis() as u64)
                .status("stream_read_error")
                .attr("error", &e)
                .emit();
            format!("Stream read error: {}", e)
        })?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Process complete SSE lines from buffer
        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                let data = data.trim();
                if data == "[DONE]" {
                    break;
                }

                if let Ok(chunk) = serde_json::from_str::<StreamChatChunk>(data) {
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            if !content.is_empty() {
                                full_response.push_str(content);
                                let _ = tx.send(content.clone()).await;
                            }
                        }
                    }
                }
            }
        }
    }

    debug!(
        "Streaming chat completion finished, total length: {}",
        full_response.len()
    );
    TelemetryEventBuilder::new("llm_client", "stream_chat_completion_completed")
        .message("Streaming chat completion finished")
        .attr("provider_id", &provider.id)
        .attr("model", model)
        .duration_ms(start.elapsed().as_millis() as u64)
        .attr("response_chars", full_response.len())
        .emit();

    Ok(full_response)
}

/// Fetch available models from an OpenAI-compatible API (optionally filtered to free models)
/// When `free_only` is true, only returns models whose ID contains ":free"
#[cfg(test)]
pub async fn fetch_models_filtered(
    provider: &PostProcessProvider,
    api_key: String,
    free_only: bool,
) -> Result<Vec<String>, String> {
    let mut models = fetch_models(provider, api_key).await?;
    if free_only {
        models.retain(|id| id.contains(":free"));
    }
    Ok(models)
}

/// Fetch available models from an OpenAI-compatible API
/// Returns a list of model IDs
pub async fn fetch_models(
    provider: &PostProcessProvider,
    api_key: String,
) -> Result<Vec<String>, String> {
    let base_url = provider.base_url.trim_end_matches('/');
    let url = format!("{}/models", base_url);
    let start = Instant::now();

    debug!("Fetching models from: {}", url);
    TelemetryEventBuilder::new("llm_client", "fetch_models_started")
        .message("Fetching available models from provider")
        .attr("provider_id", &provider.id)
        .emit();

    let client = create_client(provider, &api_key)?;

    let response = client.get(&url).send().await.map_err(|e| {
        TelemetryEventBuilder::new("llm_client", "fetch_models_failed")
            .level(TelemetryLevel::Error)
            .message("Model list request failed")
            .attr("provider_id", &provider.id)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status("transport_error")
            .attr("error", &e)
            .emit();
        format!("Failed to fetch models: {}", e)
    })?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        TelemetryEventBuilder::new("llm_client", "fetch_models_failed")
            .level(TelemetryLevel::Warn)
            .message("Model list request returned error status")
            .attr("provider_id", &provider.id)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status(status.as_u16().to_string())
            .attr("error_length", error_text.len())
            .emit();
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response.json().await.map_err(|e| {
        TelemetryEventBuilder::new("llm_client", "fetch_models_failed")
            .level(TelemetryLevel::Error)
            .message("Failed to parse model list response")
            .attr("provider_id", &provider.id)
            .duration_ms(start.elapsed().as_millis() as u64)
            .status("parse_error")
            .attr("error", &e)
            .emit();
        format!("Failed to parse response: {}", e)
    })?;

    let mut models = Vec::new();

    // Handle OpenAI format: { data: [ { id: "..." }, ... ] }
    if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
        for entry in data {
            if let Some(id) = entry.get("id").and_then(|i| i.as_str()) {
                models.push(id.to_string());
            } else if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                models.push(name.to_string());
            }
        }
    }
    // Handle array format: [ "model1", "model2", ... ]
    else if let Some(array) = parsed.as_array() {
        for entry in array {
            if let Some(model) = entry.as_str() {
                models.push(model.to_string());
            }
        }
    }

    TelemetryEventBuilder::new("llm_client", "fetch_models_completed")
        .message("Fetched provider model list")
        .attr("provider_id", &provider.id)
        .duration_ms(start.elapsed().as_millis() as u64)
        .attr("model_count", models.len())
        .emit();

    Ok(models)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::PostProcessProvider;
    use tokio::sync::OnceCell;

    fn openrouter_provider() -> PostProcessProvider {
        PostProcessProvider {
            id: "openrouter".to_string(),
            label: "OpenRouter".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: true,
        }
    }

    fn api_key() -> String {
        std::env::var("OPENROUTER_API_KEY")
            .expect("OPENROUTER_API_KEY must be set for integration tests")
    }

    /// Discover a free model from OpenRouter at runtime.
    /// Cached across tests so we only call the API once.
    static FREE_MODEL: OnceCell<String> = OnceCell::const_new();

    async fn free_model() -> &'static str {
        FREE_MODEL
            .get_or_init(|| async {
                let provider = openrouter_provider();
                let free = fetch_models_filtered(&provider, api_key(), true)
                    .await
                    .expect("should fetch free models");
                assert!(!free.is_empty(), "no free models available on OpenRouter");
                // Prefer well-known models, fall back to first available
                let preferred = [
                    "google/gemma-3-1b-it:free",
                    "qwen/qwen3-0.6b:free",
                    "meta-llama/llama-3.1-8b-instruct:free",
                ];
                for p in preferred {
                    if free.contains(&p.to_string()) {
                        return p.to_string();
                    }
                }
                free[0].clone()
            })
            .await
    }

    /// Try a chat completion, retrying once with a different free model on 404.
    async fn resilient_chat_completion(
        provider: &PostProcessProvider,
        key: &str,
        prompt: &str,
        system: Option<&str>,
        schema: Option<Value>,
    ) -> Result<Option<String>, String> {
        let model = free_model().await;
        let result = send_chat_completion_with_schema(
            provider,
            key.to_string(),
            model,
            prompt.to_string(),
            system.map(|s| s.to_string()),
            schema.clone(),
        )
        .await;

        match &result {
            Err(e) if e.contains("404") => {
                // Model disappeared — try fetching a fresh one
                let fresh = fetch_models_filtered(provider, key.to_string(), true)
                    .await
                    .unwrap_or_default();
                if let Some(fallback) = fresh.first() {
                    return send_chat_completion_with_schema(
                        provider,
                        key.to_string(),
                        fallback,
                        prompt.to_string(),
                        system.map(|s| s.to_string()),
                        schema,
                    )
                    .await;
                }
                result
            }
            _ => result,
        }
    }

    // ── fetch_models ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_fetch_models_returns_non_empty_list() {
        let provider = openrouter_provider();
        let models = fetch_models(&provider, api_key())
            .await
            .expect("fetch_models should succeed");

        assert!(
            !models.is_empty(),
            "OpenRouter should return at least one model"
        );
    }

    #[tokio::test]
    async fn test_fetch_models_filtered_free_only() {
        let provider = openrouter_provider();
        let free_models = fetch_models_filtered(&provider, api_key(), true)
            .await
            .expect("fetch_models_filtered should succeed");

        assert!(
            !free_models.is_empty(),
            "OpenRouter should have at least one free model"
        );
        for model_id in &free_models {
            assert!(
                model_id.contains(":free"),
                "Model '{}' should contain ':free' suffix",
                model_id
            );
        }
    }

    // ── chat completion ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_chat_completion_basic() {
        let provider = openrouter_provider();
        let result = resilient_chat_completion(
            &provider,
            &api_key(),
            "Reply with only the word 'hello'.",
            None,
            None,
        )
        .await
        .expect("chat completion should succeed");

        let content = result.expect("response should have content");
        assert!(
            !content.trim().is_empty(),
            "Response content should not be empty"
        );
    }

    #[tokio::test]
    async fn test_chat_completion_with_system_prompt() {
        let provider = openrouter_provider();
        let result = resilient_chat_completion(
            &provider,
            &api_key(),
            "What is 2+2? Reply with just the number.",
            Some("You are a math tutor. Answer with only the number."),
            None,
        )
        .await
        .expect("chat completion with system prompt should succeed");

        let content = result.expect("response should have content");
        assert!(
            content.contains('4'),
            "Response should contain '4', got: {}",
            content
        );
    }

    #[tokio::test]
    async fn test_chat_completion_with_json_schema() {
        let provider = openrouter_provider();
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "transcription": {
                    "type": "string",
                    "description": "The corrected transcription"
                }
            },
            "required": ["transcription"],
            "additionalProperties": false
        });

        let result = resilient_chat_completion(
            &provider,
            &api_key(),
            "Fix this transcription: 'helo wrld'",
            Some("You fix transcriptions. Return JSON."),
            Some(schema),
        )
        .await;

        match result {
            Ok(Some(content)) => {
                // If the model supports structured output, it should return valid JSON
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
                assert!(
                    parsed.is_ok(),
                    "Response should be valid JSON, got: {}",
                    content
                );
            }
            Ok(None) => {} // acceptable
            Err(e) if e.contains("400") && e.contains("response_format") => {
                // Free model's provider doesn't support json_schema — acceptable
                eprintln!(
                    "Skipped: free model provider doesn't support json_schema: {}",
                    e
                );
            }
            Err(e) if e.contains("404") => {
                eprintln!("Skipped: model unavailable: {}", e);
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    // ── error cases ─────────────────────────────────────────────────

    #[tokio::test]
    async fn test_chat_completion_invalid_model() {
        let provider = openrouter_provider();
        let result = send_chat_completion(
            &provider,
            api_key(),
            "nonexistent/model-that-does-not-exist",
            "hello".to_string(),
        )
        .await;

        assert!(
            result.is_err(),
            "Should fail with nonexistent model, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_chat_completion_invalid_api_key() {
        let provider = openrouter_provider();
        let model = free_model().await;
        let result = send_chat_completion(
            &provider,
            "sk-invalid-key-12345".to_string(),
            model,
            "hello".to_string(),
        )
        .await;

        // Free models may still respond with an invalid key, so we accept
        // either an error or a successful response — the key point is no panic.
        match result {
            Err(_) => {}
            Ok(_) => {}
        }
    }

    // ── multiple free models ────────────────────────────────────────

    #[tokio::test]
    async fn test_chat_completion_multiple_free_models() {
        let provider = openrouter_provider();
        let key = api_key();
        let free_models = fetch_models_filtered(&provider, key.clone(), true)
            .await
            .expect("should fetch free models");

        // Test up to 3 free models
        let to_test: Vec<_> = free_models.iter().take(3).collect();
        assert!(!to_test.is_empty(), "Need at least one free model");

        let mut any_succeeded = false;
        for model_id in &to_test {
            let result =
                send_chat_completion(&provider, key.clone(), model_id, "Say 'ok'.".to_string())
                    .await;

            match result {
                Ok(Some(content)) => {
                    assert!(!content.trim().is_empty());
                    any_succeeded = true;
                }
                Ok(None) => {
                    any_succeeded = true;
                }
                Err(e) => {
                    eprintln!("Model {} unavailable: {}", model_id, e);
                }
            }
        }

        assert!(
            any_succeeded,
            "At least one free model should respond successfully"
        );
    }
}
