//! Server functions backing the two hydrated islands (question/answer
//! widget, language switcher) and the server-rendered KB pages.
//!
//! NOT gated behind `#[cfg(feature = "ssr")]` at the module level — see
//! `lib.rs`'s module doc for why. Per docs/PRD-V2-RUST-PLATFORM.md Section
//! 8.7, public decision-engine "sessions" are never persisted
//! server-side: the browser holds the current `EngineState` (in the
//! question-flow island's reactive signal) and sends it back on every
//! subsequent call; this server only evaluates "given this state and this
//! answer, what's next" via `core-domain`'s pure function and returns the
//! new state plus that node's rendered display data. No database, no
//! session table, no server-side session identifier of any kind.

use core_domain::{DecisionTree, EngineState};
use kb_content::KnowledgeEntry;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::Json;

use crate::render::{render_node, RenderableNode};

fn tree() -> DecisionTree {
    // Re-parses `tree_v2.json` on every call rather than caching behind a
    // process-wide `OnceLock`: this is one JSON parse of a few hundred KB
    // per request, not a hot loop, and keeping the server function fully
    // stateless (no mutable global cache) matches the same "no hidden
    // server-side state" discipline the client-held-session design above
    // is built on.
    core_domain::vote_assist_tree_v2()
}

/// What the question-flow island needs after every step: the opaque state
/// to echo back on the next call, that node's rendered display data for
/// `locale`, whether the walkthrough just reached a terminal outcome, a
/// rough progress estimate for the qualitative progress indicator (per
/// docs/05-information-architecture.md Section 3: "a qualitative/
/// segmented indicator," never a numeric "step X of Y"), and the
/// analytics session id (see `analytics_consent_given` below) the client
/// must echo back unchanged on every `submit_answer` call so the events
/// for one walkthrough correlate — this id is generated fresh per
/// walkthrough, never derived from or linked to any real identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkthroughView {
    pub state: EngineState,
    pub node: RenderableNode,
    pub is_complete: bool,
    pub progress: f64,
    pub session_id: uuid::Uuid,
}

fn build_view(state: EngineState, locale: &str, session_id: uuid::Uuid) -> Result<WalkthroughView, ServerFnError> {
    let tree = tree();
    let node = core_domain::get_current_node(&tree, &state)
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let is_complete = node.is_terminal();
    let rendered = render_node(node, locale);
    let progress =
        core_domain::estimate_progress(&tree, &state).map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(WalkthroughView { state, node: rendered, is_complete, progress, session_id })
}

/// Starts a fresh walkthrough. Called once when a citizen lands on
/// `/start` (or clicks "start over").
#[server]
pub async fn start_walkthrough(locale: String) -> Result<WalkthroughView, ServerFnError> {
    let state = core_domain::create_session(&tree()).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let session_id = uuid::Uuid::new_v4();

    record_walkthrough_event(analytics::EventType::SessionStarted, session_id, &locale, &state.current_node_id, None)
        .await;

    build_view(state, &locale, session_id)
}

/// Advances `state` by answering the current question with `value`. Uses
/// JSON input encoding (rather than the `#[server]` macro's default
/// `serde_qs` form encoding) because `EngineState.history` is a
/// `Vec<Answer>` — a nested, non-flat shape `serde_qs` is not guaranteed
/// to round-trip correctly (see the Leptos book's server-functions
/// chapter, "Quirks to Note").
#[server(input = Json)]
pub async fn submit_answer(
    state: EngineState,
    value: String,
    locale: String,
    session_id: uuid::Uuid,
) -> Result<WalkthroughView, ServerFnError> {
    let answered_node_id = state.current_node_id.clone();
    let next = core_domain::answer(&tree(), &state, &value).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let reached_node_id = next.current_node_id.clone();

    record_walkthrough_event(
        analytics::EventType::QuestionAnswered,
        session_id,
        &locale,
        &answered_node_id,
        Some(&value),
    )
    .await;

    let view = build_view(next, &locale, session_id)?;

    if view.is_complete {
        record_walkthrough_event(analytics::EventType::TerminalReached, session_id, &locale, &reached_node_id, None)
            .await;
    }

    Ok(view)
}

/// Checks `ANALYTICS_CONSENT_COOKIE_NAME` (a plain, non-`HttpOnly` cookie
/// `public/cookie-consent.js` mirrors from `localStorage` specifically so
/// server-side code can read it — see `components::cookie_consent`'s
/// module doc). No cookie, or any value other than `"accepted"`, means no
/// recording — the default is silence, not best-effort tracking.
#[cfg(feature = "ssr")]
async fn analytics_consent_given() -> bool {
    use axum_extra::extract::cookie::CookieJar;

    let Ok(jar) = leptos_axum::extract::<CookieJar>().await else {
        return false;
    };
    jar.get(crate::components::cookie_consent::ANALYTICS_CONSENT_COOKIE_NAME)
        .is_some_and(|cookie| cookie.value() == "accepted")
}

/// Records one decision-tree-walkthrough analytics event (session
/// started/question answered/terminal reached — the three event types
/// that share a `node_id` + decision-tree-version shape) if consent is
/// given, swallowing (and logging) any failure rather than propagating
/// it, since a recording failure must never break the citizen's
/// walkthrough. `record_kb_search_event`/`record_kb_entry_viewed` below
/// repeat the same consent-check-then-swallow-errors policy for the two
/// event types that don't fit this shape (no decision-tree node
/// involved), rather than forcing an artificial fit through this helper.
#[cfg(feature = "ssr")]
async fn record_walkthrough_event(
    event_type: analytics::EventType,
    session_id: uuid::Uuid,
    locale: &str,
    node_id: &str,
    answer_option: Option<&str>,
) {
    if !analytics_consent_given().await {
        return;
    }
    let Some(pool) = use_context::<sqlx::PgPool>() else { return };

    let mut event = analytics::NewEvent::new(event_type, session_id, locale, analytics::ClientPlatform::Web)
        .with_node(tree().version as i32, node_id);
    if let Some(value) = answer_option {
        event = event.with_answer_option(value);
    }

    if let Err(err) = analytics::record_event(&pool, &event).await {
        tracing::warn!(error = %err, "failed to record analytics event");
    }
}

/// Returns every KB entry (used by `/learn` and its section pages, which
/// filter/group what's returned — rendered during SSR via a `Resource`,
/// so no JS is required to see the list).
#[server]
pub async fn list_kb_entries() -> Result<Vec<KnowledgeEntry>, ServerFnError> {
    Ok(kb_content::knowledge_entries().to_vec())
}

/// Full-text-ish search over the curated KB, backing both `/search` and
/// `/learn`'s search box.
#[server]
pub async fn search_kb(query: String) -> Result<Vec<KnowledgeEntry>, ServerFnError> {
    record_kb_search_event(&query).await;
    Ok(kb_content::search_entries(&query).into_iter().cloned().collect())
}

/// Records a `KbSearchPerformed` event — never the raw query text itself
/// (`analytics::NewEvent`'s own doc: `kb_search_category_hash` is "a
/// hash/bucket, not raw text"). Hashed with plain SHA-256 (no secret key):
/// unlike `accounts::hashing`'s contact-identifier blind index, the goal
/// here isn't resisting a targeted guess against a small enumerable
/// space, just avoiding ever storing free text while still letting
/// identical repeated queries bucket together in aggregate. A search
/// isn't part of any decision-tree walkthrough, so it gets its own fresh,
/// uncorrelated session id.
#[cfg(feature = "ssr")]
async fn record_kb_search_event(query: &str) {
    use sha2::{Digest, Sha256};

    if !analytics_consent_given().await {
        return;
    }
    let Some(pool) = use_context::<sqlx::PgPool>() else { return };

    let normalized = query.trim().to_lowercase();
    let category_hash = hex::encode(Sha256::digest(normalized.as_bytes()));

    let event = analytics::NewEvent::new(
        analytics::EventType::KbSearchPerformed,
        uuid::Uuid::new_v4(),
        crate::locale::DEFAULT_LOCALE,
        analytics::ClientPlatform::Web,
    )
    .with_kb_search_category_hash(category_hash);

    if let Err(err) = analytics::record_event(&pool, &event).await {
        tracing::warn!(error = %err, "failed to record analytics event");
    }
}

/// Records a `KbEntryViewed` event for `/learn/:slug` specifically —
/// deliberately NOT called from inside `get_kb_entry` itself, since that
/// function is also reused to resolve citation cards on every terminal
/// outcome (`components::question_flow`), which would conflate "a
/// citizen deliberately opened this article" with "this citation was
/// automatically resolved for display" every single time a walkthrough
/// finishes. Sets `node_id` directly (not via `with_node`, which pairs it
/// with a decision-tree version that has no meaning for a KB-only page
/// view) and its own fresh, uncorrelated session id, same reasoning as
/// `record_kb_search_event`.
#[server]
pub async fn record_kb_entry_viewed(id: String) -> Result<(), ServerFnError> {
    if !analytics_consent_given().await {
        return Ok(());
    }
    let Some(pool) = use_context::<sqlx::PgPool>() else { return Ok(()) };

    let mut event = analytics::NewEvent::new(
        analytics::EventType::KbEntryViewed,
        uuid::Uuid::new_v4(),
        crate::locale::DEFAULT_LOCALE,
        analytics::ClientPlatform::Web,
    );
    event.node_id = Some(id);

    if let Err(err) = analytics::record_event(&pool, &event).await {
        tracing::warn!(error = %err, "failed to record analytics event");
    }
    Ok(())
}

/// A single KB entry by id, backing `/learn/:slug` and the terminal
/// outcome's citation cards. `entry.body` (per
/// `knowledge-base/schema/entry.schema.json`: "Markdown body, plain
/// language") is rendered to `body_html` server-side via `pulldown-cmark`
/// before being sent to the client — rendering the raw Markdown source
/// through `inner_html` would show literal `**`/`#` syntax to the reader,
/// which is a real (if cosmetic) bug, not something to leave as a
/// follow-up. `body` is curated, compliance-reviewed content (per
/// docs/06-legal-compliance-review.md Section 7), never free-form user
/// submission, so server-side rendering it to HTML carries no injection
/// risk analogous to rendering untrusted input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KbEntryDetail {
    pub entry: KnowledgeEntry,
    pub body_html: String,
}

#[server]
pub async fn get_kb_entry(id: String) -> Result<Option<KbEntryDetail>, ServerFnError> {
    Ok(kb_content::get_entry(&id).map(|entry| {
        let mut body_html = String::new();
        pulldown_cmark::html::push_html(&mut body_html, pulldown_cmark::Parser::new(&entry.body));
        KbEntryDetail { entry: entry.clone(), body_html }
    }))
}

/// Entries under one `Topic` — backs `/learn/glossary` (`Topic::Glossary`
/// maps directly). `/learn/forms` and `/learn/faq` don't have a matching
/// single `Topic` (forms-related content is tagged via `relatedForms`,
/// not `topic`; there is no dedicated "is this an FAQ" schema field yet),
/// so those two pages use `list_kb_entries` and filter/group in the page
/// component instead — see `pages::learn`'s module doc for the disclosed
/// limitation on the FAQ page specifically.
#[server]
pub async fn list_entries_by_topic(topic: kb_content::Topic) -> Result<Vec<KnowledgeEntry>, ServerFnError> {
    Ok(kb_content::list_by_topic(topic).into_iter().cloned().collect())
}

/// Submits `/feedback`, writing one row to the `feedback` table
/// (`migrations/0008_feedback_and_fringe_cases.sql`) — the one server
/// function on this site that touches Postgres at all (see `main.rs`'s
/// `DATABASE_URL` doc comment). `contact_email` is optional and, per that
/// migration's own comment, the only identifying field ever stored;
/// `decision_session_tree_version` is informational only (which tree
/// version the reporter was using), never a session identifier — no
/// session id of any kind is ever persisted, consistent with this
/// module's overall no-server-side-session-state discipline.
#[server]
pub async fn submit_feedback(
    category: String,
    message: String,
    kb_entry_id: Option<String>,
    contact_email: Option<String>,
) -> Result<(), ServerFnError> {
    let pool = expect_context::<sqlx::PgPool>();

    // Empty-string form fields arrive as `Some("")` from a plain HTML
    // `<input>`, not `None` — normalize here so the DB gets a real NULL
    // rather than an empty-but-non-null string for optional fields.
    let kb_entry_id = kb_entry_id.filter(|s| !s.trim().is_empty());
    let contact_email = contact_email.filter(|s| !s.trim().is_empty());

    sqlx::query(
        r#"
        INSERT INTO feedback (category, message, kb_entry_id, decision_session_tree_version, contact_email)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(&category)
    .bind(&message)
    .bind(&kb_entry_id)
    .bind(core_domain::vote_assist_tree_v2().version as i32)
    .bind(&contact_email)
    .execute(&pool)
    .await
    .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    Ok(())
}
