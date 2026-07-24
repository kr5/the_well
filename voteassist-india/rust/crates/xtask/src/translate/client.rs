//! Thin `reqwest` wrapper over the Anthropic Messages API — no official
//! Rust SDK exists (Anthropic's SDKs are Python/TypeScript/Java/Go),
//! matching this project's established pattern (`bot-whatsapp`'s Meta
//! Cloud API client, `ivr-gateway`'s Exotel client) of a small hand-rolled
//! client over a well-documented, stable HTTP API rather than pulling in
//! an unofficial/unmaintained crate for a handful of calls.

use serde_json::json;

/// Per this session's own model roster: Haiku 4.5 is the current, fastest
/// Claude model — the right tradeoff for high-volume, lower-stakes
/// machine-translation drafts (never auto-published — see this module's
/// callers) rather than a slower/costlier model. Overridable via
/// `ANTHROPIC_TRANSLATION_MODEL` so this doesn't go stale as models turn
/// over.
pub const DEFAULT_MODEL: &str = "claude-haiku-4-5-20251001";

#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("ANTHROPIC_API_KEY must be set")]
    MissingApiKey,
    #[error("request to the Anthropic API failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Anthropic API returned {status}: {body}")]
    ApiError { status: reqwest::StatusCode, body: String },
    #[error("unexpected response shape from the Anthropic API: {0}")]
    UnexpectedResponse(String),
}

pub struct ClaudeClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeClient {
    pub fn from_env() -> Result<Self, ClaudeError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| ClaudeError::MissingApiKey)?;
        let model = std::env::var("ANTHROPIC_TRANSLATION_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
        Ok(Self { http: reqwest::Client::new(), api_key, model })
    }

    /// Sends one single-turn request and returns Claude's full text reply.
    /// Every caller in this module is a one-shot "translate this string"
    /// request — no conversation history, no tool use.
    pub async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String, ClaudeError> {
        let response = self
            .http
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.model,
                "max_tokens": 4096,
                "system": system_prompt,
                "messages": [{"role": "user", "content": user_message}],
            }))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ClaudeError::ApiError { status, body });
        }

        let body: serde_json::Value = response.json().await?;
        body["content"][0]["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| ClaudeError::UnexpectedResponse(body.to_string()))
    }
}
