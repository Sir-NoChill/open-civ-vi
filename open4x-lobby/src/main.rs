//! Native binary for the open4x-lobby — Axum + ServeDir + auth surface.
//!
//! Boots an `AppState` (sqlite db + magic-link signer + LogMailer +
//! account store), wires the session-cookie middleware on every
//! request, serves the Trunk-built SPA + /health for liveness checks.
//! HTTP routes for email auth + /me come online incrementally as
//! Phase 3 progresses.

use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use open4x_lobby::server::{self, AppState};

#[tokio::main]
async fn main() {
    let static_dir = std::env::var("OPEN4X_LOBBY_STATIC_DIR")
        .unwrap_or_else(|_| "./open4x-lobby/dist".to_string());
    let data_dir = std::env::var("OPEN4X_LOBBY_DATA_DIR")
        .unwrap_or_else(|_| "./data/lobby".to_string());
    let book_dir = std::env::var("OPEN4X_LOBBY_BOOK_DIR")
        .unwrap_or_else(|_| "./book/book".to_string());

    let state = AppState::boot(&data_dir)
        .await
        .expect("AppState::boot failed — check db / key file permissions");

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api/v1", server::rest::v1_router())
        .nest_service("/book", ServeDir::new(&book_dir))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            server::auth::session_layer,
        ))
        .fallback_service(ServeDir::new(&static_dir))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("open4x-lobby listening on {addr}");
    println!("  static files: {static_dir}");
    println!("  data dir:     {data_dir}");
    println!("  book dir:     {book_dir} (run `mdbook build book/` to populate)");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
