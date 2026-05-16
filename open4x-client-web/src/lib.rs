//! Leptos/WASM web client for open4x.
//!
//! Mounts to a static `index.html` via [`start`], talks to the server
//! exclusively over [`open4x_sdk`] (REST today, WS for `pages/game.rs`)
//! using the wire shapes in [`open4x_protocol::v1`].
//!
//! The crate is wasm-only at runtime — every UI module pulls in
//! `leptos` and `web-sys`, which are gated to `cfg(target_arch =
//! "wasm32")` in `Cargo.toml`. We mirror that gate at the source level
//! so `cargo build --workspace` on a native host still type-checks the
//! crate as an empty cdylib (with just the `start` shim cfg-removed).

#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]
pub mod components;
#[cfg(target_arch = "wasm32")]
pub mod pages;
#[cfg(target_arch = "wasm32")]
pub mod tabs;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(crate::pages::RestGamePage);
}
