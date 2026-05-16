//! Typed client SDK for the open4x REST/WS API.
//!
//! Two transports share one set of typed endpoint functions:
//!
//! - [`native`] — `reqwest`-backed, both blocking and async. Used by
//!   `open4x-cli` remote mode and by server-side integration tests.
//! - [`wasm`]   — `web_sys::fetch`-backed. Used by `open4x-client-web`.
//!
//! Phase 0 (scaffolding): every module is an empty stub. Phase 2 fills
//! [`native`] and [`wasm`] and the per-resource bodies under
//! [`endpoints`].

#![allow(dead_code)]

pub mod endpoints;
pub mod error;

#[cfg(any(feature = "native-blocking", feature = "native-async"))]
pub mod native;

#[cfg(feature = "wasm")]
pub mod wasm;
