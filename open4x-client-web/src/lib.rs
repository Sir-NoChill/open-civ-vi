//! Leptos/WASM web client for open4x.
//!
//! Phase 0 (scaffolding): empty `wasm_bindgen(start)` so the cdylib has an
//! entry point. Phase 4 moves `open4x-server/src/{pages,components,tabs}`
//! into this crate and rewires them onto `open4x-sdk` + `open4x-protocol`.

#![allow(dead_code)]

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}
