//! Server-side admin session storage, backed by the `sessions` table
//! (`migrations/0006_admin_and_audit.sql`). Deliberately hand-rolled
//! rather than built on the `tower-sessions` crate the PRD names
//! (docs/PRD-V2-RUST-PLATFORM.md Section 6.10): `tower-sessions`'
//! Postgres store integration (`tower-sessions-sqlx-store`) manages its
//! own table schema via its own `.migrate()` call, and this project's
//! `sessions` table was already fixed by migration 0006 with a specific
//! shape (`id`, `admin_user_id`, `data`, `expiry_date`) — reconciling the
//! two without being able to compile and check the store crate's exact
//! expectations against that fixed schema would be guessing. A small,
//! fully self-contained session mechanism against our own already-defined
//! table is the safer, fully-auditable choice for this pass; migrating to
//! `tower-sessions` proper is a reasonable follow-up once this can be
//! verified against a real build.
//!
//! Token generation uses `rand`'s OS-backed CSPRNG (32 random bytes, hex
//! -encoded) rather than a UUID, specifically because a session token's
//! unguessability is a real security property this code depends on, not
//! just convenient uniqueness.

use rand::RngCore;
use sqlx::PgPool;

/// 8 hours — short enough to bound the blast radius of a leaked cookie,
/// long enough not to force re-login mid-shift. Step-up (re-authentication
/// for specific high-impact actions) is layered separately, per PRD v2
/// Section 16's admin-session hardening note — not implemented in this
/// pass; see this crate's README for the disclosed scope boundary.
const SESSION_LIFETIME_HOURS: i64 = 8;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub role: String,
}

pub async fn create_session(pool: &PgPool, admin_user_id: &str) -> Result<String, sqlx::Error> {
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex_encode(&token_bytes);

    let expiry = chrono::Utc::now() + chrono::Duration::hours(SESSION_LIFETIME_HOURS);

    sqlx::query(
        r#"
        INSERT INTO sessions (id, admin_user_id, data, expiry_date)
        VALUES ($1, $2::uuid, ''::bytea, $3)
        "#,
    )
    .bind(&token)
    .bind(admin_user_id)
    .bind(expiry)
    .execute(pool)
    .await?;

    Ok(token)
}

pub async fn validate_session(pool: &PgPool, token: &str) -> Result<Option<AdminUser>, sqlx::Error> {
    let row: Option<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT admin_users.id::text, admin_users.email, admin_users.role::text
        FROM sessions
        JOIN admin_users ON admin_users.id = sessions.admin_user_id
        WHERE sessions.id = $1
          AND sessions.expiry_date > now()
          AND admin_users.is_active
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id, email, role)| AdminUser { id, email, role }))
}

pub async fn destroy_session(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = $1").bind(token).execute(pool).await?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_encode_produces_a_lowercase_fixed_width_string() {
        let encoded = hex_encode(&[0x00, 0xff, 0x0a]);
        assert_eq!(encoded, "00ff0a");
        assert_eq!(hex_encode(&[0u8; 32]).len(), 64);
    }
}
