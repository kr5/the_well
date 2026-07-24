//! Ephemeral, in-process session storage for bot/IVR channel adapters, per
//! docs/PRD-V2-RUST-PLATFORM.md Section 8.7's policy for public
//! decision-engine sessions: "Redis-ephemeral (bot/IVR channels where
//! client-side state doesn't fit naturally): a short-TTL (30-60 min
//! inactivity) store keyed by a rotating token holds in-progress state.
//! Never written to Postgres."
//!
//! This implementation is in-process (a `tokio::sync::Mutex<HashMap<...>>`)
//! rather than Redis-backed, consistent with this project's stated "no
//! extra Redis dependency" infrastructure preference (Section 6.8 makes
//! the same call for `crates/jobs`'s scheduler). Known scaling limitation,
//! disclosed rather than hidden: this store is scoped to a single OS
//! process. If a bot adapter is ever horizontally scaled across multiple
//! instances behind a load balancer, an in-memory store would fragment
//! session state across instances — at that point a shared store (Redis,
//! or a short-TTL Postgres table with the same never-resumable discipline)
//! becomes necessary. For the MVP-Rust-v1 single-instance-per-service
//! hosting posture (Section 5), a single process is the expected
//! deployment shape, so this is the right-sized implementation today.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use core_domain::EngineState;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Per docs/PRD-V2-RUST-PLATFORM.md Section 8.7: "short-TTL (30-60 min
/// inactivity)". 45 minutes splits that range.
pub const DEFAULT_SESSION_TTL: Duration = Duration::from_secs(45 * 60);

struct StoredSession {
    state: EngineState,
    expires_at: DateTime<Utc>,
}

/// Thread-safe, TTL-based store mapping an opaque rotating token to a
/// citizen's in-progress `EngineState`. Cloning is cheap (an `Arc` behind
/// the scenes), so every connection handler / update callback can hold its
/// own clone rather than sharing a reference across async task boundaries.
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<Mutex<HashMap<String, StoredSession>>>,
    ttl: Duration,
}

impl SessionStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }

    /// Starts a new session under a freshly generated random token. Per
    /// docs/PRD-V2-RUST-PLATFORM.md Section 8.7, this token must never be
    /// derived from a stable channel identifier (a Telegram `chat_id`, a
    /// WhatsApp phone number) — doing so would silently reintroduce a
    /// stable cross-session identifier via the back door if this token
    /// ever flows into `analytics_events.session_id`.
    pub async fn create(&self, state: EngineState) -> String {
        let token = Uuid::new_v4().to_string();
        self.put(token.clone(), state).await;
        token
    }

    pub async fn put(&self, token: String, state: EngineState) {
        let mut sessions = self.sessions.lock().await;
        sessions.insert(
            token,
            StoredSession {
                state,
                expires_at: Utc::now() + to_chrono_duration(self.ttl),
            },
        );
    }

    /// Reads back a session, refreshing its TTL on read — the 30-60 minute
    /// window is inactivity-based, not an absolute expiry from creation.
    pub async fn get(&self, token: &str) -> Option<EngineState> {
        let mut sessions = self.sessions.lock().await;
        let now = Utc::now();
        match sessions.get_mut(token) {
            Some(session) if session.expires_at > now => {
                session.expires_at = now + to_chrono_duration(self.ttl);
                Some(session.state.clone())
            }
            Some(_expired) => {
                sessions.remove(token);
                None
            }
            None => None,
        }
    }

    pub async fn remove(&self, token: &str) {
        self.sessions.lock().await.remove(token);
    }

    /// Sweeps every expired entry. Intended to be called on a periodic
    /// timer by the adapter's own binary — `get`/`put` are self-cleaning
    /// for the keys they touch, but a session a citizen abandons
    /// mid-conversation and never revisits would otherwise sit in memory
    /// until the process restarts.
    pub async fn sweep_expired(&self) -> usize {
        let mut sessions = self.sessions.lock().await;
        let now = Utc::now();
        let before = sessions.len();
        sessions.retain(|_, session| session.expires_at > now);
        before - sessions.len()
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(DEFAULT_SESSION_TTL)
    }
}

fn to_chrono_duration(std_duration: Duration) -> chrono::Duration {
    chrono::Duration::from_std(std_duration)
        .unwrap_or_else(|_| chrono::Duration::seconds(std_duration.as_secs() as i64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_domain::create_session;
    use core_domain::vote_assist_tree_v2;

    fn sample_state() -> EngineState {
        let tree = vote_assist_tree_v2();
        create_session(&tree).unwrap()
    }

    #[tokio::test]
    async fn create_then_get_round_trips_the_state() {
        let store = SessionStore::new(Duration::from_secs(60));
        let state = sample_state();
        let token = store.create(state.clone()).await;

        let fetched = store.get(&token).await.expect("session should still be present");
        assert_eq!(fetched.current_node_id, state.current_node_id);
    }

    #[tokio::test]
    async fn unknown_token_returns_none() {
        let store = SessionStore::new(Duration::from_secs(60));
        assert!(store.get("not-a-real-token").await.is_none());
    }

    #[tokio::test]
    async fn expired_session_is_evicted_on_read() {
        let store = SessionStore::new(Duration::from_millis(20));
        let token = store.create(sample_state()).await;

        tokio::time::sleep(Duration::from_millis(80)).await;

        assert!(store.get(&token).await.is_none());
    }

    #[tokio::test]
    async fn sweep_expired_removes_only_stale_entries() {
        let store = SessionStore::new(Duration::from_millis(20));
        let stale_token = store.create(sample_state()).await;

        tokio::time::sleep(Duration::from_millis(80)).await;

        let fresh_token = store.create(sample_state()).await;

        let removed = store.sweep_expired().await;
        assert_eq!(removed, 1);
        assert!(store.get(&stale_token).await.is_none());
        assert!(store.get(&fresh_token).await.is_some());
    }

    #[tokio::test]
    async fn remove_deletes_a_session_outright() {
        let store = SessionStore::new(Duration::from_secs(60));
        let token = store.create(sample_state()).await;
        store.remove(&token).await;
        assert!(store.get(&token).await.is_none());
    }
}
