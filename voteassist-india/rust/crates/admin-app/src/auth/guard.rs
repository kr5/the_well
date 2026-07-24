//! The per-server-function authorization gate. Every admin server
//! function that reads or writes real data calls `require_role` (or
//! `require_admin` for "any authenticated admin, role doesn't matter")
//! as its first line — see `mod.rs`'s doc comment for why this, not a
//! page-level check, is the actual security boundary.

use axum_extra::extract::cookie::CookieJar;
use leptos::prelude::ServerFnError;
use sqlx::PgPool;

use crate::auth::{role_satisfies, validate_session, AdminUser, SESSION_COOKIE_NAME};

async fn current_admin_user(pool: &PgPool) -> Option<AdminUser> {
    let jar: CookieJar = leptos_axum::extract().await.ok()?;
    let token = jar.get(SESSION_COOKIE_NAME)?.value().to_string();
    validate_session(pool, &token).await.ok().flatten()
}

pub async fn require_admin(pool: &PgPool) -> Result<AdminUser, ServerFnError> {
    current_admin_user(pool)
        .await
        .ok_or_else(|| ServerFnError::ServerError("not authenticated — please log in".to_string()))
}

pub async fn require_role(pool: &PgPool, allowed: &[&str]) -> Result<AdminUser, ServerFnError> {
    let admin = require_admin(pool).await?;
    if role_satisfies(&admin.role, allowed) {
        Ok(admin)
    } else {
        Err(ServerFnError::ServerError(
            "your role does not have permission to do this".to_string(),
        ))
    }
}
