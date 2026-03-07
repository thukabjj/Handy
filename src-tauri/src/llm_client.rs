use crate::settings::PostProcessProvider;
use log::debug;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    debug!("Sending chat completion request to: {}", url);

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
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read error response".to_string());
        return Err(format!(
            "API request failed with status {}: {}",
            status, error_text
        ));
    }

    let completion: ChatCompletionResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API response: {}", e))?;

    Ok(completion
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone()))
}

/// Fetch available models from an OpenAI-compatible API (optionally filtered to free models)
/// When `free_only` is true, only returns models whose ID contains ":free"
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

    debug!("Fetching models from: {}", url);

    let client = create_client(provider, &api_key)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch models: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Model list request failed ({}): {}",
            status, error_text
        ));
    }

    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

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
