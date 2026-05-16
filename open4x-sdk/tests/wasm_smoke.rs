//! Smoke test for the wasm transport backend.
//!
//! Cfg-gated to `target_arch = "wasm32"` so a native `cargo test` skips it
//! entirely. Run under `wasm-pack test --headless --firefox` (or any other
//! browser harness) once that tooling is wired in.
//!
//! Scope: we exercise the parts of [`open4x_sdk::wasm::WasmClient`] that
//! don't require a real `fetch` round-trip — construction, builder methods,
//! and accessor invariants. A full request/response round-trip would need
//! a mock fetch impl or a real HTTP server reachable from the browser, both
//! of which require harness work outside this lane.

#![cfg(target_arch = "wasm32")]

use open4x_sdk::wasm::WasmClient;
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn constructs_with_base() {
    let client = WasmClient::new("http://localhost");
    assert_eq!(client.base(), "http://localhost");
    assert!(client.token().is_none());
}

#[wasm_bindgen_test]
fn builder_attaches_token() {
    let client = WasmClient::new("http://localhost").with_token("test");
    assert_eq!(client.base(), "http://localhost");
    assert_eq!(client.token(), Some("test"));
}

#[wasm_bindgen_test]
fn client_is_clone() {
    let client = WasmClient::new("http://example.com/").with_token("abc");
    let cloned = client.clone();
    assert_eq!(cloned.base(), "http://example.com/");
    assert_eq!(cloned.token(), Some("abc"));
}
