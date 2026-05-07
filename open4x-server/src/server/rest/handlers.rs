//! REST handler functions for `/api/v1/*`.
//!
//! Phase 0: only `health` is implemented. Subsequent phases will fill in the
//! full surface from `book/src/roadmap/web-ui.md` §4.

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub api: &'static str,
}

/// `GET /api/v1/health` — unauthenticated liveness check.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        api: "v1",
    })
}
