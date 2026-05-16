//! Wire-protocol types for the open4x REST/WS API.
//!
//! Every public type that crosses the network boundary lives in [`v1`]. The
//! versioned namespace lets future protocol revisions land alongside the
//! current one without breaking existing clients.
//!
//! Phase 0 (scaffolding): empty. Phase 1 lifts `open4x-server/src/types/*`
//! into `v1` and the server re-exports from here.

#![allow(dead_code)]

pub mod v1;
