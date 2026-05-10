//! HUD components for the REST-driven single-player client.
//!
//! These mirror the layout of the legacy wireframe in
//! `docs/legacy-wireframe/4X Wireframes.html` but consume the typed wire
//! shapes from [`crate::types::web`] via [`crate::components::api`]. They are
//! independent of the WS-driven `pages/game.rs` flow.

pub mod snapshot_map;
