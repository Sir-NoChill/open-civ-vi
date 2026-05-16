//! Protocol v1 namespace.
//!
//! Lifted from `open4x-server/src/types/` in Phase 1 of the crate-split
//! migration (see `book/src/roadmap/crate-split.md`). The server keeps a
//! re-export shim at `open4x-server/src/types/mod.rs` so existing
//! `use crate::types::*` call sites continue to work until Phase 6.

pub mod coord;
pub mod enums;
pub mod ids;
pub mod messages;
pub mod profile;
pub mod reports;
pub mod view;
pub mod web;

// Re-export key types at the v1 root for convenience.
pub use coord::{HexCoord, HexDir};
pub use enums::*;
pub use ids::*;
pub use messages::{ClientMessage, GameAction, GameStatus, ServerMessage};
pub use profile::{CivTemplate, ProfileView};
pub use reports::*;
pub use view::GameView;
