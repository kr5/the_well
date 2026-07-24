//! Session storage for logged-in public accounts (`account_sessions`,
//! `migrations/0012_account_otp_and_sessions.sql`) — deliberately separate
//! from `admin-app`'s admin session mechanism (different table, different
//! trust domain). Same CSPRNG-token design as `admin-app::auth::session`
//! (see that module's doc comment for why not `tower-sessions`).

use rand::RngCore;
use sqlx::PgPool;

/// Longer-lived than the admin session (30 days vs. 8 hours): this is a
/// low-stakes convenience account (no PII beyond a hashed contact and
/// saved decision-tree answers), not an admin credential, so a longer
/// "stay logged in" window is the right tradeoff.
const SESSION_LIFETIME_DAYS: i64 = 30;

pub async fn create_account_session(pool: &PgPool, account_id: &str) -> Result<String, sqlx::Error> {
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex_encode(&token_bytes);

    let expiry = chrono::Utc::now() + chrono::Duration::days(SESSION_LIFETIME_DAYS);

    sqlx::query("INSERT INTO account_sessions (id, account_id, expiry_date) VALUES ($1, $2::uuid, $3)")
        .bind(&token)
        .bind(account_id)
        .bind(expiry)
        .execute(pool)
        .await?;

    Ok(token)
}

pub async fn validate_account_session(pool: &PgPool, token: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT account_id::text
        FROM account_sessions
        WHERE id = $1 AND expiry_date > now()
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn destroy_account_session(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM account_sessions WHERE id = $1").bind(token).execute(pool).await?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
