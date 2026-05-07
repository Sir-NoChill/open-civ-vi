//! REST handlers for the `/api/v1/*` surface defined in
//! `book/src/roadmap/web-ui.md` §4.
//!
//! Phase 0 (scaffolding): only `/api/v1/health` is wired. The remaining
//! handlers are added phase by phase.
//!
//! All endpoints (besides `/health`) require `Authorization: Bearer <token>`
//! resolved through [`crate::server::api_token`].

pub mod auth;
pub mod handlers;

pub use handlers::*;
