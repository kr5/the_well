//! Thin `reqwest` wrappers over two interchangeable translation backends —
//! no official Rust SDK exists for either, matching this project's
//! established pattern (`bot-whatsapp`'s Meta Cloud API client,
//! `ivr-gateway`'s Exotel client) of a small hand-rolled client over a
//! well-documented, stable HTTP API rather than pulling in an
//! unofficial/unmaintained crate for a handful of calls.
//!
//! Two backends exist because an `ANTHROPIC_API_KEY` isn't always
//! available (it's a metered, paid key), while NVIDIA's NIM API
//! (`https://integrate.api.nvidia.com`) issues free-tier keys usable
//! against hosted open models, including several Nemotron variants —
//! a reasonable substitute for exactly this "high-volume, lower-stakes
//! draft translation" use case (never auto-published — see this module's
//! callers). `TranslationClient::from_env` picks whichever backend has a
//! key configured, so the rest of the pipeline (`mod.rs`, `kb_entry.rs`,
//! `tree.rs`) is written against one interface and doesn't care which
//! backend actually ran.

use serde_json::json;

/// Per this session's own model roster: Haiku 4.5 is the current, fastest
/// Claude model — the right tradeoff for high-volume, lower-stakes
/// machine-translation drafts. Overridable via `ANTHROPIC_TRANSLATION_MODEL`
/// so this doesn't go stale as models turn over.
pub const DEFAULT_CLAUDE_MODEL: &str = "claude-haiku-4-5-20251001";

/// NVIDIA NIM's hosted Nemotron instruct model — a solid general-purpose,
/// free-tier-accessible instruction-following model for the same
/// draft-translation task. Overridable via `NEMOTRON_TRANSLATION_MODEL`
/// (NVIDIA's catalog includes several other sizes/variants, e.g. the
/// smaller `nvidia/llama-3.1-nemotron-nano-8b-v1`, at
/// https://build.nvidia.com).
pub const DEFAULT_NEMOTRON_MODEL: &str = "nvidia/llama-3.1-nemotron-70b-instruct";

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error(
        "set ANTHROPIC_API_KEY (to use Claude) or NVIDIA_API_KEY (to use NVIDIA NIM's free-tier \
         Nemotron models) before running the translation pipeline"
    )]
    MissingApiKey,
    #[error("TRANSLATION_PROVIDER=\"{0}\" is not recognized — expected \"anthropic\" or \"nemotron\"")]
    UnknownProvider(String),
    #[error("request to the {provider} API failed: {source}")]
    Request { provider: &'static str, source: reqwest::Error },
    #[error("{provider} API returned {status}: {body}")]
    ApiError { provider: &'static str, status: reqwest::StatusCode, body: String },
    #[error("unexpected response shape from the {provider} API: {body}")]
    UnexpectedResponse { provider: &'static str, body: String },
}

/// One translation backend, selected once at startup by `from_env` and
/// used for every call in a single pipeline run — the pipeline never
/// mixes backends within one `xtask translate-*` invocation.
pub enum TranslationClient {
    Claude(ClaudeClient),
    Nemotron(NemotronClient),
}

impl TranslationClient {
    /// Backend selection: `TRANSLATION_PROVIDER=anthropic`/`=nemotron`
    /// forces a specific backend (and errors if that backend's key isn't
    /// set); otherwise, whichever API key is present wins, checking
    /// `NVIDIA_API_KEY` first since it's the free-tier option and the one
    /// more likely to be available in a given environment. Erroring here
    /// (rather than silently falling back) means a typo'd
    /// `TRANSLATION_PROVIDER` value fails loudly instead of silently
    /// running the wrong model.
    pub fn from_env() -> Result<Self, TranslationError> {
        match std::env::var("TRANSLATION_PROVIDER").ok().as_deref() {
            Some("anthropic") => return ClaudeClient::from_env().map(Self::Claude),
            Some("nemotron") => return NemotronClient::from_env().map(Self::Nemotron),
            Some(other) => return Err(TranslationError::UnknownProvider(other.to_string())),
            None => {}
        }

        if std::env::var("NVIDIA_API_KEY").is_ok() {
            NemotronClient::from_env().map(Self::Nemotron)
        } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            ClaudeClient::from_env().map(Self::Claude)
        } else {
            Err(TranslationError::MissingApiKey)
        }
    }

    pub fn provider_name(&self) -> &'static str {
        match self {
            Self::Claude(_) => "Anthropic Claude",
            Self::Nemotron(_) => "NVIDIA NIM (Nemotron)",
        }
    }

    /// Sends one single-turn request and returns the model's full text
    /// reply. Every caller in this module is a one-shot "translate this
    /// string" request — no conversation history, no tool use.
    pub async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String, TranslationError> {
        match self {
            Self::Claude(client) => client.complete(system_prompt, user_message).await,
            Self::Nemotron(client) => client.complete(system_prompt, user_message).await,
        }
    }
}

pub struct ClaudeClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl ClaudeClient {
    fn from_env() -> Result<Self, TranslationError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| TranslationError::MissingApiKey)?;
        let model = std::env::var("ANTHROPIC_TRANSLATION_MODEL").unwrap_or_else(|_| DEFAULT_CLAUDE_MODEL.to_string());
        Ok(Self { http: reqwest::Client::new(), api_key, model })
    }

    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String, TranslationError> {
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
            .await
            .map_err(|source| TranslationError::Request { provider: "Anthropic", source })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslationError::ApiError { provider: "Anthropic", status, body });
        }

        let body: serde_json::Value =
            response.json().await.map_err(|source| TranslationError::Request { provider: "Anthropic", source })?;
        body["content"][0]["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| TranslationError::UnexpectedResponse { provider: "Anthropic", body: body.to_string() })
    }
}

pub struct NemotronClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl NemotronClient {
    fn from_env() -> Result<Self, TranslationError> {
        let api_key = std::env::var("NVIDIA_API_KEY").map_err(|_| TranslationError::MissingApiKey)?;
        let model = std::env::var("NEMOTRON_TRANSLATION_MODEL").unwrap_or_else(|_| DEFAULT_NEMOTRON_MODEL.to_string());
        Ok(Self { http: reqwest::Client::new(), api_key, model })
    }

    /// NVIDIA NIM's `/v1/chat/completions` endpoint is OpenAI-compatible —
    /// same request/response shape as the OpenAI Chat Completions API,
    /// just a different base URL, bearer token, and model catalog.
    async fn complete(&self, system_prompt: &str, user_message: &str) -> Result<String, TranslationError> {
        let response = self
            .http
            .post("https://integrate.api.nvidia.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .header("accept", "application/json")
            .json(&json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_message},
                ],
                "temperature": 0.2,
                "top_p": 1,
                "max_tokens": 4096,
                "stream": false,
            }))
            .send()
            .await
            .map_err(|source| TranslationError::Request { provider: "NVIDIA NIM", source })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(TranslationError::ApiError { provider: "NVIDIA NIM", status, body });
        }

        let body: serde_json::Value =
            response.json().await.map_err(|source| TranslationError::Request { provider: "NVIDIA NIM", source })?;
        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| TranslationError::UnexpectedResponse { provider: "NVIDIA NIM", body: body.to_string() })
    }
}
