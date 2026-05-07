//! Builders that turn `GameView` into wire types under
//! [`crate::types::web`].
//!
//! See `book/src/roadmap/web-ui.md` §5.2. Each builder is a pure function
//! over the player-projected `GameView` plus a small amount of room
//! configuration where needed (e.g. `turn_limit` for `/player-state`).
//!
//! Builders for endpoints that haven't been wired yet (Phase 2+) still
//! return `Default::default()`.

#![allow(dead_code, unused_variables)]

use crate::types::enums::{BoardTopology, BuiltinTerrain, TileVisibility};
use crate::types::view::GameView;
use crate::types::web::*;

// ── /player-state ────────────────────────────────────────────────────────────

pub fn build_player_state(view: &GameView, turn_limit: Option<u32>) -> player_state::PlayerState {
    let civ = &view.my_civ;
    let yields = &civ.yields;

    let resources = player_state::Resources {
        gold: player_state::Bucket {
            value: Some(civ.gold),
            per_turn: yields.gold,
        },
        science: player_state::Bucket {
            value: None,
            per_turn: yields.science,
        },
        culture: player_state::Bucket {
            value: None,
            per_turn: yields.culture,
        },
        faith: player_state::Bucket {
            value: Some(civ.faith as i32),
            per_turn: yields.faith,
        },
        food: player_state::Bucket {
            value: None,
            per_turn: yields.food,
        },
        production: player_state::Bucket {
            value: None,
            per_turn: yields.production,
        },
    };

    let strategic: std::collections::BTreeMap<String, u32> = civ
        .strategic_resources
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();

    player_state::PlayerState {
        turn: view.turn,
        turn_max: turn_limit.unwrap_or(0),
        era: format!("{:?}", civ.current_era),
        era_progress: 0.0, // TODO(WEBUI-ERA): plumb era progress through GameView
        resources,
        happiness: yields.amenities,
        strategic,
    }
}

// ── /world/snapshot ──────────────────────────────────────────────────────────

/// Project a `WorldSnapshot` from the player-visible `GameView`.
///
/// `radius == 0` returns every explored tile in the view; any other value
/// keeps only tiles within that hex distance from the camera centre `(q, r)`.
pub fn build_world_snapshot(
    view: &GameView,
    cam_q: i32,
    cam_r: i32,
    radius: u32,
) -> world::WorldSnapshot {
    let board = &view.board;

    let (wrap_x, wrap_y) = match board.topology {
        BoardTopology::Flat => (false, false),
        BoardTopology::CylindricalEW => (true, false),
        BoardTopology::Toroidal => (true, true),
    };

    let world_meta = world::WorldMeta {
        width: board.width,
        height: board.height,
        wrap_x,
        wrap_y,
        seed: 0, // TODO(WEBUI-SEED): plumb world seed through GameView
        turn: view.turn,
    };

    let camera = world::Camera {
        x: cam_q,
        y: cam_r,
        zoom: 1.0,
        selection: Some(world::TileCoord {
            q: cam_q,
            r: cam_r,
        }),
    };

    let legend = world::Legend {
        terrains: vec![
            "Grassland".into(),
            "Plains".into(),
            "Desert".into(),
            "Tundra".into(),
            "Snow".into(),
            "Coast".into(),
            "Ocean".into(),
            "Mountain".into(),
        ],
        resources: vec![
            "Wheat".into(),
            "Cattle".into(),
            "Iron".into(),
            "Horses".into(),
            "Fish".into(),
            "Pearls".into(),
        ],
        edge_kinds: vec!["river".into(), "border".into(), "coast".into()],
    };

    // Pre-compute tile lookups for cities/units placed on tiles.
    let cities_by_coord: std::collections::HashMap<_, _> = view
        .cities
        .iter()
        .map(|c| (c.coord, c))
        .collect();
    let units_by_coord: std::collections::HashMap<_, _> = view
        .units
        .iter()
        .map(|u| (u.coord, u))
        .collect();
    let unit_type_by_id: std::collections::HashMap<_, _> = view
        .unit_type_defs
        .iter()
        .map(|d| (d.id, d))
        .collect();
    let my_civ_id = view.my_civ_id;

    let tiles = board
        .tiles
        .iter()
        .filter(|t| {
            radius == 0
                || hex_distance(cam_q, cam_r, t.coord.q, t.coord.r) <= radius as i32
        })
        .map(|t| {
            let owner = t.owner.map(|id| {
                if id == my_civ_id {
                    view.my_civ.name.clone()
                } else {
                    view.other_civs
                        .iter()
                        .find(|c| c.id == id)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| id.to_string())
                }
            });

            let city = cities_by_coord.get(&t.coord).map(|c| world::TileCity {
                id: c.id.to_string(),
                name: c.name.clone(),
                pop: c.population,
                capital: c.is_capital,
            });

            let unit = units_by_coord.get(&t.coord).map(|u| {
                let kind = unit_type_by_id
                    .get(&u.unit_type)
                    .map(|d| capitalize(&d.name))
                    .unwrap_or_else(|| "Unit".into());
                world::TileUnit {
                    id: u.id.to_string(),
                    kind,
                    name: format!("{:?}", u.category),
                    hp: format!("{}/{}", u.health, 100),
                }
            });

            world::TileView {
                q: t.coord.q,
                r: t.coord.r,
                terrain: terrain_label(t.terrain, t.hills),
                yields: world::TileYields::default(),
                appeal: 0,
                flood: 0,
                fog: matches!(t.visibility, TileVisibility::Foggy),
                owner,
                resource: t.resource.map(|r| format!("{r:?}")),
                improvement: t.improvement.map(|i| format!("{i:?}")),
                city,
                unit,
            }
        })
        .collect();

    world::WorldSnapshot {
        world: world_meta,
        camera,
        legend,
        tiles,
    }
}

fn terrain_label(t: BuiltinTerrain, hills: bool) -> String {
    if hills {
        format!("{t:?}+Hills")
    } else {
        format!("{t:?}")
    }
}

/// Cube-coordinate hex distance using only the axial `(q, r)` pair.
fn hex_distance(a_q: i32, a_r: i32, b_q: i32, b_r: i32) -> i32 {
    let dq = a_q - b_q;
    let dr = a_r - b_r;
    let ds = -dq - dr;
    (dq.abs() + dr.abs() + ds.abs()) / 2
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// ── Phase 2+ stubs ───────────────────────────────────────────────────────────

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

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::coord::HexCoord;
    use crate::types::ids::CivId;
    use crate::types::view::*;

    fn empty_game_view() -> GameView {
        let civ_id = CivId::from_ulid(ulid::Ulid::new());
        GameView {
            turn: 7,
            my_civ_id: civ_id,
            board: BoardView {
                width: 10,
                height: 10,
                topology: BoardTopology::CylindricalEW,
                tiles: Vec::new(),
                river_edges: Vec::new(),
            },
            my_civ: CivView {
                id: civ_id,
                name: "Egypt".into(),
                adjective: "Egyptian".into(),
                leader_name: "Cleopatra".into(),
                gold: 100,
                current_era: crate::types::enums::AgeType::Classical,
                researched_techs: Vec::new(),
                research_queue: Vec::new(),
                completed_civics: Vec::new(),
                civic_in_progress: None,
                current_government: None,
                active_policies: Vec::new(),
                unlocked_units: Vec::new(),
                unlocked_buildings: Vec::new(),
                unlocked_improvements: Vec::new(),
                strategic_resources: [("Iron".into(), 3u32), ("Horses".into(), 2u32)]
                    .into_iter()
                    .collect(),
                yields: YieldBundleView {
                    food: 1,
                    production: 2,
                    gold: 5,
                    science: 7,
                    culture: 4,
                    faith: 1,
                    housing: 0,
                    amenities: 2,
                    tourism: 0,
                    great_person_points: 0,
                },
                faith: 11,
                pantheon_belief: None,
                founded_religion: None,
            },
            other_civs: Vec::new(),
            cities: Vec::new(),
            units: Vec::new(),
            tech_tree: TechTreeView { nodes: Vec::new() },
            civic_tree: CivicTreeView { nodes: Vec::new() },
            trade_routes: Vec::new(),
            unit_type_defs: Vec::new(),
            building_defs: Vec::new(),
            scores: Vec::new(),
            religions: Vec::new(),
            game_over: None,
        }
    }

    #[test]
    fn player_state_projects_yields_and_strategic_resources() {
        let view = empty_game_view();
        let ps = build_player_state(&view, Some(500));

        assert_eq!(ps.turn, 7);
        assert_eq!(ps.turn_max, 500);
        assert_eq!(ps.era, "Classical");
        assert_eq!(ps.resources.gold.value, Some(100));
        assert_eq!(ps.resources.gold.per_turn, 5);
        assert!(ps.resources.science.value.is_none());
        assert_eq!(ps.resources.science.per_turn, 7);
        assert_eq!(ps.resources.faith.value, Some(11));
        assert_eq!(ps.happiness, 2);
        assert_eq!(ps.strategic.get("Iron").copied(), Some(3));
        assert_eq!(ps.strategic.get("Horses").copied(), Some(2));
    }

    #[test]
    fn world_snapshot_topology_maps_to_wrap_flags() {
        let view = empty_game_view();
        let snap = build_world_snapshot(&view, 0, 0, 0);
        assert_eq!(snap.world.width, 10);
        assert_eq!(snap.world.height, 10);
        assert!(snap.world.wrap_x);
        assert!(!snap.world.wrap_y);
        assert_eq!(snap.world.turn, 7);
    }

    #[test]
    fn world_snapshot_radius_filters_tiles() {
        let mut view = empty_game_view();
        for q in -3..=3 {
            for r in -3..=3 {
                view.board.tiles.push(TileView {
                    coord: HexCoord::from_qr(q, r),
                    terrain: crate::types::enums::BuiltinTerrain::Plains,
                    hills: false,
                    feature: None,
                    resource: None,
                    improvement: None,
                    road: None,
                    owner: None,
                    visibility: TileVisibility::Visible,
                });
            }
        }
        // Radius 0 means "no filter": keep all tiles.
        assert_eq!(build_world_snapshot(&view, 0, 0, 0).tiles.len(), 49);
        // Radius 1 = centre + 6 neighbours = 7 tiles.
        assert_eq!(build_world_snapshot(&view, 0, 0, 1).tiles.len(), 7);
        // Radius 2 = 1 + 6 + 12 = 19 tiles.
        assert_eq!(build_world_snapshot(&view, 0, 0, 2).tiles.len(), 19);
    }
}
