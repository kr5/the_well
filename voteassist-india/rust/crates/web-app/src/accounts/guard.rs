//! Reads the account-session cookie and resolves it to an `account_id`.
//! Mirrors `admin-app::auth::guard`'s pattern, against the separate
//! `account_sessions`/`user_accounts` tables.

use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::ServerFnError;
use sqlx::PgPool;

use crate::accounts::session::validate_account_session;
use crate::accounts::ACCOUNT_SESSION_COOKIE_NAME;

pub async fn current_account_id(pool: &PgPool) -> Option<String> {
    let jar: CookieJar = leptos_axum::extract().await.ok()?;
    let token = jar.get(ACCOUNT_SESSION_COOKIE_NAME)?.value().to_string();
    validate_account_session(pool, &token).await.ok().flatten()
}

pub async fn require_account(pool: &PgPool) -> Result<String, ServerFnError> {
    current_account_id(pool)
        .await
        .ok_or_else(|| ServerFnError::ServerError("please log in to use this feature".to_string()))
}
