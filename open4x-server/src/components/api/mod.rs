//! Client-side bindings for the `/api/v1/*` REST surface.
//!
//! One module per endpoint group. Reads return `Result<T, ApiError>`; writes
//! return `Result<MutationResponse<T>, ApiError>`. All transport goes through
//! [`http::fetch_json`] — there is no JS shim.
//!
//! Phase 0 (scaffolding): only [`health`] and a stub [`player_state::get`] are
//! implemented; subsequent phases fill in the rest.

pub mod cities;
pub mod games;
pub mod government;
pub mod http;
pub mod player_state;
pub mod tech;
pub mod turn;
pub mod units;
pub mod world;

pub use http::{ApiError, fetch_json};
