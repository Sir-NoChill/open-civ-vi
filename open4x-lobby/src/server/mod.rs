//! Server-only Axum surface for the lobby. ssr feature only.
//!
//! Phase 3 of `book/src/roadmap/accounts-and-login.md`. Owns the
//! shared `AppState`, session-cookie middleware, magic-link auth
//! route pair, and `/api/v1/me` profile reads/writes.

#![cfg(feature = "ssr")]

pub mod auth;
pub mod client_ip;
pub mod embed;
pub mod orchestrator;
pub mod rest;
pub mod state;

pub use auth::{AuthCookie, RequireSession, session_layer, SESSION_COOKIE_NAME};
pub use state::AppState;
