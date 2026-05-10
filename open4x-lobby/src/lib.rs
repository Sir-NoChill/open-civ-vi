//! Open4X-VI lobby/launcher — paper-themed Leptos SPA holding the pre-game
//! surface (landing → login → menu → new-game wizard → profile).
//!
//! Visual reference: `docs/open4x-landing/project/hifi/`. The full design
//! brief (rationale for splitting auth + lobby off from the in-game server,
//! the planned `open4x-accounts` substrate) lives in
//! `docs/open4x-landing/README.md`.
//!
//! The ssr/csr feature gates mirror `open4x-server`. Build the SPA with
//! `trunk build --features csr --no-default-features` and the native binary
//! with `cargo run -p open4x-lobby` (default `ssr`).

#![allow(dead_code)]

#[cfg(feature = "csr")]
pub mod app;
#[cfg(feature = "csr")]
pub mod components;
#[cfg(feature = "csr")]
pub mod screens;

#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(crate::app::App);
}
