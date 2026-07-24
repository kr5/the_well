//! Server functions for the optional public account system: OTP login,
//! saved decision-tree drafts ("checklists"), consent history, and
//! account deletion. NOT gated behind `#[cfg(feature = "ssr")]` at module
//! level — same reasoning as `server_fns.rs` (the `#[server]` macro
//! splits real bodies from client stubs); every function below writes
//! `use crate::accounts::...;` *inside* its body since `crate::accounts`
//! itself is `ssr`-gated.

use core_domain::EngineState;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::Json;

use crate::render::RenderableTerminal;

/// ----- OTP login -----

#[server]
pub async fn request_otp(contact: String, channel: String) -> Result<(), ServerFnError> {
    use crate::accounts::email::send_otp_email;
    use crate::accounts::hashing::{hash_contact_identifier, normalize_contact};
    use crate::accounts::otp::{generate_otp, hash_otp, OTP_LIFETIME_MINUTES};

    if channel != "email" {
        // Disclosed gap, not a silent no-op — see accounts::email's module
        // doc for why phone/SMS OTP has no vendor integration in this pass.
        return Err(ServerFnError::ServerError(
            "phone-based verification isn't available yet on this deployment — please use email".to_string(),
        ));
    }

    let pool = expect_context::<sqlx::PgPool>();
    let normalized = normalize_contact(&contact, &channel);
    let contact_hash =
        hash_contact_identifier(&normalized).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let otp = generate_otp();
    let otp_hash = hash_otp(&otp).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(OTP_LIFETIME_MINUTES);

    sqlx::query(
        r#"
        INSERT INTO account_otp_challenges (contact_identifier_hash, contact_channel, otp_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&contact_hash)
    .bind(&channel)
    .bind(&otp_hash)
    .bind(expires_at)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    send_otp_email(&normalized, &otp).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn verify_otp_and_login(contact: String, channel: String, otp: String) -> Result<(), ServerFnError> {
    use crate::accounts::hashing::{hash_contact_identifier, normalize_contact};
    use crate::accounts::otp::{verify_otp, MAX_OTP_ATTEMPTS};
    use crate::accounts::session::create_account_session;

    let pool = expect_context::<sqlx::PgPool>();
    let normalized = normalize_contact(&contact, &channel);
    let contact_hash =
        hash_contact_identifier(&normalized).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let challenge: Option<(String, String, i32)> = sqlx::query_as(
        r#"
        SELECT id::text, otp_hash, attempt_count
        FROM account_otp_challenges
        WHERE contact_identifier_hash = $1 AND contact_channel = $2
          AND expires_at > now() AND consumed_at IS NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&contact_hash)
    .bind(&channel)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let Some((challenge_id, otp_hash, attempt_count)) = challenge else {
        return Err(ServerFnError::ServerError("that code has expired — request a new one".to_string()));
    };

    if attempt_count >= MAX_OTP_ATTEMPTS {
        return Err(ServerFnError::ServerError("too many attempts — request a new code".to_string()));
    }

    if !verify_otp(&otp_hash, &otp) {
        sqlx::query("UPDATE account_otp_challenges SET attempt_count = attempt_count + 1 WHERE id = $1::uuid")
            .bind(&challenge_id)
            .execute(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
        return Err(ServerFnError::ServerError("incorrect code".to_string()));
    }

    sqlx::query("UPDATE account_otp_challenges SET consumed_at = now() WHERE id = $1::uuid")
        .bind(&challenge_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let account_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO user_accounts (contact_identifier_hash, contact_channel)
        VALUES ($1, $2)
        ON CONFLICT (contact_identifier_hash) DO UPDATE SET last_login_at = now()
        RETURNING id::text
        "#,
    )
    .bind(&contact_hash)
    .bind(&channel)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let token =
        create_account_session(&pool, &account_id).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    set_account_cookie(&token);

    Ok(())
}

#[server]
pub async fn logout_account() -> Result<(), ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;

    use crate::accounts::session::destroy_account_session;
    use crate::accounts::ACCOUNT_SESSION_COOKIE_NAME;

    let pool = expect_context::<sqlx::PgPool>();

    if let Ok(jar) = leptos_axum::extract::<CookieJar>().await {
        if let Some(cookie) = jar.get(ACCOUNT_SESSION_COOKIE_NAME) {
            let _ = destroy_account_session(&pool, cookie.value()).await;
        }
    }

    clear_account_cookie();
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountView {
    pub id: String,
    pub contact_channel: String,
    pub notifications_opt_in: bool,
}

#[server]
pub async fn current_account() -> Result<Option<AccountView>, ServerFnError> {
    use crate::accounts::guard::current_account_id;

    let pool = expect_context::<sqlx::PgPool>();
    let Some(account_id) = current_account_id(&pool).await else {
        return Ok(None);
    };

    let row: Option<(String, bool)> =
        sqlx::query_as("SELECT contact_channel, notifications_opt_in FROM user_accounts WHERE id = $1::uuid")
            .bind(&account_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(row.map(|(contact_channel, notifications_opt_in)| AccountView {
        id: account_id,
        contact_channel,
        notifications_opt_in,
    }))
}

/// Deletes the account outright — cascades to `saved_drafts` and
/// `consent_artifacts` via `ON DELETE CASCADE`
/// (`migrations/0007_accounts_and_drafts.sql`). "No soft-delete, no grace
/// period in the live system," per that migration's own comment.
#[server]
pub async fn delete_account() -> Result<(), ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    sqlx::query("DELETE FROM user_accounts WHERE id = $1::uuid")
        .bind(&account_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    logout_account().await
}

#[cfg(feature = "ssr")]
fn set_account_cookie(token: &str) {
    use http::header::{HeaderValue, SET_COOKIE};

    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        let cookie_value = format!(
            "{}={token}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=2592000",
            crate::accounts::ACCOUNT_SESSION_COOKIE_NAME
        );
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            opts.insert_header(SET_COOKIE, header_value);
        }
    }
}

#[cfg(feature = "ssr")]
fn clear_account_cookie() {
    use http::header::{HeaderValue, SET_COOKIE};

    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        let cookie_value = format!(
            "{}=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0",
            crate::accounts::ACCOUNT_SESSION_COOKIE_NAME
        );
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            opts.insert_header(SET_COOKIE, header_value);
        }
    }
}

/// ----- Saved checklists (drafts) -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftSummary {
    pub id: String,
    pub label: Option<String>,
    pub is_frozen: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftDetail {
    pub id: String,
    pub label: Option<String>,
    pub state: EngineState,
    pub frozen_terminal_snapshot: Option<RenderableTerminal>,
}

/// Saves the current walkthrough as a named checklist. If `terminal` is
/// `Some` (the walkthrough reached a terminal outcome), it's frozen
/// exactly as shown — per `saved_drafts.frozen_terminal_snapshot`'s
/// comment, "so a later KB edit doesn't silently rewrite a saved
/// checklist out from under a user."
#[server(input = Json)]
pub async fn save_draft(
    label: Option<String>,
    state: EngineState,
    terminal: Option<RenderableTerminal>,
) -> Result<String, ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    let answer_history = serde_json::to_value(&state.history).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let frozen_snapshot =
        terminal.as_ref().map(serde_json::to_value).transpose().map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let frozen_at = terminal.is_some().then(chrono::Utc::now);

    let draft_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO saved_drafts (account_id, label, decision_tree_version, answer_history, frozen_terminal_snapshot, frozen_at)
        VALUES ($1::uuid, $2, $3, $4, $5, $6)
        RETURNING id::text
        "#,
    )
    .bind(&account_id)
    .bind(&label)
    .bind(core_domain::vote_assist_tree_v2().version as i32)
    .bind(&answer_history)
    .bind(&frozen_snapshot)
    .bind(frozen_at)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(draft_id)
}

#[server]
pub async fn list_my_drafts() -> Result<Vec<DraftSummary>, ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    let rows: Vec<(String, Option<String>, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id::text, label, (frozen_at IS NOT NULL), updated_at
        FROM saved_drafts
        WHERE account_id = $1::uuid
        ORDER BY updated_at DESC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|(id, label, is_frozen, updated_at)| DraftSummary { id, label, is_frozen, updated_at })
        .collect())
}

#[server]
pub async fn get_draft(id: String) -> Result<Option<DraftDetail>, ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    let row: Option<(String, Option<String>, serde_json::Value, Option<serde_json::Value>)> = sqlx::query_as(
        r#"
        SELECT id::text, label, answer_history, frozen_terminal_snapshot
        FROM saved_drafts
        WHERE id = $1::uuid AND account_id = $2::uuid
        "#,
    )
    .bind(&id)
    .bind(&account_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let Some((id, label, answer_history, frozen_terminal_snapshot)) = row else {
        return Ok(None);
    };

    let history = serde_json::from_value(answer_history).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let frozen_terminal_snapshot = frozen_terminal_snapshot
        .map(serde_json::from_value)
        .transpose()
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let tree = core_domain::vote_assist_tree_v2();
    let state = EngineState {
        tree_id: tree.id.clone(),
        current_node_id: reconstruct_current_node_id(&tree, &history),
        history,
    };

    Ok(Some(DraftDetail { id, label, state, frozen_terminal_snapshot }))
}

/// Replays a saved `answer_history` from the tree's start node to find
/// where it currently leaves off — never trusts a stored `current_node_id`
/// directly, since the tree may have changed shape since the draft was
/// saved (matching `frozen_terminal_snapshot`'s own reasoning: content can
/// move on, a saved artifact shouldn't silently break instead of just
/// stopping wherever replay gets to).
fn reconstruct_current_node_id(tree: &core_domain::DecisionTree, history: &[core_domain::Answer]) -> String {
    let mut state = match core_domain::create_session(tree) {
        Ok(state) => state,
        Err(_) => return tree.start_node_id.clone(),
    };
    for answer in history {
        match core_domain::answer(tree, &state, &answer.value) {
            Ok(next) => state = next,
            Err(_) => break,
        }
    }
    state.current_node_id
}

#[server]
pub async fn delete_draft(id: String) -> Result<(), ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    sqlx::query("DELETE FROM saved_drafts WHERE id = $1::uuid AND account_id = $2::uuid")
        .bind(&id)
        .bind(&account_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- Consent history & notification preferences -----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentEntry {
    pub purpose: String,
    pub granted: bool,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Appends a consent grant/revocation row — never deletes/overwrites a
/// prior row, per `consent_artifacts`' own "append-only... so the account
/// settings page can render an honest history" comment.
#[server]
pub async fn record_consent(purpose: String, granted: bool) -> Result<(), ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    sqlx::query("INSERT INTO consent_artifacts (account_id, purpose, granted) VALUES ($1::uuid, $2, $3)")
        .bind(&account_id)
        .bind(&purpose)
        .bind(granted)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn list_consent_history() -> Result<Vec<ConsentEntry>, ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    sqlx::query_as(
        r#"
        SELECT purpose, granted, occurred_at
        FROM consent_artifacts
        WHERE account_id = $1::uuid
        ORDER BY occurred_at DESC
        "#,
    )
    .bind(&account_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server]
pub async fn set_notification_prefs(opt_in: bool, state_name: Option<String>) -> Result<(), ServerFnError> {
    use crate::accounts::guard::require_account;

    let pool = expect_context::<sqlx::PgPool>();
    let account_id = require_account(&pool).await?;

    let state_id: Option<String> = match &state_name {
        Some(name) => sqlx::query_scalar("SELECT id::text FROM states WHERE name = $1")
            .bind(name)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?,
        None => None,
    };

    sqlx::query(
        r#"
        UPDATE user_accounts
        SET notifications_opt_in = $2, notification_state_interest = $3::uuid
        WHERE id = $1::uuid
        "#,
    )
    .bind(&account_id)
    .bind(opt_in)
    .bind(&state_id)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}
