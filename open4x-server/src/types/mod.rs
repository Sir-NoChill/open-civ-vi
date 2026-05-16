//! Re-export shim for the wire protocol. Phase 1 of the crate-split
//! migration. Removed in Phase 6 once all internal `use crate::types::*`
//! call sites are migrated to `use open4x_protocol::v1::*` directly.
//!
//! See `book/src/roadmap/crate-split.md`.

pub use open4x_protocol::v1::*;

// Keep submodule paths available for `use crate::types::messages::Foo`
// style imports until Phase 6 rewrites them.
pub use open4x_protocol::v1::{
    coord, enums, ids, messages, profile, reports, view, web,
};
