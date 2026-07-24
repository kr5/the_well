//! Admin server functions. NOT gated behind `#[cfg(feature = "ssr")]` at
//! the module level, same reasoning as `web-app`'s `server_fns.rs`: the
//! `#[server]` macro itself splits real bodies (ssr) from fetch stubs
//! (hydrate). Because `crate::auth` IS gated to `ssr` (see that module's
//! doc comment), every function below writes `use crate::auth::...;`
//! *inside* its body rather than at file scope, so the import itself is
//! also compiled out on the client build along with the rest of the body.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::Json;

/// ----- Auth -----

#[server]
pub async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth::{create_session, verify_password};

    let pool = expect_context::<sqlx::PgPool>();

    let row: Option<(String, String)> =
        sqlx::query_as("SELECT id::text, password_hash FROM admin_users WHERE email = $1 AND is_active")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let Some((admin_id, password_hash)) = row else {
        // Deliberately the same error for "no such user" and "wrong
        // password" below — distinguishing them lets an attacker enumerate
        // valid admin emails.
        return Err(ServerFnError::ServerError("invalid email or password".to_string()));
    };

    let verified =
        verify_password(&password_hash, &password).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    if !verified {
        return Err(ServerFnError::ServerError("invalid email or password".to_string()));
    }

    let token = create_session(&pool, &admin_id).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE admin_users SET last_login_at = now() WHERE id = $1::uuid")
        .bind(&admin_id)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    set_session_cookie(&token);

    Ok(())
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;

    use crate::auth::{destroy_session, SESSION_COOKIE_NAME};

    let pool = expect_context::<sqlx::PgPool>();

    if let Ok(jar) = leptos_axum::extract::<CookieJar>().await {
        if let Some(cookie) = jar.get(SESSION_COOKIE_NAME) {
            let _ = destroy_session(&pool, cookie.value()).await;
        }
    }

    clear_session_cookie();
    Ok(())
}

#[server]
pub async fn current_user() -> Result<Option<crate::auth::AdminUser>, ServerFnError> {
    use axum_extra::extract::cookie::CookieJar;

    use crate::auth::{validate_session, SESSION_COOKIE_NAME};

    let pool = expect_context::<sqlx::PgPool>();

    let Ok(jar) = leptos_axum::extract::<CookieJar>().await else {
        return Ok(None);
    };
    let Some(cookie) = jar.get(SESSION_COOKIE_NAME) else {
        return Ok(None);
    };

    validate_session(&pool, cookie.value()).await.map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[cfg(feature = "ssr")]
fn set_session_cookie(token: &str) {
    use http::header::{HeaderValue, SET_COOKIE};

    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        // HttpOnly (never readable from JS — this cookie is a bearer
        // credential), Secure (only sent over HTTPS — a production
        // deployment terminates TLS in front of this app, per
        // docs/SECURITY-AND-SRE-OPERATIONS.md), SameSite=Strict (this is
        // a same-site-only admin surface, unlike the public product which
        // has no login at all — matches PRD v2 Section 16's admin-session
        // hardening note).
        let cookie_value = format!(
            "{}={token}; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=28800",
            crate::auth::SESSION_COOKIE_NAME
        );
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            opts.insert_header(SET_COOKIE, header_value);
        }
    }
}

#[cfg(feature = "ssr")]
fn clear_session_cookie() {
    use http::header::{HeaderValue, SET_COOKIE};

    if let Some(opts) = use_context::<leptos_axum::ResponseOptions>() {
        let cookie_value =
            format!("{}=; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=0", crate::auth::SESSION_COOKIE_NAME);
        if let Ok(header_value) = HeaderValue::from_str(&cookie_value) {
            opts.insert_header(SET_COOKIE, header_value);
        }
    }
}

/// ----- Dashboard -----

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardStats {
    pub new_feedback_count: i64,
    pub needs_reverification_count: i64,
    pub active_mcc_window_count: i64,
    pub verified_entry_count: i64,
}

#[server]
pub async fn dashboard_stats() -> Result<DashboardStats, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    let new_feedback_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM feedback WHERE status = 'new'")
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let needs_reverification_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_entries WHERE review_status = 'needs_reverification'")
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let active_mcc_window_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcc_windows WHERE is_active")
        .fetch_one(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let verified_entry_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM knowledge_entries WHERE review_status = 'verified'")
            .fetch_one(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(DashboardStats {
        new_feedback_count,
        needs_reverification_count,
        active_mcc_window_count,
        verified_entry_count,
    })
}

/// ----- Knowledge base editor -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminKbEntrySummary {
    pub id: String,
    pub title: String,
    pub topic: String,
    pub review_status: String,
    pub last_verified_date: chrono::NaiveDate,
    pub version: i32,
}

#[server]
pub async fn list_kb_entries_admin() -> Result<Vec<AdminKbEntrySummary>, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    sqlx::query_as(
        r#"
        SELECT id, title, topic::text, review_status::text, last_verified_date, version
        FROM knowledge_entries
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminKbEntryDetail {
    pub id: String,
    pub topic: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub source_type: String,
    pub last_verified_date: chrono::NaiveDate,
    pub review_status: String,
    pub caution: Option<String>,
}

#[server]
pub async fn get_kb_entry_admin(id: String) -> Result<Option<AdminKbEntryDetail>, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    let row: Option<(String, String, String, String, String, String, chrono::NaiveDate, String, Option<String>)> =
        sqlx::query_as(
            r#"
            SELECT id, topic::text, title, summary, body, source_type::text,
                   last_verified_date, review_status::text, caution
            FROM knowledge_entries
            WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(row.map(|(id, topic, title, summary, body, source_type, last_verified_date, review_status, caution)| {
        AdminKbEntryDetail { id, topic, title, summary, body, source_type, last_verified_date, review_status, caution }
    }))
}

/// Creates or updates a knowledge-base entry: upserts `knowledge_entries`,
/// writes an append-only `knowledge_entry_revisions` snapshot, and writes
/// one `audit_log` row — all in a single transaction, per
/// docs/06-legal-compliance-review.md Section 7's review-trail
/// requirement and `knowledge_entry_revisions`'s own "this is the backbone
/// the admin Content Editor's diff view and rollback feature read from"
/// comment. Requires `contributor` or higher (any of the content roles).
#[server(input = Json)]
pub async fn save_kb_entry(entry: AdminKbEntryDetail, change_summary: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER, ROLE_TRANSLATOR};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER, ROLE_TRANSLATOR]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let next_version: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO knowledge_entries (id, topic, title, summary, body, source_type, last_verified_date, review_status, caution, created_by, version)
        VALUES ($1, $2::kb_topic, $3, $4, $5, $6::kb_source_type, $7, $8::kb_review_status, $9, $10::uuid, 1)
        ON CONFLICT (id) DO UPDATE SET
            topic = EXCLUDED.topic,
            title = EXCLUDED.title,
            summary = EXCLUDED.summary,
            body = EXCLUDED.body,
            source_type = EXCLUDED.source_type,
            last_verified_date = EXCLUDED.last_verified_date,
            review_status = EXCLUDED.review_status,
            caution = EXCLUDED.caution,
            version = knowledge_entries.version + 1
        RETURNING version
        "#,
    )
    .bind(&entry.id)
    .bind(&entry.topic)
    .bind(&entry.title)
    .bind(&entry.summary)
    .bind(&entry.body)
    .bind(&entry.source_type)
    .bind(entry.last_verified_date)
    .bind(&entry.review_status)
    .bind(&entry.caution)
    .bind(&admin.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let snapshot = serde_json::to_value(&entry).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO knowledge_entry_revisions (entry_id, version, snapshot, review_status_at_time, changed_by, change_summary)
        VALUES ($1, $2, $3, $4::kb_review_status, $5::uuid, $6)
        "#,
    )
    .bind(&entry.id)
    .bind(next_version)
    .bind(&snapshot)
    .bind(&entry.review_status)
    .bind(&admin.id)
    .bind(&change_summary)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'kb_entry.saved', 'knowledge_entries', $2, '{}'::jsonb, $3)
        "#,
    )
    .bind(&admin.id)
    .bind(&entry.id)
    .bind(&snapshot)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- MCC control panel -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MccWindowView {
    pub id: String,
    pub state_name: String,
    pub window_start: chrono::NaiveDate,
    pub window_end: Option<chrono::NaiveDate>,
    pub is_active: bool,
}

#[server]
pub async fn list_mcc_windows() -> Result<Vec<MccWindowView>, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    sqlx::query_as(
        r#"
        SELECT mcc_windows.id::text, states.name AS state_name, window_start, window_end, is_active
        FROM mcc_windows
        JOIN states ON states.id = mcc_windows.state_id
        ORDER BY window_start DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Declares a new active MCC window for `state_name`. Gated to
/// `legal_reviewer`/`superadmin` — per docs/06-legal-compliance-review.md
/// Section 4, MCC exposure is "a genuinely live legal risk area," so this
/// is deliberately not a `contributor`-level action.
#[server]
pub async fn open_mcc_window(state_name: String, window_start: chrono::NaiveDate) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let state_id: String = sqlx::query_scalar("SELECT id::text FROM states WHERE name = $1")
        .bind(&state_name)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .ok_or_else(|| ServerFnError::ServerError(format!("unknown state \"{state_name}\"")))?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let window_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO mcc_windows (state_id, window_start, entered_by)
        VALUES ($1::uuid, $2, $3::uuid)
        RETURNING id::text
        "#,
    )
    .bind(&state_id)
    .bind(window_start)
    .bind(&admin.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'mcc_window.opened', 'mcc_windows', $2, '{}'::jsonb, jsonb_build_object('state', $3::text, 'window_start', $4::date))
        "#,
    )
    .bind(&admin.id)
    .bind(&window_id)
    .bind(&state_name)
    .bind(window_start)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn close_mcc_window(window_id: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        UPDATE mcc_windows
        SET window_end = CURRENT_DATE, closed_by = $2::uuid, closed_at = now()
        WHERE id = $1::uuid AND window_end IS NULL
        "#,
    )
    .bind(&window_id)
    .bind(&admin.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'mcc_window.closed', 'mcc_windows', $2, '{}'::jsonb, '{}'::jsonb)
        "#,
    )
    .bind(&admin.id)
    .bind(&window_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- Audit log viewer -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntryView {
    pub id: String,
    pub actor_email: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: String,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

#[server]
pub async fn list_audit_log(limit: i64) -> Result<Vec<AuditLogEntryView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_ANALYTICS_VIEWER, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER, ROLE_ANALYTICS_VIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT audit_log.id::text, admin_users.email AS actor_email, action, target_type, target_id, occurred_at
        FROM audit_log
        LEFT JOIN admin_users ON admin_users.id = audit_log.actor_id
        ORDER BY occurred_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit.clamp(1, 500))
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// ----- Analytics dashboard -----
///
/// Reads only `analytics_rollups_daily` (`migrations/0009_analytics.sql`)
/// — the fully anonymized, indefinitely-retained aggregate table — never
/// `analytics_events` (the 30-day raw table). This page has no per-user
/// drill-down capability at all, by construction: there is no query here
/// that could even ask for one, since `analytics_rollups_daily` has no
/// session- or user-level column to drill into.

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EventTypeCount {
    pub event_type: String,
    pub total_events: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlatformCount {
    pub client_platform: Option<String>,
    pub total_events: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailySessionCount {
    pub bucket_day: chrono::NaiveDate,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub window_days: i32,
    pub by_event_type: Vec<EventTypeCount>,
    pub by_platform: Vec<PlatformCount>,
    pub daily_sessions: Vec<DailySessionCount>,
}

#[server]
pub async fn analytics_summary(window_days: i32) -> Result<AnalyticsSummary, ServerFnError> {
    use crate::auth::{require_role, ROLE_ANALYTICS_VIEWER, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER, ROLE_ANALYTICS_VIEWER]).await?;

    let window_days = window_days.clamp(1, 365);

    let by_event_type: Vec<EventTypeCount> = sqlx::query_as(
        r#"
        SELECT event_type::text, SUM(event_count)::bigint AS total_events
        FROM analytics_rollups_daily
        WHERE bucket_day >= (CURRENT_DATE - $1::int)
        GROUP BY event_type
        ORDER BY total_events DESC
        "#,
    )
    .bind(window_days)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let by_platform: Vec<PlatformCount> = sqlx::query_as(
        r#"
        SELECT client_platform::text, SUM(event_count)::bigint AS total_events
        FROM analytics_rollups_daily
        WHERE bucket_day >= (CURRENT_DATE - $1::int)
        GROUP BY client_platform
        ORDER BY total_events DESC
        "#,
    )
    .bind(window_days)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let daily_sessions: Vec<DailySessionCount> = sqlx::query_as(
        r#"
        SELECT bucket_day, SUM(event_count)::bigint AS session_count
        FROM analytics_rollups_daily
        WHERE event_type = 'session_started' AND bucket_day >= (CURRENT_DATE - $1::int)
        GROUP BY bucket_day
        ORDER BY bucket_day ASC
        "#,
    )
    .bind(window_days)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(AnalyticsSummary { window_days, by_event_type, by_platform, daily_sessions })
}
