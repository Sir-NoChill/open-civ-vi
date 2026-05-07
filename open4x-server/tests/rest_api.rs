//! End-to-end integration tests for the `/api/v1/*` REST surface.
//!
//! These exercise the same `Router` `main.rs` mounts but skip the TCP
//! listener — every request goes through `tower::ServiceExt::oneshot`. That
//! means the tests are deterministic, parallel-safe, and add no port-binding
//! dance to CI.
//!
//! Each test follows the same shape:
//!   1. build a fresh `AppState` and the v1 `Router`
//!   2. POST `/api/v1/games/new` to get a bearer token
//!   3. exercise the endpoint(s) under test, asserting status + key fields
//!
//! Plan: book/src/roadmap/web-ui.md §4.

#![cfg(feature = "ssr")]

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt as _;
use serde_json::Value;
use tower::ServiceExt;

use open4x_server::server::rest::v1_router;
use open4x_server::server::state::AppState;

// ── test rig ────────────────────────────────────────────────────────────────

fn build_app() -> (Router, Arc<AppState>) {
    let state = AppState::new();
    let router = Router::new()
        .nest("/api/v1", v1_router())
        .with_state(state.clone());
    (router, state)
}

async fn json_body(resp: axum::response::Response) -> (StatusCode, Value) {
    let status = resp.status();
    let body = resp.into_body();
    let collected = body.collect().await.expect("collect body");
    let bytes = collected.to_bytes();
    let value: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("response was not json")
    };
    (status, value)
}

async fn get_with(app: &Router, path: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    json_body(resp).await
}

async fn post_with(
    app: &Router,
    path: &str,
    token: Option<&str>,
    body: Value,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let resp = app
        .clone()
        .oneshot(req.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap();
    json_body(resp).await
}

async fn delete_with(app: &Router, path: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(path)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    json_body(resp).await
}

/// Bootstrap a single-player session and return the bearer token.
async fn bootstrap_token(app: &Router) -> String {
    let body = serde_json::json!({
        "width": 12,
        "height": 8,
        "seed": 7,
        "num_ai": 0,
        "turn_limit": 50,
    });
    let (status, body) = post_with(app, "/api/v1/games/new", None, body).await;
    assert_eq!(status, StatusCode::CREATED, "games/new: {body:?}");
    body["token"].as_str().expect("token in response").to_string()
}

// ── tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn health_is_unauthenticated() {
    let (app, _state) = build_app();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, body) = json_body(resp).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
    assert_eq!(body["api"], "v1");
}

#[tokio::test]
async fn unauthenticated_endpoint_rejects_missing_token() {
    let (app, _state) = build_app();
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/player-state")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let (status, _) = json_body(resp).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn games_new_mints_token_and_player_state_uses_it() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, ps) = get_with(&app, "/api/v1/player-state", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(ps["turn"], 0);
    assert_eq!(ps["turn_max"], 50);
    assert_eq!(ps["era"], "Ancient");
    assert!(ps["resources"]["gold"].is_object());
}

#[tokio::test]
async fn world_snapshot_returns_tiles_and_dimensions() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, snap) = get_with(&app, "/api/v1/world/snapshot?radius=4", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(snap["world"]["width"], 12);
    assert_eq!(snap["world"]["height"], 8);
    let tiles = snap["tiles"].as_array().unwrap();
    assert!(!tiles.is_empty(), "expected at least one tile in view");
}

#[tokio::test]
async fn cities_units_and_diplomacy_round_trip() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, cities) = get_with(&app, "/api/v1/cities", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(cities["cities"].as_array().unwrap().len() >= 1);

    let (status, units) = get_with(&app, "/api/v1/units", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(units["units"].as_array().unwrap().len() >= 1);

    let (status, dip) = get_with(&app, "/api/v1/diplomacy", &token).await;
    assert_eq!(status, StatusCode::OK);
    // Single-player solo game: 0 other civs.
    assert_eq!(dip["civs"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn end_turn_blocks_when_required_action_pending() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    // Fresh game: research queue empty -> 'choose_research' is required.
    let (status, body) = post_with(&app, "/api/v1/turn/end", Some(&token), serde_json::json!({})).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "turn/end body: {body:?}");
    assert_eq!(body["error"], "unresolved_required_actions");
    let items = body["items"].as_array().expect("items array");
    assert!(items.iter().any(|it| it["id"] == "choose_research"));
}

#[tokio::test]
async fn end_turn_advances_after_research_chosen() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, tt) = get_with(&app, "/api/v1/tech", &token).await;
    assert_eq!(status, StatusCode::OK);
    let tech_id = tt["techs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["status"] == "available")
        .and_then(|t| t["id"].as_str())
        .expect("at least one available tech")
        .to_string();

    let (status, _) = post_with(
        &app,
        "/api/v1/tech/research",
        Some(&token),
        serde_json::json!({"tech_id": tech_id}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = post_with(&app, "/api/v1/turn/end", Some(&token), serde_json::json!({})).await;
    assert_eq!(status, StatusCode::OK, "after research: {body:?}");
    assert_eq!(body["ok"], true);
    assert_eq!(body["view"]["turn"], 1);
}

#[tokio::test]
async fn production_queue_and_cancel_round_trip() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (_, cities) = get_with(&app, "/api/v1/cities", &token).await;
    let city_id = cities["cities"][0]["id"].as_str().unwrap().to_string();

    // Pull a unit-type id from /api/v1/registry.
    let (_, reg) = get_with(&app, "/api/v1/registry", &token).await;
    let warrior_type = reg["unit_types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["name"] == "warrior")
        .and_then(|d| d["id"].as_str())
        .expect("warrior unit type")
        .to_string();

    let (status, body) = post_with(
        &app,
        &format!("/api/v1/cities/{city_id}/production"),
        Some(&token),
        serde_json::json!({"item_id": warrior_type, "item_type": "unit"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "queue_production: {body:?}");
    assert_eq!(body["view"]["production_queue"][0], "Warrior");

    let (status, body) = delete_with(
        &app,
        &format!("/api/v1/cities/{city_id}/production/0"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "cancel_production: {body:?}");
    assert!(body["view"]["production_queue"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn turn_queue_lists_required_choose_research_on_fresh_game() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, q) = get_with(&app, "/api/v1/turn-queue", &token).await;
    assert_eq!(status, StatusCode::OK);
    let items = q["items"].as_array().unwrap();
    assert!(items
        .iter()
        .any(|it| it["id"] == "choose_research" && it["required"] == true));
}

#[tokio::test]
async fn victory_includes_six_conditions_and_ranks_player() {
    let (app, _state) = build_app();
    let token = bootstrap_token(&app).await;

    let (status, v) = get_with(&app, "/api/v1/victory", &token).await;
    assert_eq!(status, StatusCode::OK);
    let conditions = v["conditions"].as_array().unwrap();
    assert_eq!(conditions.len(), 6);
    let names: Vec<&str> = conditions
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"score"));
    assert!(names.contains(&"culture"));
    assert!(names.contains(&"science"));
    let leaderboard = v["leaderboard"].as_array().unwrap();
    assert!(leaderboard.iter().any(|r| r["is_player"] == true));
}
