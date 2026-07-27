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
    /// See `migrations/0013_kb_entry_grouping_and_faq.sql`'s column
    /// comment — shared by every language variant of the same content,
    /// including the English original (e.g. both `form-6` and
    /// `form-6-hi` set this to `"form-6"`).
    pub translation_group_id: Option<String>,
    /// Tags this entry for `/learn/faq` — a content-curation decision.
    pub is_faq: bool,
}

#[server]
pub async fn get_kb_entry_admin(id: String) -> Result<Option<AdminKbEntryDetail>, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    let row: Option<(
        String,
        String,
        String,
        String,
        String,
        String,
        chrono::NaiveDate,
        String,
        Option<String>,
        Option<String>,
        bool,
    )> = sqlx::query_as(
        r#"
        SELECT id, topic::text, title, summary, body, source_type::text,
               last_verified_date, review_status::text, caution,
               translation_group_id, is_faq
        FROM knowledge_entries
        WHERE id = $1
        "#,
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(row.map(
        |(
            id,
            topic,
            title,
            summary,
            body,
            source_type,
            last_verified_date,
            review_status,
            caution,
            translation_group_id,
            is_faq,
        )| AdminKbEntryDetail {
            id,
            topic,
            title,
            summary,
            body,
            source_type,
            last_verified_date,
            review_status,
            caution,
            translation_group_id,
            is_faq,
        },
    ))
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
        INSERT INTO knowledge_entries (id, topic, title, summary, body, source_type, last_verified_date, review_status, caution, translation_group_id, is_faq, created_by, version)
        VALUES ($1, $2::kb_topic, $3, $4, $5, $6::kb_source_type, $7, $8::kb_review_status, $9, $10, $11, $12::uuid, 1)
        ON CONFLICT (id) DO UPDATE SET
            topic = EXCLUDED.topic,
            title = EXCLUDED.title,
            summary = EXCLUDED.summary,
            body = EXCLUDED.body,
            source_type = EXCLUDED.source_type,
            last_verified_date = EXCLUDED.last_verified_date,
            review_status = EXCLUDED.review_status,
            caution = EXCLUDED.caution,
            translation_group_id = EXCLUDED.translation_group_id,
            is_faq = EXCLUDED.is_faq,
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
    .bind(&entry.translation_group_id)
    .bind(entry.is_faq)
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

/// Revision history for the Content Editor's diff view — reads
/// `knowledge_entry_revisions` (already written, in the same transaction,
/// by every `save_kb_entry` call above; this is the first thing that
/// reads it back). `changed_fields` is computed here (not stored) by
/// comparing each revision's `snapshot` against the immediately preceding
/// one for a fixed set of tracked top-level fields — good enough for "what
/// changed at a glance" without pulling in a general JSON-diff crate for
/// one admin page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbRevisionView {
    pub version: i32,
    pub review_status_at_time: String,
    pub changed_by_email: Option<String>,
    pub change_summary: Option<String>,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub changed_fields: Vec<String>,
}

/// Matches `AdminKbEntryDetail`'s own (unrenamed, snake_case) field names
/// exactly — `snapshot` is `serde_json::to_value(&entry)` of that exact
/// struct in `save_kb_entry` above.
const DIFFABLE_SNAPSHOT_FIELDS: &[&str] = &[
    "title",
    "summary",
    "body",
    "topic",
    "source_type",
    "last_verified_date",
    "review_status",
    "caution",
    "translation_group_id",
    "is_faq",
];

#[server]
pub async fn list_kb_entry_revisions(entry_id: String) -> Result<Vec<KbRevisionView>, ServerFnError> {
    use crate::auth::require_admin;

    let pool = expect_context::<sqlx::PgPool>();
    require_admin(&pool).await?;

    let rows: Vec<(i32, serde_json::Value, String, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT knowledge_entry_revisions.version, snapshot, review_status_at_time::text,
                   admin_users.email, change_summary, changed_at
            FROM knowledge_entry_revisions
            LEFT JOIN admin_users ON admin_users.id = knowledge_entry_revisions.changed_by
            WHERE entry_id = $1
            ORDER BY version ASC
            "#,
        )
        .bind(&entry_id)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let mut views = Vec::with_capacity(rows.len());
    let mut previous_snapshot: Option<serde_json::Value> = None;

    for (version, snapshot, review_status_at_time, changed_by_email, change_summary, changed_at) in rows {
        let changed_fields = match &previous_snapshot {
            None => DIFFABLE_SNAPSHOT_FIELDS.iter().map(|f| f.to_string()).collect(),
            Some(previous) => DIFFABLE_SNAPSHOT_FIELDS
                .iter()
                .filter(|field| previous.get(**field) != snapshot.get(**field))
                .map(|f| f.to_string())
                .collect(),
        };

        views.push(KbRevisionView {
            version,
            review_status_at_time,
            changed_by_email,
            change_summary,
            changed_at,
            changed_fields,
        });
        previous_snapshot = Some(snapshot);
    }

    views.reverse(); // newest first, matching every other list page in this app
    Ok(views)
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

/// ----- Feedback & grievance triage -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FeedbackView {
    pub id: String,
    pub category: String,
    pub message: String,
    pub kb_entry_id: Option<String>,
    pub contact_email: Option<String>,
    pub status: String,
    pub internal_notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// `status_filter` of `None` (or `"all"`) returns every row, most recent
/// first, capped at 200 — this page triages a working queue, not an
/// archive browser; older resolved items are still in the table for the
/// Data Export & Retention page's sweep, just not shown here by default.
#[server]
pub async fn list_feedback(status_filter: Option<String>) -> Result<Vec<FeedbackView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let query = r#"
        SELECT id::text, category, message, kb_entry_id, contact_email, status::text,
               internal_notes, created_at, resolved_at
        FROM feedback
        WHERE $1::text IS NULL OR status::text = $1
        ORDER BY created_at DESC
        LIMIT 200
    "#;

    let normalized_filter = status_filter.filter(|s| s != "all");

    sqlx::query_as(query)
        .bind(normalized_filter)
        .fetch_all(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Updates one feedback row's triage status and internal notes in one
/// transaction, plus an `audit_log` row — `resolved_at` is set
/// automatically the first time a row moves to `resolved`/`wontfix`
/// (and left untouched on any later edit, so it always reflects when
/// triage actually concluded, not the most recent save).
#[server]
pub async fn update_feedback_status(
    id: String,
    status: String,
    internal_notes: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        UPDATE feedback
        SET status = $2::feedback_status,
            internal_notes = $3,
            resolved_at = CASE
                WHEN resolved_at IS NULL AND $2::feedback_status IN ('resolved', 'wontfix') THEN now()
                ELSE resolved_at
            END
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(&status)
    .bind(&internal_notes)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'feedback.status_updated', 'feedback', $2, '{}'::jsonb, jsonb_build_object('status', $3::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&id)
    .bind(&status)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- Citation & link health -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LinkHealthRow {
    pub source_id: String,
    pub entry_id: String,
    pub source_title: String,
    pub url: String,
    pub http_status: Option<i32>,
    pub error_message: Option<String>,
    pub checked_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// One row per `knowledge_entry_sources` entry, joined to its most recent
/// `link_check_results` row (if any — a source cited today but not yet
/// checked by the nightly `jobs::link_checker` run shows up with
/// `checked_at: None`, not silently omitted). `DISTINCT ON` picks the
/// single latest check per source; this is a small, curated citation list
/// (tens of rows, not thousands), so no pagination.
#[server]
pub async fn list_link_health() -> Result<Vec<LinkHealthRow>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT DISTINCT ON (knowledge_entry_sources.id)
            knowledge_entry_sources.id::text AS source_id,
            knowledge_entry_sources.entry_id,
            knowledge_entry_sources.title AS source_title,
            knowledge_entry_sources.url,
            link_check_results.http_status,
            link_check_results.error_message,
            link_check_results.checked_at
        FROM knowledge_entry_sources
        LEFT JOIN link_check_results ON link_check_results.source_id = knowledge_entry_sources.id
        ORDER BY knowledge_entry_sources.id, link_check_results.checked_at DESC NULLS LAST
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Same query as `list_link_health`, scoped to one entry — backs the
/// Knowledge Base Content Editor's inline citation-health section rather
/// than duplicating this join there.
#[server]
pub async fn list_link_health_for_entry(entry_id: String) -> Result<Vec<LinkHealthRow>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT DISTINCT ON (knowledge_entry_sources.id)
            knowledge_entry_sources.id::text AS source_id,
            knowledge_entry_sources.entry_id,
            knowledge_entry_sources.title AS source_title,
            knowledge_entry_sources.url,
            link_check_results.http_status,
            link_check_results.error_message,
            link_check_results.checked_at
        FROM knowledge_entry_sources
        LEFT JOIN link_check_results ON link_check_results.source_id = knowledge_entry_sources.id
        WHERE knowledge_entry_sources.entry_id = $1
        ORDER BY knowledge_entry_sources.id, link_check_results.checked_at DESC NULLS LAST
        "#,
    )
    .bind(&entry_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Performs one live HEAD request right now (reusing `jobs::build_client`'s
/// courteously-identified HTTP client, not a second, differently-behaved
/// client) and records the outcome as a new `link_check_results` row —
/// same table the nightly job writes to, so this and the scheduled run
/// share one history rather than keeping separate "manual" vs "automatic"
/// check logs.
#[server]
pub async fn check_link_now(source_id: String) -> Result<LinkHealthRow, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let source: Option<(String, String, String)> = sqlx::query_as(
        "SELECT entry_id, title, url FROM knowledge_entry_sources WHERE id = $1::uuid",
    )
    .bind(&source_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let (entry_id, source_title, url) =
        source.ok_or_else(|| ServerFnError::ServerError("no such citation source".to_string()))?;

    let client = jobs::build_client().map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let (http_status, error_message) = match client.head(url.as_str()).send().await {
        Ok(response) => (Some(response.status().as_u16() as i32), None),
        Err(err) => (None, Some(err.to_string())),
    };

    sqlx::query(
        r#"
        INSERT INTO link_check_results (source_id, source_url, http_status, error_message)
        VALUES ($1::uuid, $2, $3, $4)
        "#,
    )
    .bind(&source_id)
    .bind(&url)
    .bind(http_status)
    .bind(&error_message)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(LinkHealthRow {
        source_id,
        entry_id,
        source_title,
        url,
        http_status,
        error_message,
        checked_at: Some(chrono::Utc::now()),
    })
}

/// ----- Bot channel management -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BotChannelConfigView {
    pub id: String,
    pub channel: String,
    pub display_name: String,
    pub is_enabled: bool,
    pub webhook_configured: bool,
    pub last_health_check_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_health_check_ok: Option<bool>,
}

#[server]
pub async fn list_bot_channels() -> Result<Vec<BotChannelConfigView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT id::text, channel::text, display_name, is_enabled, webhook_configured,
               last_health_check_at, last_health_check_ok
        FROM bot_channel_config
        ORDER BY channel
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// A live per-channel kill-switch, independent of MCC status — this is
/// the same `bot_channel_config` row `mcc_panel.rs`'s module doc names as
/// a disclosed follow-up. Gated to `legal_reviewer` (not `reviewer`),
/// matching MCC window gating: this flips whether real users can reach a
/// live bot channel at all, not a content edit.
#[server]
pub async fn set_bot_channel_enabled(channel: String, is_enabled: bool) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE bot_channel_config SET is_enabled = $2 WHERE channel = $1::client_platform")
        .bind(&channel)
        .bind(is_enabled)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'bot_channel.toggled', 'bot_channel_config', $2, '{}'::jsonb, jsonb_build_object('is_enabled', $3::bool))
        "#,
    )
    .bind(&admin.id)
    .bind(&channel)
    .bind(is_enabled)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WhatsappTemplateView {
    pub id: String,
    pub template_name: String,
    pub template_body: String,
    pub locale: String,
    pub meta_approval_status: String,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[server]
pub async fn list_whatsapp_templates() -> Result<Vec<WhatsappTemplateView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT id::text, template_name, template_body, locale, meta_approval_status,
               submitted_at, approved_at
        FROM whatsapp_message_templates
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Registers a template as submitted for Meta's pre-approval review
/// (`submitted_at` is set immediately — this row IS the record of "we
/// asked Meta to review this", not a draft stage before submission).
/// `translator` can propose template copy; only `legal_reviewer`/
/// `superadmin` can later record Meta's actual decision via
/// `set_whatsapp_template_status`.
#[server]
pub async fn create_whatsapp_template(
    template_name: String,
    template_body: String,
    locale: String,
) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER, ROLE_TRANSLATOR};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER, ROLE_TRANSLATOR]).await?;

    let id: String = sqlx::query_scalar(
        r#"
        INSERT INTO whatsapp_message_templates (template_name, template_body, locale, submitted_at)
        VALUES ($1, $2, $3, now())
        RETURNING id::text
        "#,
    )
    .bind(&template_name)
    .bind(&template_body)
    .bind(&locale)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'whatsapp_template.submitted', 'whatsapp_message_templates', $2, '{}'::jsonb, jsonb_build_object('template_name', $3::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&id)
    .bind(&template_name)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// Records Meta's actual approval decision — this never triggers or
/// checks anything against Meta's API itself (no such integration exists
/// in this codebase); it's the admin recording, by hand, what Meta's
/// dashboard already says, so `bot-whatsapp`'s proactive-send path has a
/// database row to check before using a template outside the 24-hour
/// customer-service window.
#[server]
pub async fn set_whatsapp_template_status(id: String, status: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        UPDATE whatsapp_message_templates
        SET meta_approval_status = $2,
            approved_at = CASE WHEN $2 = 'approved' AND approved_at IS NULL THEN now() ELSE approved_at END
        WHERE id = $1::uuid
        "#,
    )
    .bind(&id)
    .bind(&status)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'whatsapp_template.status_updated', 'whatsapp_message_templates', $2, '{}'::jsonb, jsonb_build_object('status', $3::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&id)
    .bind(&status)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- Translation management -----

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TranslationStatusRow {
    pub locale: String,
    pub content_type: String,
    pub total_items: i32,
    pub translated_items: i32,
    pub reviewed_items: i32,
    pub computed_at: chrono::DateTime<chrono::Utc>,
}

/// Reads the latest `translation_status` row per (locale, content_type) —
/// that table accumulates one row per computation run rather than
/// updating in place (`migrations/0010`'s comment), so `DISTINCT ON`
/// picks only the newest. Does not compute anything itself; see
/// `recompute_translation_status` for that.
#[server]
pub async fn list_translation_status() -> Result<Vec<TranslationStatusRow>, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER, ROLE_TRANSLATOR};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER, ROLE_TRANSLATOR]).await?;

    sqlx::query_as(
        r#"
        SELECT DISTINCT ON (locale, content_type)
            locale, content_type, total_items, translated_items, reviewed_items, computed_at
        FROM translation_status
        ORDER BY locale, content_type, computed_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Recomputes KB-entry translation completeness right now (writes a fresh
/// `translation_status` row per tracked locale) instead of waiting for
/// `jobs`' nightly scheduled run — reuses that crate's own
/// `TRACKED_LOCALES` list and `compute_kb_translation_status` function
/// directly rather than a second copy of either. See
/// `compute_kb_translation_status`'s own module doc for the "row count,
/// not strict 1:1 correlation" caveat this inherits.
#[server]
pub async fn recompute_translation_status() -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_TRANSLATOR};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_TRANSLATOR, ROLE_LEGAL_REVIEWER]).await?;

    jobs::compute_kb_translation_status(&pool, jobs::TRACKED_LOCALES)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// ----- User & role management -----

const VALID_ADMIN_ROLES: &[&str] =
    &["contributor", "reviewer", "legal_reviewer", "translator", "analytics_viewer", "superadmin"];

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AdminUserView {
    pub id: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Superadmin-only, same as every mutation below — creating/deactivating
/// admin accounts and reassigning roles is the one page in this app
/// where `reviewer`/`legal_reviewer` (who can approve everything else)
/// still can't act, matching PRD v2 Section 11's "who manages the
/// managers" boundary.
#[server]
pub async fn list_admin_users() -> Result<Vec<AdminUserView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    sqlx::query_as(
        r#"
        SELECT id::text, email, role::text, is_active, created_at, last_login_at
        FROM admin_users
        ORDER BY email
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Mirrors `xtask create-admin`'s validation exactly (same
/// `VALID_ADMIN_ROLES` list, same argon2 hashing) — this page and the CLI
/// are two front doors to the same operation, not two different
/// implementations of it. Unlike the CLI (which exists specifically
/// because `admin_users` starts empty), this is how every admin account
/// *after* the first superadmin gets created.
#[server]
pub async fn create_admin_user(email: String, password: String, role: String) -> Result<(), ServerFnError> {
    use crate::auth::{hash_password, require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    if !VALID_ADMIN_ROLES.contains(&role.as_str()) {
        return Err(ServerFnError::ServerError(format!(
            "\"{role}\" is not a valid role — must be one of: {}",
            VALID_ADMIN_ROLES.join(", ")
        )));
    }
    if password.len() < 12 {
        return Err(ServerFnError::ServerError("password must be at least 12 characters".to_string()));
    }

    let password_hash = hash_password(&password).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let new_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO admin_users (email, password_hash, role)
        VALUES ($1, $2, $3::admin_role)
        RETURNING id::text
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .bind(&role)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(format!("failed to create admin user (does this email already exist?): {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'admin_user.created', 'admin_users', $2, '{}'::jsonb, jsonb_build_object('email', $3::text, 'role', $4::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&new_id)
    .bind(&email)
    .bind(&role)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// Refuses to let a superadmin deactivate their own account — a cheap
/// guard against an easy, hard-to-undo self-lockout mistake (there is no
/// "reactivate yourself" path once the only active superadmin session is
/// gone; the operator would be back to needing direct database access or
/// `xtask create-admin` against production Postgres).
#[server]
pub async fn set_admin_user_active(id: String, is_active: bool) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    if !is_active && admin.id == id {
        return Err(ServerFnError::ServerError("you cannot deactivate your own account".to_string()));
    }

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE admin_users SET is_active = $2 WHERE id = $1::uuid")
        .bind(&id)
        .bind(is_active)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'admin_user.active_toggled', 'admin_users', $2, '{}'::jsonb, jsonb_build_object('is_active', $3::bool))
        "#,
    )
    .bind(&admin.id)
    .bind(&id)
    .bind(is_active)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// Refuses to let a superadmin demote themselves away from `superadmin`
/// — same self-lockout reasoning as `set_admin_user_active` above.
#[server]
pub async fn set_admin_user_role(id: String, role: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    if !VALID_ADMIN_ROLES.contains(&role.as_str()) {
        return Err(ServerFnError::ServerError(format!(
            "\"{role}\" is not a valid role — must be one of: {}",
            VALID_ADMIN_ROLES.join(", ")
        )));
    }
    if admin.id == id && role != ROLE_SUPERADMIN {
        return Err(ServerFnError::ServerError("you cannot remove your own superadmin role".to_string()));
    }

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE admin_users SET role = $2::admin_role WHERE id = $1::uuid")
        .bind(&id)
        .bind(&role)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'admin_user.role_changed', 'admin_users', $2, '{}'::jsonb, jsonb_build_object('role', $3::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&id)
    .bind(&role)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// ----- Data export & retention tools -----

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetentionSweepSummary {
    pub admin_sessions_purged: i64,
    pub account_sessions_purged: i64,
    pub otp_challenges_purged: i64,
}

/// The same three `DELETE ... WHERE expires_at/expiry_date < now()`
/// statements as `xtask purge-expired-sessions` (and the cron entry
/// `scripts/purge-expired-sessions.sh` schedules) — this button exists so
/// a superadmin can run the sweep on demand (e.g. right after handling a
/// DPDP data-subject request) without shell access to the box `xtask`
/// runs on.
#[server]
pub async fn purge_expired_sessions_now() -> Result<RetentionSweepSummary, ServerFnError> {
    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    let admin_sessions_purged = sqlx::query("DELETE FROM sessions WHERE expiry_date < now()")
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .rows_affected() as i64;

    let account_sessions_purged = sqlx::query("DELETE FROM account_sessions WHERE expiry_date < now()")
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .rows_affected() as i64;

    let otp_challenges_purged = sqlx::query("DELETE FROM account_otp_challenges WHERE expires_at < now()")
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?
        .rows_affected() as i64;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'retention.sessions_purged', 'sessions', 'all', '{}'::jsonb,
                jsonb_build_object('admin_sessions', $2::bigint, 'account_sessions', $3::bigint, 'otp_challenges', $4::bigint))
        "#,
    )
    .bind(&admin.id)
    .bind(admin_sessions_purged)
    .bind(account_sessions_purged)
    .bind(otp_challenges_purged)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(RetentionSweepSummary { admin_sessions_purged, account_sessions_purged, otp_challenges_purged })
}

/// Enforces `migrations/0008_feedback_and_fringe_cases.sql`'s documented
/// retention policy: "180 days post-resolution" for `feedback.contact_email`
/// specifically — the message/category text is anonymized product-
/// feedback history and is kept, only the one identifying field is
/// cleared. Rows with no `resolved_at` (still open) are never touched,
/// regardless of age.
#[server]
pub async fn purge_old_feedback_contact_emails() -> Result<i64, ServerFnError> {
    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    let purged: i64 = sqlx::query(
        r#"
        UPDATE feedback
        SET contact_email = NULL
        WHERE contact_email IS NOT NULL
          AND resolved_at IS NOT NULL
          AND resolved_at < now() - INTERVAL '180 days'
        "#,
    )
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?
    .rows_affected() as i64;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'retention.feedback_contact_emails_purged', 'feedback', 'all', '{}'::jsonb, jsonb_build_object('count', $2::bigint))
        "#,
    )
    .bind(&admin.id)
    .bind(purged)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(purged)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct AuditLogExportRow {
    id: String,
    actor_email: Option<String>,
    action: String,
    target_type: String,
    target_id: String,
    before_value: Option<serde_json::Value>,
    after_value: Option<serde_json::Value>,
    occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Exports every `audit_log` row in `[from_date, to_date]` (inclusive) as
/// a `data:` URI the page turns into a download link — per
/// `audit_log`'s own table comment, "exportable for external compliance
/// review" (PRD v2 admin page 13). Base64-encoded rather than percent-
/// encoded so arbitrary JSONB content (which may contain non-ASCII text,
/// e.g. a non-English knowledge-base title in a `before_value`/
/// `after_value` diff) round-trips correctly without a second escaping
/// scheme to get right.
#[server]
pub async fn export_audit_log(from_date: chrono::NaiveDate, to_date: chrono::NaiveDate) -> Result<String, ServerFnError> {
    use base64::Engine;

    use crate::auth::{require_role, ROLE_SUPERADMIN};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_SUPERADMIN]).await?;

    let rows: Vec<AuditLogExportRow> = sqlx::query_as(
        r#"
        SELECT audit_log.id::text, admin_users.email AS actor_email, action, target_type, target_id,
               before_value, after_value, occurred_at
        FROM audit_log
        LEFT JOIN admin_users ON admin_users.id = audit_log.actor_id
        WHERE occurred_at::date BETWEEN $1 AND $2
        ORDER BY occurred_at ASC
        "#,
    )
    .bind(from_date)
    .bind(to_date)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let json = serde_json::to_string_pretty(&rows).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(json.as_bytes());

    Ok(format!("data:application/json;charset=utf-8;base64,{encoded}"))
}

/// ----- Decision tree visual editor -----
///
/// "Visual" here means a structured JSON editor with validate/publish/
/// rollback around `decision_tree_drafts`/`decision_trees`
/// (`migrations/0003_decision_trees.sql`), not a drag-and-drop node-graph
/// canvas — a disclosed, deliberate simplification. A graphical canvas is
/// a substantial standalone JS component this pass doesn't attempt to
/// hand-write without being able to compile/run it; the JSON artifact
/// underneath is identical either way, so a canvas UI could be layered on
/// top of these same server functions later without changing this file.

/// The only two `tree_key` values this codebase actually has content
/// for — `core_domain::vote_assist_tree_v1`/`v2`'s compiled-in JSON.
/// `decision_trees`/`decision_tree_drafts` start empty (no seed migration
/// inserts rows, unlike e.g. `bot_channel_config`), so `get_or_create_draft`
/// falls back to these when nothing has ever been published yet — the
/// same "no sync job populates the database from the file yet" gap
/// `jobs::link_checker`'s module doc already discloses for
/// `knowledge_entries`.
const KNOWN_TREE_KEYS: &[&str] = &["voteassist-core-v1", "voteassist-core-v2"];

fn seed_tree_for_key(tree_key: &str) -> Option<&'static core_domain::DecisionTree> {
    match tree_key {
        "voteassist-core-v1" => Some(core_domain::vote_assist_tree_v1()),
        "voteassist-core-v2" => Some(core_domain::vote_assist_tree_v2()),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftView {
    pub id: String,
    pub tree_key: String,
    pub based_on_version: i32,
    pub artifact_json: String,
    pub last_validation_issues: Vec<core_domain::TreeValidationIssue>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

fn build_draft_view(
    id: String,
    tree_key: String,
    based_on_version: i32,
    artifact: serde_json::Value,
    last_validation_issues_json: serde_json::Value,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> DraftView {
    let artifact_json = serde_json::to_string_pretty(&artifact).unwrap_or_else(|_| artifact.to_string());
    let last_validation_issues: Vec<core_domain::TreeValidationIssue> =
        serde_json::from_value(last_validation_issues_json).unwrap_or_default();
    DraftView { id, tree_key, based_on_version, artifact_json, last_validation_issues, updated_at }
}

#[server]
pub async fn list_tree_keys() -> Result<Vec<String>, ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let mut keys: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT tree_key FROM decision_trees
        UNION
        SELECT tree_key FROM decision_tree_drafts
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    for known in KNOWN_TREE_KEYS {
        if !keys.iter().any(|k| k == known) {
            keys.push(known.to_string());
        }
    }
    keys.sort();
    Ok(keys)
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DecisionTreeVersionSummary {
    pub version: i32,
    pub is_active: bool,
    pub validated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub published_by_email: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[server]
pub async fn list_tree_versions(tree_key: String) -> Result<Vec<DecisionTreeVersionSummary>, ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT decision_trees.version, is_active, validated_at, admin_users.email AS published_by_email, published_at
        FROM decision_trees
        LEFT JOIN admin_users ON admin_users.id = decision_trees.published_by
        WHERE tree_key = $1
        ORDER BY version DESC
        "#,
    )
    .bind(&tree_key)
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Returns the most recently updated draft for `tree_key`, creating one
/// if none exists yet: seeded from the currently active published
/// version, or — if nothing has ever been published — from
/// `seed_tree_for_key`'s compiled-in default.
#[server]
pub async fn get_or_create_draft(tree_key: String) -> Result<DraftView, ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let existing: Option<(String, i32, serde_json::Value, serde_json::Value, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT id::text, based_on_version, artifact, last_validation_issues, updated_at
            FROM decision_tree_drafts
            WHERE tree_key = $1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(&tree_key)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some((id, based_on_version, artifact, issues, updated_at)) = existing {
        return Ok(build_draft_view(id, tree_key, based_on_version, artifact, issues, updated_at));
    }

    let active: Option<(serde_json::Value, i32)> =
        sqlx::query_as("SELECT artifact, version FROM decision_trees WHERE tree_key = $1 AND is_active")
            .bind(&tree_key)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let (artifact, based_on_version) = match active {
        Some((artifact, version)) => (artifact, version),
        None => {
            let seed = seed_tree_for_key(&tree_key).ok_or_else(|| {
                ServerFnError::ServerError(format!(
                    "no published version of \"{tree_key}\" exists yet, and it isn't one of this \
                     codebase's known built-in trees to start a draft from"
                ))
            })?;
            let artifact = serde_json::to_value(seed).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
            (artifact, 0)
        }
    };

    let id: String = sqlx::query_scalar(
        r#"
        INSERT INTO decision_tree_drafts (tree_key, based_on_version, artifact)
        VALUES ($1, $2, $3)
        RETURNING id::text
        "#,
    )
    .bind(&tree_key)
    .bind(based_on_version)
    .bind(&artifact)
    .fetch_one(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(build_draft_view(id, tree_key, based_on_version, artifact, serde_json::json!([]), chrono::Utc::now()))
}

/// Overwrites a draft's working JSON. Only checked for well-formed JSON
/// here (not a valid `DecisionTree` shape) — mid-edit JSON is expected to
/// be temporarily invalid, per `decision_tree_drafts`'s own table
/// comment; `validate_draft` is the actual structural check.
#[server]
pub async fn save_draft_artifact(draft_id: String, artifact_json: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let artifact: serde_json::Value =
        serde_json::from_str(&artifact_json).map_err(|e| ServerFnError::ServerError(format!("not valid JSON: {e}")))?;

    sqlx::query(
        "UPDATE decision_tree_drafts SET artifact = $2, last_validation_issues = '[]'::jsonb WHERE id = $1::uuid",
    )
    .bind(&draft_id)
    .bind(&artifact)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// Runs `core_domain::validate_tree` against a draft's current artifact
/// (an unparseable artifact — mid-edit invalid JSON, or valid JSON that
/// doesn't match `DecisionTree`'s shape — surfaces as a single issue
/// rather than a hard error, so the editor UI has something concrete to
/// show either way) and caches the result on the draft row for display
/// without re-validating on every page load.
#[server]
pub async fn validate_draft(draft_id: String) -> Result<Vec<core_domain::TreeValidationIssue>, ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let artifact: Option<serde_json::Value> =
        sqlx::query_scalar("SELECT artifact FROM decision_tree_drafts WHERE id = $1::uuid")
            .bind(&draft_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let artifact = artifact.ok_or_else(|| ServerFnError::ServerError("draft not found".to_string()))?;

    let issues = match serde_json::from_value::<core_domain::DecisionTree>(artifact) {
        Ok(tree) => core_domain::validate_tree(&tree),
        Err(e) => vec![core_domain::TreeValidationIssue {
            node_id: "(parse)".to_string(),
            message: format!("artifact does not match the DecisionTree JSON shape: {e}"),
        }],
    };

    let issues_json = serde_json::to_value(&issues).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    sqlx::query("UPDATE decision_tree_drafts SET last_validation_issues = $2 WHERE id = $1::uuid")
        .bind(&draft_id)
        .bind(&issues_json)
        .execute(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(issues)
}

/// Publishing is gated to `legal_reviewer`/`superadmin` — same tier as
/// MCC windows — and always re-validates the artifact itself rather than
/// trusting the draft's cached `last_validation_issues` (which could be
/// stale if the artifact changed after the last "Validate" click without
/// a re-check). Returns the new version number on success.
#[server]
pub async fn publish_draft(draft_id: String) -> Result<i32, ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let row: Option<(String, serde_json::Value)> =
        sqlx::query_as("SELECT tree_key, artifact FROM decision_tree_drafts WHERE id = $1::uuid")
            .bind(&draft_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let (tree_key, artifact) = row.ok_or_else(|| ServerFnError::ServerError("draft not found".to_string()))?;

    let tree: core_domain::DecisionTree = serde_json::from_value(artifact.clone())
        .map_err(|e| ServerFnError::ServerError(format!("artifact does not match the DecisionTree JSON shape: {e}")))?;

    let issues = core_domain::validate_tree(&tree);
    if !issues.is_empty() {
        let summary = issues.iter().map(|i| format!("{}: {}", i.node_id, i.message)).collect::<Vec<_>>().join("; ");
        return Err(ServerFnError::ServerError(format!(
            "cannot publish — {} validation issue(s): {summary}",
            issues.len()
        )));
    }

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let next_version: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM decision_trees WHERE tree_key = $1")
            .bind(&tree_key)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE decision_trees SET is_active = false WHERE tree_key = $1 AND is_active")
        .bind(&tree_key)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO decision_trees (tree_key, version, artifact, is_active, validated_at, published_by, published_at)
        VALUES ($1, $2, $3, true, now(), $4::uuid, now())
        "#,
    )
    .bind(&tree_key)
    .bind(next_version)
    .bind(&artifact)
    .bind(&admin.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("DELETE FROM decision_tree_drafts WHERE id = $1::uuid")
        .bind(&draft_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'tree.published', 'decision_trees', $2, '{}'::jsonb, jsonb_build_object('version', $3::int))
        "#,
    )
    .bind(&admin.id)
    .bind(&tree_key)
    .bind(next_version)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(next_version)
}

/// Rolling back never creates a new row — per `decision_trees`'s own
/// table comment, an existing version's `is_active` just flips back on,
/// exactly like a fresh publish flips it, so the version-history list
/// stays a plain append-only sequence either way.
#[server]
pub async fn rollback_tree_version(tree_key: String, version: i32) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_LEGAL_REVIEWER]).await?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let exists: Option<i32> =
        sqlx::query_scalar("SELECT version FROM decision_trees WHERE tree_key = $1 AND version = $2")
            .bind(&tree_key)
            .bind(version)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    if exists.is_none() {
        return Err(ServerFnError::ServerError(format!("no version {version} of \"{tree_key}\" exists")));
    }

    sqlx::query("UPDATE decision_trees SET is_active = false WHERE tree_key = $1 AND is_active")
        .bind(&tree_key)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE decision_trees SET is_active = true WHERE tree_key = $1 AND version = $2")
        .bind(&tree_key)
        .bind(version)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'tree.rolled_back', 'decision_trees', $2, '{}'::jsonb, jsonb_build_object('version', $3::int))
        "#,
    )
    .bind(&admin.id)
    .bind(&tree_key)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}

/// Cross-referenced on the tree editor page per `fringe_case`'s own table
/// comment ("surfaces open fringe_case rows inline"). `fringe_case` has
/// no `tree_key` column (only a bare `linked_tree_node_id`, since node
/// ids live in JSONB, not a relational column with a real FK) — so this
/// lists every open report across all trees, not scoped to one tree_key;
/// the page labels it accordingly rather than implying a scoping that
/// doesn't exist in the schema.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FringeCaseView {
    pub id: String,
    pub description: String,
    pub reporter_channel: String,
    pub status: String,
    pub linked_tree_node_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[server]
pub async fn list_open_fringe_cases() -> Result<Vec<FringeCaseView>, ServerFnError> {
    use crate::auth::{require_role, ROLE_CONTRIBUTOR, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    require_role(&pool, &[ROLE_CONTRIBUTOR, ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    sqlx::query_as(
        r#"
        SELECT id::text, description, reporter_channel::text, status::text, linked_tree_node_id, created_at
        FROM fringe_case
        WHERE status NOT IN ('wontfix', 'added_to_tree')
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

/// Files a new `fringe_case` row from an existing `feedback` row —
/// `description`/`linked_kb_entry_id` are copied over, `reporter_channel`
/// is always `'feedback_form'` (the report genuinely did originate from
/// the public feedback form; this button just formalizes it into the
/// tracked fringe-case backlog `pages::tree_editor` cross-references), and
/// the source feedback row is marked `triaged` if it was still `new` —
/// never overwriting a more specific status (e.g. `resolved`) a reviewer
/// already set.
#[server]
pub async fn convert_feedback_to_fringe_case(feedback_id: String) -> Result<(), ServerFnError> {
    use crate::auth::{require_role, ROLE_LEGAL_REVIEWER, ROLE_REVIEWER};

    let pool = expect_context::<sqlx::PgPool>();
    let admin = require_role(&pool, &[ROLE_REVIEWER, ROLE_LEGAL_REVIEWER]).await?;

    let feedback: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT message, kb_entry_id FROM feedback WHERE id = $1::uuid")
            .bind(&feedback_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let (message, kb_entry_id) =
        feedback.ok_or_else(|| ServerFnError::ServerError("feedback not found".to_string()))?;

    let mut tx = pool.begin().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    let fringe_case_id: String = sqlx::query_scalar(
        r#"
        INSERT INTO fringe_case (description, reporter_channel, linked_feedback_id, linked_kb_entry_id)
        VALUES ($1, 'feedback_form'::fringe_case_channel, $2::uuid, $3)
        RETURNING id::text
        "#,
    )
    .bind(&message)
    .bind(&feedback_id)
    .bind(&kb_entry_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query("UPDATE feedback SET status = 'triaged' WHERE id = $1::uuid AND status = 'new'")
        .bind(&feedback_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO audit_log (actor_id, action, target_type, target_id, before_value, after_value)
        VALUES ($1::uuid, 'feedback.converted_to_fringe_case', 'feedback', $2, '{}'::jsonb, jsonb_build_object('fringe_case_id', $3::text))
        "#,
    )
    .bind(&admin.id)
    .bind(&feedback_id)
    .bind(&fringe_case_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    tx.commit().await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
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
