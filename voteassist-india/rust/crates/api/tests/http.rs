//! End-to-end HTTP tests against the real router (not a mock) — exercises
//! the exact same `Router` the production binary serves.

use api::build_router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn healthz_returns_ok() {
    let response = build_router()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn full_nri_flow_start_to_terminal_over_http() {
    let router = build_router();

    // 1. Start a session.
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = body_json(response).await;
    assert_eq!(body["node"]["id"], "start");
    assert_eq!(body["isComplete"], false);
    let mut state_token = body["state"].as_str().unwrap().to_string();

    // 2. Answer "no" (not registered).
    for value in ["no", "adult", "nri"] {
        let req = Request::builder()
            .method("POST")
            .uri("/v1/sessions/answers")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&serde_json::json!({ "state": state_token, "value": value }))
                    .unwrap(),
            ))
            .unwrap();
        let response = router.clone().oneshot(req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "answering {value} should succeed"
        );
        let body = body_json(response).await;
        state_token = body["state"].as_str().unwrap().to_string();

        if value == "nri" {
            assert_eq!(body["isComplete"], true);
            assert_eq!(body["node"]["id"], "terminal_form6a");
            assert_eq!(body["node"]["citations"][0]["knowledgeBaseId"], "form-6a");
            assert!(body["node"]["deepLinks"][0]["url"]
                .as_str()
                .unwrap()
                .starts_with("https://"));
        }
    }
}

#[tokio::test]
async fn invalid_answer_value_returns_400() {
    let router = build_router();
    let start = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/sessions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_json(start).await;
    let state_token = body["state"].as_str().unwrap().to_string();

    let req = Request::builder()
        .method("POST")
        .uri("/v1/sessions/answers")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(
                &serde_json::json!({ "state": state_token, "value": "not-a-real-option" }),
            )
            .unwrap(),
        ))
        .unwrap();
    let response = router.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn garbage_state_token_returns_400_not_500() {
    let req = Request::builder()
        .method("POST")
        .uri("/v1/sessions/answers")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_vec(&serde_json::json!({ "state": "garbage", "value": "x" })).unwrap(),
        ))
        .unwrap();
    let response = build_router().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn kb_entry_lookup_and_404() {
    let router = build_router();

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/kb/entries/form-8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["id"], "form-8");

    let response = router
        .oneshot(
            Request::builder()
                .uri("/v1/kb/entries/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn kb_search_finds_form_8_for_shifting_of_residence() {
    let response = build_router()
        .oneshot(
            Request::builder()
                .uri("/v1/kb/search?q=shifting%20of%20residence")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    let ids: Vec<&str> = body
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"form-8"));
}

#[tokio::test]
async fn openapi_doc_is_served() {
    let response = build_router()
        .oneshot(
            Request::builder()
                .uri("/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert!(body["paths"]["/v1/sessions"].is_object());
}
