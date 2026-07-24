//! Thin `reqwest` wrapper over the Meta WhatsApp Cloud API's Graph API
//! endpoints, per docs/PRD-V2-RUST-PLATFORM.md Section 6.7: "no mature
//! Rust SDK exists [...] a thin wrapper is the right size." This module
//! only knows how to serialize/deserialize Meta's JSON shapes and make
//! the HTTP call — it holds no decision logic and no session state.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Meta's documented Graph API versioning scheme (`vMAJOR.MINOR`, e.g.
/// `v21.0`) changes periodically; kept configurable via
/// `WHATSAPP_GRAPH_API_VERSION` (see `main.rs`) rather than hard-coded, so
/// an operator can bump it without a code change when Meta deprecates an
/// older version.
pub const DEFAULT_GRAPH_API_VERSION: &str = "v21.0";

#[derive(Debug, Clone)]
pub struct WhatsAppClient {
    http: reqwest::Client,
    graph_api_version: String,
    phone_number_id: String,
    access_token: String,
}

#[derive(Debug, thiserror::Error)]
pub enum WhatsAppApiError {
    #[error("HTTP transport error calling the WhatsApp Cloud API: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("WhatsApp Cloud API returned an error (code {code:?}): {message}")]
    Api { message: String, code: Option<i64> },
}

#[derive(Debug, Deserialize)]
struct SendMessageResponse {
    #[serde(default)]
    error: Option<GraphApiError>,
}

#[derive(Debug, Deserialize)]
struct GraphApiError {
    message: String,
    #[serde(default)]
    code: Option<i64>,
}

/// One row of an interactive "list" message
/// (`interactive.action.sections[].rows[]`). `id` becomes the
/// `interactive.list_reply.id` this crate's webhook handler reads back
/// when the citizen taps it — the same decision-tree option `value` that
/// `channel_core::render_node` produced.
#[derive(Debug, Clone)]
pub struct ListRow {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

/// One reply button of an interactive "button" message (max 3 per Meta's
/// limit — see `render_whatsapp::render_question`, which picks buttons vs.
/// a list based on option count).
#[derive(Debug, Clone)]
pub struct ReplyButton {
    pub id: String,
    pub title: String,
}

impl WhatsAppClient {
    pub fn new(phone_number_id: String, access_token: String, graph_api_version: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            graph_api_version,
            phone_number_id,
            access_token,
        }
    }

    fn messages_url(&self) -> String {
        format!(
            "https://graph.facebook.com/{}/{}/messages",
            self.graph_api_version, self.phone_number_id
        )
    }

    async fn post(&self, body: Value) -> Result<(), WhatsAppApiError> {
        let response = self
            .http
            .post(self.messages_url())
            .bearer_auth(&self.access_token)
            .json(&body)
            .send()
            .await?;

        let parsed: SendMessageResponse = response.json().await?;
        if let Some(error) = parsed.error {
            return Err(WhatsAppApiError::Api { message: error.message, code: error.code });
        }
        Ok(())
    }

    pub async fn send_text(&self, to: &str, body: &str) -> Result<(), WhatsAppApiError> {
        self.post(json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "text",
            "text": { "body": body },
        }))
        .await
    }

    /// Sends an interactive reply-button message (2-3 options — Meta caps
    /// buttons at 3). Prefer `send_interactive_list` above 3 options.
    pub async fn send_interactive_buttons(
        &self,
        to: &str,
        body_text: &str,
        buttons: &[ReplyButton],
    ) -> Result<(), WhatsAppApiError> {
        let button_values: Vec<Value> = buttons
            .iter()
            .map(|b| {
                json!({
                    "type": "reply",
                    "reply": { "id": b.id, "title": b.title },
                })
            })
            .collect();

        self.post(json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "interactive",
            "interactive": {
                "type": "button",
                "body": { "text": body_text },
                "action": { "buttons": button_values },
            },
        }))
        .await
    }

    /// Sends an interactive list message (up to 10 rows total across
    /// sections — this wrapper always uses a single section, per
    /// `render_whatsapp`'s option-count handling).
    pub async fn send_interactive_list(
        &self,
        to: &str,
        body_text: &str,
        button_label: &str,
        rows: &[ListRow],
    ) -> Result<(), WhatsAppApiError> {
        let row_values: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "id": r.id,
                    "title": r.title,
                    "description": r.description,
                })
            })
            .collect();

        self.post(json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "interactive",
            "interactive": {
                "type": "list",
                "body": { "text": body_text },
                "action": {
                    "button": button_label,
                    "sections": [{ "title": "Options", "rows": row_values }],
                },
            },
        }))
        .await
    }

    /// Sends a pre-approved template message — the only outbound message
    /// type Meta allows outside a user-initiated 24-hour window (PRD v2
    /// admin page 9's stated constraint). `components` follows Meta's
    /// template-component JSON shape (header/body/button parameter
    /// substitutions); pass an empty slice for a template with no
    /// variables.
    pub async fn send_template(
        &self,
        to: &str,
        template_name: &str,
        language_code: &str,
        components: Vec<Value>,
    ) -> Result<(), WhatsAppApiError> {
        self.post(json!({
            "messaging_product": "whatsapp",
            "recipient_type": "individual",
            "to": to,
            "type": "template",
            "template": {
                "name": template_name,
                "language": { "code": language_code },
                "components": components,
            },
        }))
        .await
    }
}

/// Marks an inbound message as read (a courteous UX touch — the citizen
/// sees the blue double-check — not required for correctness).
#[derive(Debug, Serialize)]
pub struct MarkReadRequest<'a> {
    pub messaging_product: &'static str,
    pub status: &'static str,
    pub message_id: &'a str,
}

impl WhatsAppClient {
    pub async fn mark_read(&self, message_id: &str) -> Result<(), WhatsAppApiError> {
        self.post(json!({
            "messaging_product": "whatsapp",
            "status": "read",
            "message_id": message_id,
        }))
        .await
    }
}
