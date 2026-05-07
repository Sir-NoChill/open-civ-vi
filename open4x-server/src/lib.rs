#![allow(dead_code)]

/// Shared wire-protocol types (compiles for both native and wasm32 targets).
pub mod types;

/// Server-only modules (Axum, game state, WebSocket, REST API).
#[cfg(feature = "ssr")]
pub mod server;

/// Client-only frontend components (Leptos CSR).
#[cfg(feature = "csr")]
pub mod components;

/// Client-only page components.
#[cfg(feature = "csr")]
pub mod pages;

/// Client-only tab components for the game interface.
#[cfg(feature = "csr")]
pub mod tabs;

/// WASM entry point: mounts the Leptos `RestGamePage` to `<body>`.
///
/// Trunk auto-detects this `#[wasm_bindgen(start)]` function on the
/// `cdylib` and invokes it after the bundle loads.
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(crate::pages::RestGamePage);
}
