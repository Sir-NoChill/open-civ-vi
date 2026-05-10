//! REST surface — `/api/v1/*`. Nested into the top-level router by
//! `main.rs` after the session-cookie middleware runs.

#![cfg(feature = "ssr")]

pub mod email_auth;

use axum::Router;
use axum::routing::{get, post};

use super::AppState;

pub fn v1_router() -> Router<AppState> {
    Router::new()
        .route("/auth/email/start", post(email_auth::start))
        .route("/auth/email/verify", get(email_auth::verify))
        .route("/auth/signout", post(email_auth::signout))
}
