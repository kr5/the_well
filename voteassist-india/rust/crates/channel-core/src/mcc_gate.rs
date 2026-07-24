//! The MCC (Model Code of Conduct) broadcast gate, per R-MCC-1
//! (docs/PRD-V2-RUST-PLATFORM.md Section 2.6, item 1): "Broadcast-capable
//! send paths (bot-telegram, bot-whatsapp, jobs) SHALL check a per-state
//! MCC-active flag before firing proactive/broadcast messages." This
//! module is the single, shared implementation every adapter's broadcast
//! path calls — per that section, `mcc_windows` is "the single source of
//! truth" and must not be duplicated elsewhere.
//!
//! This gate only ever governs *proactive* (bot- or job-initiated)
//! messages. Replying to a citizen's own in-progress question-and-answer
//! walkthrough is never gated by this check — MCC restricts unsolicited
//! mobilization content, not answering what someone already asked.

use sqlx::PgPool;

/// True if any state currently has an active MCC window
/// (`mcc_windows.is_active`). Use for genuinely platform-wide broadcasts
/// (rare); prefer `is_mcc_active_for_state` for the common, state-scoped
/// case, since MCC windows are declared per state.
pub async fn is_any_mcc_active(pool: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM mcc_windows WHERE is_active)")
        .fetch_one(pool)
        .await
}

/// True if the named state (matching `states.name` exactly, e.g.
/// `"Maharashtra"`) currently has an active MCC window.
pub async fn is_mcc_active_for_state(pool: &PgPool, state_name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM mcc_windows w
            JOIN states s ON s.id = w.state_id
            WHERE s.name = $1 AND w.is_active
        )
        "#,
    )
    .bind(state_name)
    .fetch_one(pool)
    .await
}

/// Outcome of a gate check, returned instead of a bare `bool` so callers'
/// logs/metrics can distinguish "suppressed by MCC" from "sent" without
/// re-deriving the reason at every call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastDecision {
    Allowed,
    SuppressedByMcc,
}

/// The single call every broadcast-capable send path (per R-MCC-1) should
/// make immediately before firing a proactive message. `state_name` is
/// `None` for platform-wide broadcasts not scoped to one state.
pub async fn check_broadcast_allowed(
    pool: &PgPool,
    state_name: Option<&str>,
) -> Result<BroadcastDecision, sqlx::Error> {
    let mcc_active = match state_name {
        Some(name) => is_mcc_active_for_state(pool, name).await?,
        None => is_any_mcc_active(pool).await?,
    };

    Ok(if mcc_active {
        BroadcastDecision::SuppressedByMcc
    } else {
        BroadcastDecision::Allowed
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_decision_variants_are_distinct() {
        assert_ne!(BroadcastDecision::Allowed, BroadcastDecision::SuppressedByMcc);
    }
}
