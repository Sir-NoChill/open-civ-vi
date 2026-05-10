//! Native binary for the open4x-lobby — Axum + ServeDir, no game logic.
//!
//! Serves the Trunk-built SPA (`open4x-lobby/dist/`) and (eventually) the
//! account / lobby HTTP surface. Today the only endpoint is `/health`; the
//! rest of the requests are static asset lookups against the SPA.

use axum::Router;
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let static_dir = std::env::var("OPEN4X_LOBBY_STATIC_DIR")
        .unwrap_or_else(|_| "./open4x-lobby/dist".to_string());

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .fallback_service(ServeDir::new(&static_dir))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".to_string());
    let addr = format!("0.0.0.0:{port}");
    println!("open4x-lobby listening on {addr}");
    println!("  static files: {static_dir}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
