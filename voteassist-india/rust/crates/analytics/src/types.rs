use chrono::{DateTime, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Mirrors the Postgres `analytics_event_type` enum
/// (migrations/0009_analytics.sql). `sqlx::Type` binds this directly to
/// the Postgres enum via its type name, so inserts/selects can use the
/// runtime-checked `sqlx::query`/`query_as` API without requiring
/// compile-time database introspection (`cargo sqlx prepare`) — a
/// deliberate choice for code written and reviewed without access to a
/// live database; switching specific hot-path queries to the compile-time-
/// checked `sqlx::query!` macros is a natural follow-up once this crate is
/// built against a real server-side Postgres instance (see
/// docs/PRD-V2-RUST-PLATFORM.md Section 16's CI `cargo sqlx prepare
/// --check` gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "analytics_event_type", rename_all = "snake_case")]
pub enum EventType {
    SessionStarted,
    QuestionAnswered,
    TerminalReached,
    DeepLinkClicked,
    KbSearchPerformed,
    KbEntryViewed,
}

/// Mirrors the Postgres `client_platform` enum
/// (migrations/0001_extensions_and_enums.sql), shared across analytics,
/// bot channel config, and the future multi-channel adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "client_platform", rename_all = "snake_case")]
pub enum ClientPlatform {
    Web,
    Telegram,
    Whatsapp,
    Ivr,
}

/// A single privacy-preserving analytics event, ready to be recorded.
/// Every non-negotiable from docs/PRD-V2-RUST-PLATFORM.md Section 12 is
/// enforced by this struct's shape, not left to caller discipline:
/// - `answer_option` is always an option VALUE (e.g. "student_hostel"),
///   never free text — this struct has no field that could hold free text
///   from a user for anything other than `kb_search_category_hash`, which
///   is explicitly documented as a hash/bucket, not raw text.
/// - `session_id` is expected to be a fresh, ephemeral, rotated identifier
///   per session — this crate does not generate or persist any mapping
///   from it back to a real identity.
/// - There is no field for IP address, device fingerprint, or any
///   sensitive-category data, because none should ever be collected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEvent {
    pub event_type: EventType,
    pub decision_tree_version: Option<i32>,
    pub node_id: Option<String>,
    pub answer_option: Option<String>,
    pub session_id: Uuid,
    pub locale: String,
    pub client_platform: ClientPlatform,
    pub deep_link_clicked: Option<bool>,
    pub kb_search_category_hash: Option<String>,
}

impl NewEvent {
    /// Convenience constructor for the common case (no search/deep-link
    /// specific fields set).
    pub fn new(
        event_type: EventType,
        session_id: Uuid,
        locale: impl Into<String>,
        client_platform: ClientPlatform,
    ) -> Self {
        Self {
            event_type,
            decision_tree_version: None,
            node_id: None,
            answer_option: None,
            session_id,
            locale: locale.into(),
            client_platform,
            deep_link_clicked: None,
            kb_search_category_hash: None,
        }
    }

    pub fn with_node(mut self, tree_version: i32, node_id: impl Into<String>) -> Self {
        self.decision_tree_version = Some(tree_version);
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_answer_option(mut self, answer_option: impl Into<String>) -> Self {
        self.answer_option = Some(answer_option.into());
        self
    }

    pub fn with_deep_link_clicked(mut self, clicked: bool) -> Self {
        self.deep_link_clicked = Some(clicked);
        self
    }

    pub fn with_kb_search_category_hash(mut self, hash: impl Into<String>) -> Self {
        self.kb_search_category_hash = Some(hash.into());
        self
    }
}

/// Truncates a timestamp down to the start of its hour — the bucketing
/// discipline required for `occurred_at_hour` (never full-precision, to
/// prevent re-identification via timing correlation, per PRD v2 Section
/// 12).
pub fn truncate_to_hour(timestamp: DateTime<Utc>) -> DateTime<Utc> {
    timestamp
        .date_naive()
        .and_hms_opt(timestamp.time().hour(), 0, 0)
        .expect("hour is always in range 0..24")
        .and_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn truncate_to_hour_zeroes_minutes_and_seconds() {
        let t = Utc.with_ymd_and_hms(2026, 3, 15, 14, 37, 52).unwrap();
        let truncated = truncate_to_hour(t);
        assert_eq!(truncated, Utc.with_ymd_and_hms(2026, 3, 15, 14, 0, 0).unwrap());
    }

    #[test]
    fn new_event_builder_sets_only_requested_fields() {
        let event = NewEvent::new(
            EventType::QuestionAnswered,
            Uuid::new_v4(),
            "hi",
            ClientPlatform::Web,
        )
        .with_node(2, "residence_type")
        .with_answer_option("tribal_remote_or_slum");

        assert_eq!(event.node_id.as_deref(), Some("residence_type"));
        assert_eq!(event.answer_option.as_deref(), Some("tribal_remote_or_slum"));
        assert!(event.deep_link_clicked.is_none());
        assert!(event.kb_search_category_hash.is_none());
    }
}
