use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use core_domain::{
    answer, create_session, get_current_node, is_session_complete, vote_assist_tree_v1, EngineError,
};
use kb_content::{get_entry, search_entries};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::{self, StateTokenError};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    /// Opaque, client-held session token — echo this back on every
    /// subsequent request. Carries no privilege; see `state.rs`.
    pub state: String,
    pub tree_id: String,
    pub tree_version: u32,
    pub is_complete: bool,
    /// The current question or terminal node, serialized exactly as
    /// `core_domain::DecisionNode` (an internally-tagged `{"type": "question"
    /// | "terminal", ...}` object) — see `core-domain` for the authoritative
    /// shape; represented here as an open object for the OpenAPI schema.
    #[schema(value_type = Object)]
    pub node: serde_json::Value,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AnswerRequest {
    pub state: String,
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (self.0, Json(ApiError { error: self.1 })).into_response()
    }
}

pub struct ApiErrorResponse(pub StatusCode, pub String);

fn engine_error_response(err: EngineError) -> ApiErrorResponse {
    match err {
        EngineError::InvalidAnswer { .. } => {
            ApiErrorResponse(StatusCode::BAD_REQUEST, err.to_string())
        }
        EngineError::TerminalReached { .. } => {
            ApiErrorResponse(StatusCode::CONFLICT, err.to_string())
        }
        EngineError::UnknownNode { .. } => {
            ApiErrorResponse(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
        }
    }
}

fn session_token_error_response(err: StateTokenError) -> ApiErrorResponse {
    ApiErrorResponse(StatusCode::BAD_REQUEST, err.to_string())
}

/// Start a new, fully anonymous decision-engine session.
#[utoipa::path(
    post,
    path = "/v1/sessions",
    responses((status = 201, description = "Session created", body = SessionResponse)),
    tag = "sessions"
)]
pub async fn start_session() -> Result<(StatusCode, Json<SessionResponse>), ApiErrorResponse> {
    let tree = vote_assist_tree_v1();
    let session = create_session(tree).map_err(engine_error_response)?;
    let node = get_current_node(tree, &session).map_err(engine_error_response)?;
    let is_complete = is_session_complete(tree, &session).map_err(engine_error_response)?;

    Ok((
        StatusCode::CREATED,
        Json(SessionResponse {
            state: state::encode(&session),
            tree_id: tree.id.clone(),
            tree_version: tree.version,
            is_complete,
            node: serde_json::to_value(node).expect("DecisionNode always serializes"),
        }),
    ))
}

/// Submit an answer to the current node, server-validated against
/// `core-domain` (never client-trusted) and return the next node.
#[utoipa::path(
    post,
    path = "/v1/sessions/answers",
    request_body = AnswerRequest,
    responses(
        (status = 200, description = "Next node (question or terminal)", body = SessionResponse),
        (status = 400, description = "Invalid state token or answer value", body = ApiError),
        (status = 409, description = "Session already reached a terminal node", body = ApiError),
    ),
    tag = "sessions"
)]
pub async fn submit_answer(
    Json(req): Json<AnswerRequest>,
) -> Result<Json<SessionResponse>, ApiErrorResponse> {
    let tree = vote_assist_tree_v1();
    let current = state::decode(&req.state).map_err(session_token_error_response)?;
    let next = answer(tree, &current, &req.value).map_err(engine_error_response)?;
    let node = get_current_node(tree, &next).map_err(engine_error_response)?;
    let is_complete = is_session_complete(tree, &next).map_err(engine_error_response)?;

    Ok(Json(SessionResponse {
        state: state::encode(&next),
        tree_id: tree.id.clone(),
        tree_version: tree.version,
        is_complete,
        node: serde_json::to_value(node).expect("DecisionNode always serializes"),
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SearchQuery {
    pub q: String,
}

/// Full-text (substring, MVP-level) search across the knowledge base.
///
/// `kb-content` deliberately has no `utoipa`/web-framework dependency (it
/// stays a pure domain crate reusable by non-HTTP channels too), so its
/// response body is represented generically here; the authoritative shape
/// is `kb_content::KnowledgeEntry`, exercised directly by that crate's own
/// test suite.
#[utoipa::path(
    get,
    path = "/v1/kb/search",
    params(("q" = String, Query, description = "Search query")),
    responses((status = 200, description = "Matching entries — array of kb_content::KnowledgeEntry")),
    tag = "knowledge-base"
)]
pub async fn search_kb(Query(q): Query<SearchQuery>) -> Json<Vec<kb_content::KnowledgeEntry>> {
    Json(search_entries(&q.q).into_iter().cloned().collect())
}

/// Fetch a single knowledge base entry by id.
#[utoipa::path(
    get,
    path = "/v1/kb/entries/{id}",
    params(("id" = String, Path, description = "Knowledge base entry id")),
    responses(
        (status = 200, description = "The entry — kb_content::KnowledgeEntry"),
        (status = 404, description = "Unknown entry id", body = ApiError),
    ),
    tag = "knowledge-base"
)]
pub async fn get_kb_entry(
    Path(id): Path<String>,
) -> Result<Json<kb_content::KnowledgeEntry>, ApiErrorResponse> {
    get_entry(&id).cloned().map(Json).ok_or_else(|| {
        ApiErrorResponse(
            StatusCode::NOT_FOUND,
            format!("unknown knowledge base entry id \"{id}\""),
        )
    })
}

/// Liveness/readiness probe for SRE tooling (load balancer health checks,
/// uptime monitoring — see docs/SECURITY-AND-SRE.md).
#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Service is healthy")))]
pub async fn healthz() -> &'static str {
    "ok"
}
