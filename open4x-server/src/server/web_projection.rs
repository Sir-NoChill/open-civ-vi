//! Builders that turn `GameState` / `GameView` into wire types under
//! [`crate::types::web`].
//!
//! Phase 0 (scaffolding): every builder returns a `Default` value. Real logic
//! lands in subsequent phases — see `book/src/roadmap/web-ui.md` §5.2.

#![allow(dead_code, unused_variables)]

use crate::types::view::GameView;
use crate::types::web::*;

pub fn build_player_state(view: &GameView) -> player_state::PlayerState {
    player_state::PlayerState::default()
}

pub fn build_world_snapshot(view: &GameView, q: i32, r: i32, radius: u32) -> world::WorldSnapshot {
    world::WorldSnapshot::default()
}

pub fn build_units(view: &GameView) -> unit_data::UnitData {
    unit_data::UnitData::default()
}

pub fn build_armies(view: &GameView) -> army_data::ArmyData {
    army_data::ArmyData::default()
}

pub fn build_cities(view: &GameView) -> city_data::CityData {
    city_data::CityData::default()
}

pub fn build_city_tiles(view: &GameView, city_id: &str) -> city_tiles::CityTiles {
    city_tiles::CityTiles::default()
}

pub fn build_tech_tree(view: &GameView) -> tech_tree::TechTreeView {
    tech_tree::TechTreeView::default()
}

pub fn build_civics_tree(view: &GameView) -> civics_tree::CivicsTreeView {
    civics_tree::CivicsTreeView::default()
}

pub fn build_government(view: &GameView) -> government::GovernmentPolicies {
    government::GovernmentPolicies::default()
}

pub fn build_diplomacy(view: &GameView) -> diplomacy::Diplomacy {
    diplomacy::Diplomacy::default()
}

pub fn build_empire_overview(view: &GameView) -> empire_overview::EmpireOverview {
    empire_overview::EmpireOverview::default()
}

pub fn build_victory(view: &GameView) -> victory::Victory {
    victory::Victory::default()
}

pub fn build_notifications(view: &GameView) -> notifications::Notifications {
    notifications::Notifications::default()
}

pub fn build_turn_queue(view: &GameView) -> turn_queue::TurnQueue {
    turn_queue::TurnQueue::default()
}

pub fn build_map_overlays(view: &GameView) -> map_overlays::MapOverlays {
    map_overlays::MapOverlays::default()
}
