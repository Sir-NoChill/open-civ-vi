//! Typed endpoint functions, one module per REST resource.
//!
//! Hot-zone rule (see `book/src/roadmap/crate-split.md` §11.3): Phase 0
//! pre-creates every module declaration in this file so Phase 2a and 2b can
//! land in parallel without anyone editing `mod.rs`. Each phase fills the
//! per-resource module body.

pub mod armies;
pub mod cities;
pub mod civics;
pub mod combat;
pub mod diplomacy;
pub mod empire;
pub mod games;
pub mod government;
pub mod health;
pub mod map;
pub mod notifications;
pub mod player_state;
pub mod registry;
pub mod tech;
pub mod turn;
pub mod units;
pub mod victory;
pub mod world;
