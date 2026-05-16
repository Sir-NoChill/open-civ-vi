#![allow(dead_code)]

/// Shared wire-protocol types (compiles for both native and wasm32 targets).
pub mod types;

/// Server-only modules (Axum, game state, WebSocket, REST API).
#[cfg(feature = "ssr")]
pub mod server;
