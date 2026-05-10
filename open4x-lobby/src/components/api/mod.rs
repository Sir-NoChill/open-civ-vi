//! Client-side bindings for the lobby's `/api/v1/*` REST surface.
//!
//! Mirrors the shape of `open4x-server/src/components/api/`:
//! `http::fetch_json` is the single transport helper; each module
//! under here wraps one logical endpoint group. No JS shim — pure
//! `web_sys::Fetch` + `wasm-bindgen`.

pub mod auth;
pub mod friends;
pub mod games;
pub mod http;
pub mod me;
pub mod presets;

pub use http::{ApiError, fetch_json};
