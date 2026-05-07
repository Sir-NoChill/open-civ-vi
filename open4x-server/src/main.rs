#![allow(dead_code)]

use axum::Router;
use axum::routing::{delete, get, post};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use open4x_server::server;

#[tokio::main]
async fn main() {
    let state = server::state::AppState::new();

    // Static file directory for the trunk-built frontend.
    let static_dir = std::env::var("OPEN4X_STATIC_DIR")
        .unwrap_or_else(|_| "./open4x-server/dist".to_string());

    let app = Router::new()
        .route("/ws", get(server::ws::ws_handler))
        .route("/health", get(|| async { "ok" }))
        .route("/api/demo-game", get(demo_game_handler))
        // REST v1 (under construction — see book/src/roadmap/web-ui.md)
        .route("/api/v1/health", get(server::rest::handlers::health))
        .route("/api/v1/games/new", post(server::rest::handlers::new_game))
        .route("/api/v1/player-state", get(server::rest::handlers::player_state))
        .route("/api/v1/world/snapshot", get(server::rest::handlers::world_snapshot))
        .route("/api/v1/world/tile/{q}/{r}", get(server::rest::handlers::world_tile))
        .route("/api/v1/map/overlays", get(server::rest::handlers::map_overlays))
        .route("/api/v1/cities", get(server::rest::handlers::cities))
        .route("/api/v1/cities/{id}", get(server::rest::handlers::city_detail))
        .route("/api/v1/cities/{id}/tiles", get(server::rest::handlers::city_tiles))
        .route("/api/v1/units", get(server::rest::handlers::units))
        .route("/api/v1/units/{id}", get(server::rest::handlers::unit_detail))
        .route("/api/v1/armies", get(server::rest::handlers::armies))
        .route("/api/v1/combat/preview", get(server::rest::handlers::combat_preview))
        .route("/api/v1/cities/{id}/production", post(server::rest::handlers::queue_production))
        .route("/api/v1/cities/{id}/production/{pos}", delete(server::rest::handlers::cancel_production))
        .route("/api/v1/units/{id}/action", post(server::rest::handlers::unit_action))
        .route("/api/v1/turn/end", post(server::rest::handlers::end_turn))
        // Legacy /api/game/* (deprecated, removed in Phase 5)
        .route("/api/game/view", get(server::api::game_view))
        .route("/api/game/cities", get(server::api::cities))
        .route("/api/game/city/{id}", get(server::api::city_detail))
        .route("/api/game/resources", get(server::api::resources))
        .route("/api/game/units", get(server::api::units))
        .route("/api/game/map-stats", get(server::api::map_stats))
        .route("/api/game/players", get(server::api::players))
        .route("/api/game/science", get(server::api::science))
        .route("/api/game/culture", get(server::api::culture))
        .route("/api/game/turn", get(server::api::turn_status))
        .fallback_service(ServeDir::new(&static_dir))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("open4x-server listening on {addr}");
    println!("  static files: {static_dir}");
    println!("  data dir:     {}", std::env::var("OPEN4X_DATA_DIR").unwrap_or_else(|_| "./data".into()));
    let listener = tokio::net::TcpListener::bind(&addr).await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}

/// GET /api/demo-game?seed=42&width=20&height=14&turns=100
async fn demo_game_handler(
    params: axum::extract::Query<DemoParams>,
) -> axum::Json<server::demo::DemoGameResult> {
    let result = server::demo::run_demo_game(
        params.seed.unwrap_or(42),
        params.width.unwrap_or(20),
        params.height.unwrap_or(14),
        params.turns.unwrap_or(100),
        params.players.unwrap_or(2),
    );
    axum::Json(result)
}

#[derive(serde::Deserialize)]
struct DemoParams {
    seed: Option<u64>,
    width: Option<u32>,
    height: Option<u32>,
    turns: Option<u32>,
    players: Option<u32>,
}
