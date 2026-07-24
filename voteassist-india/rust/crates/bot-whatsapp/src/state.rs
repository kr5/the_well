//! Shared application state for the WhatsApp webhook server: the Meta
//! Cloud API client, the decision tree, the ephemeral session store
//! (`channel_core::SessionStore`), and a small `wa_id -> session_token`
//! routing index. WhatsApp has no per-chat dialogue mechanism analogous
//! to teloxide's `InMemStorage` (which `bot-telegram` uses), so this
//! crate builds the equivalent itself — see `ConversationIndex` below.
//! The index also tracks each `wa_id`'s last-inbound timestamp, which
//! `broadcast::send_proactive_message` reads to decide whether Meta's
//! 24-hour free-form customer-service window is still open (E8.F2.T3).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use channel_core::SessionStore;
use chrono::{DateTime, Utc};
use core_domain::DecisionTree;
use tokio::sync::Mutex;

use crate::client::WhatsAppClient;

/// Meta's documented customer-service window: free-form replies are only
/// allowed within 24 hours of the user's last inbound message; outside
/// that, only pre-approved templates may be sent (PRD v2 admin page 9's
/// stated constraint, E8.F2.T3).
pub const CUSTOMER_SERVICE_WINDOW: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(Clone)]
struct ConversationEntry {
    session_token: String,
    last_inbound_at: DateTime<Utc>,
}

/// Maps a WhatsApp `wa_id` (the platform's stable, phone-derived contact
/// identifier) to that citizen's current `session_token` and the instant
/// of their last inbound message. This mapping is purely internal routing
/// — the `session_token` it points at (never the `wa_id` itself) is what
/// would ever reach `analytics_events.session_id` if/when this channel is
/// wired into analytics, preserving the same "never a stable channel
/// identifier" discipline `bot-telegram`'s dialogue-state comment
/// documents.
#[derive(Clone)]
pub struct ConversationIndex {
    entries: Arc<Mutex<HashMap<String, ConversationEntry>>>,
}

impl ConversationIndex {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Records/refreshes `wa_id`'s current session token and the instant
    /// of this inbound message (which also resets the 24-hour free-form
    /// window).
    pub async fn record_inbound(&self, wa_id: &str, session_token: String) {
        let mut entries = self.entries.lock().await;
        entries.insert(
            wa_id.to_string(),
            ConversationEntry {
                session_token,
                last_inbound_at: Utc::now(),
            },
        );
    }

    pub async fn session_token_for(&self, wa_id: &str) -> Option<String> {
        self.entries
            .lock()
            .await
            .get(wa_id)
            .map(|entry| entry.session_token.clone())
    }

    pub async fn remove(&self, wa_id: &str) {
        self.entries.lock().await.remove(wa_id);
    }

    /// True if `wa_id` sent an inbound message within Meta's 24-hour
    /// customer-service window, i.e. a free-form (non-template) proactive
    /// reply is still allowed.
    pub async fn within_free_form_window(&self, wa_id: &str) -> bool {
        let entries = self.entries.lock().await;
        match entries.get(wa_id) {
            Some(entry) => {
                let elapsed = Utc::now() - entry.last_inbound_at;
                elapsed
                    .to_std()
                    .map(|elapsed| elapsed < CUSTOMER_SERVICE_WINDOW)
                    .unwrap_or(false)
            }
            None => false,
        }
    }
}

impl Default for ConversationIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<WhatsAppClient>,
    pub tree: Arc<DecisionTree>,
    pub sessions: SessionStore,
    pub conversations: ConversationIndex,
    /// Compared against the `hub.verify_token` query parameter on the
    /// GET webhook-verification handshake; chosen by the operator and
    /// configured to match in Meta's App Dashboard webhook subscription.
    pub verify_token: Arc<str>,
    /// Used to verify `X-Hub-Signature-256` on every inbound POST — see
    /// `signature::verify_signature`.
    pub app_secret: Arc<str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_domain::{create_session, vote_assist_tree_v2};

    #[tokio::test]
    async fn a_freshly_recorded_conversation_is_within_the_free_form_window() {
        let index = ConversationIndex::new();
        index.record_inbound("919000000000", "token-a".into()).await;
        assert!(index.within_free_form_window("919000000000").await);
        assert_eq!(index.session_token_for("919000000000").await.as_deref(), Some("token-a"));
    }

    #[tokio::test]
    async fn an_unknown_wa_id_is_not_within_the_free_form_window() {
        let index = ConversationIndex::new();
        assert!(!index.within_free_form_window("never-seen").await);
        assert!(index.session_token_for("never-seen").await.is_none());
    }

    #[tokio::test]
    async fn remove_clears_the_conversation() {
        let index = ConversationIndex::new();
        index.record_inbound("919000000000", "token-a".into()).await;
        index.remove("919000000000").await;
        assert!(index.session_token_for("919000000000").await.is_none());
    }

    #[tokio::test]
    async fn app_state_is_constructible_and_cloneable() {
        let tree = vote_assist_tree_v2();
        let _engine_state = create_session(&tree).unwrap();
        let state = AppState {
            client: Arc::new(WhatsAppClient::new("phone-id".into(), "token".into(), "v21.0".into())),
            tree: Arc::new(tree),
            sessions: SessionStore::default(),
            conversations: ConversationIndex::default(),
            verify_token: "verify".into(),
            app_secret: "secret".into(),
        };
        let _clone = state.clone();
    }
}
