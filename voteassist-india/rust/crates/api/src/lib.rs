//! Axum HTTP API for VoteAssist India.
//!
//! Thin HTTP layer over `core-domain` (decision engine) and `kb-content`
//! (knowledge base) — no decision logic lives here, only request/response
//! translation, per docs/PRD-V2-RUST-PLATFORM.md Section 6.3/6.7.

pub mod handlers;
pub mod state;

use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::start_session,
        handlers::submit_answer,
        handlers::search_kb,
        handlers::get_kb_entry,
        handlers::healthz,
    ),
    components(schemas(handlers::SessionResponse, handlers::AnswerRequest, handlers::ApiError, handlers::SearchQuery)),
    tags(
        (name = "sessions", description = "Anonymous, stateless decision-engine sessions"),
        (name = "knowledge-base", description = "Curated, cited knowledge base"),
    )
)]
pub struct ApiDoc;

/// Builds the full Axum router. Exposed as a function (rather than only a
/// `main`-local value) so integration tests exercise the exact same router
/// the production binary serves — see `tests/http.rs`.
pub fn build_router() -> Router {
    Router::new()
        .route("/healthz", get(handlers::healthz))
        .route("/v1/sessions", post(handlers::start_session))
        .route("/v1/sessions/answers", post(handlers::submit_answer))
        .route("/v1/kb/search", get(handlers::search_kb))
        .route("/v1/kb/entries/:id", get(handlers::get_kb_entry))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
}
