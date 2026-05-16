//! Wire-schema snapshot tests.
//!
//! These pin the JSON shape of representative top-level protocol types so
//! accidental field renames or shape drift trip CI. Per the crate-split
//! risk register (`book/src/roadmap/crate-split.md` §7), the snapshots
//! compare deserialized `serde_json::Value`s — not strings — so whitespace
//! and field-order are tolerated while structural drift still fails.
//!
//! When a schema change is intentional, update the inline expected JSON
//! literal to match.

use open4x_protocol::v1;
use open4x_protocol::v1::ids::CivId;
use serde::Serialize;
use serde_json::{Value, json};
use ulid::Ulid;

/// Serialize `value`, parse the inline `expected` JSON string, and compare
/// them as `serde_json::Value` so whitespace + field-order don't matter.
fn assert_wire_schema<T: Serialize>(value: &T, expected: &str) {
    let actual: Value = serde_json::to_value(value).expect("serialize");
    let expected: Value = serde_json::from_str(expected).expect("parse expected JSON");
    assert_eq!(actual, expected);
}

#[test]
fn snapshot_game_view() {
    // GameView has no Default — construct minimally with nil ULIDs so the
    // serialized form is deterministic.
    let nil_civ = CivId::from_ulid(Ulid::nil());
    let view = v1::view::GameView {
        turn: 0,
        my_civ_id: nil_civ,
        board: v1::view::BoardView {
            width: 0,
            height: 0,
            topology: v1::enums::BoardTopology::Flat,
            tiles: vec![],
            river_edges: vec![],
        },
        my_civ: v1::view::CivView {
            id: nil_civ,
            name: String::new(),
            adjective: String::new(),
            leader_name: String::new(),
            gold: 0,
            current_era: v1::enums::AgeType::Ancient,
            researched_techs: vec![],
            research_queue: vec![],
            completed_civics: vec![],
            civic_in_progress: None,
            current_government: None,
            active_policies: vec![],
            unlocked_units: vec![],
            unlocked_buildings: vec![],
            unlocked_improvements: vec![],
            strategic_resources: Default::default(),
            yields: Default::default(),
            faith: 0,
            pantheon_belief: None,
            founded_religion: None,
        },
        other_civs: vec![],
        cities: vec![],
        units: vec![],
        tech_tree: v1::view::TechTreeView { nodes: vec![] },
        civic_tree: v1::view::CivicTreeView { nodes: vec![] },
        trade_routes: vec![],
        unit_type_defs: vec![],
        building_defs: vec![],
        scores: vec![],
        religions: vec![],
        game_over: None,
    };

    let expected = r#"{
        "turn": 0,
        "my_civ_id": "00000000000000000000000000",
        "board": {
            "width": 0,
            "height": 0,
            "topology": "Flat",
            "tiles": [],
            "river_edges": []
        },
        "my_civ": {
            "id": "00000000000000000000000000",
            "name": "",
            "adjective": "",
            "leader_name": "",
            "gold": 0,
            "current_era": "Ancient",
            "researched_techs": [],
            "research_queue": [],
            "completed_civics": [],
            "civic_in_progress": null,
            "current_government": null,
            "active_policies": [],
            "unlocked_units": [],
            "unlocked_buildings": [],
            "unlocked_improvements": [],
            "strategic_resources": {},
            "yields": {
                "food": 0,
                "production": 0,
                "gold": 0,
                "science": 0,
                "culture": 0,
                "faith": 0,
                "housing": 0,
                "amenities": 0,
                "tourism": 0,
                "great_person_points": 0
            },
            "faith": 0,
            "pantheon_belief": null,
            "founded_religion": null
        },
        "other_civs": [],
        "cities": [],
        "units": [],
        "tech_tree": {"nodes": []},
        "civic_tree": {"nodes": []},
        "trade_routes": [],
        "unit_type_defs": [],
        "building_defs": [],
        "scores": [],
        "religions": [],
        "game_over": null
    }"#;

    assert_wire_schema(&view, expected);
}

#[test]
fn snapshot_player_state() {
    let state = v1::web::player_state::PlayerState::default();
    let expected = r#"{
        "turn": 0,
        "turn_max": 0,
        "era": "",
        "era_progress": 0.0,
        "resources": {
            "gold":       {"value": null, "per_turn": 0},
            "science":    {"value": null, "per_turn": 0},
            "culture":    {"value": null, "per_turn": 0},
            "faith":      {"value": null, "per_turn": 0},
            "food":       {"value": null, "per_turn": 0},
            "production": {"value": null, "per_turn": 0}
        },
        "happiness": 0,
        "strategic": {}
    }"#;

    assert_wire_schema(&state, expected);
}

#[test]
fn snapshot_world_snapshot() {
    let snap = v1::web::world::WorldSnapshot::default();
    let expected = r#"{
        "world":  {
            "width": 0,
            "height": 0,
            "wrap_x": false,
            "wrap_y": false,
            "seed": 0,
            "turn": 0
        },
        "camera": {
            "x": 0,
            "y": 0,
            "zoom": 0.0,
            "selection": null
        },
        "legend": {
            "terrains": [],
            "resources": [],
            "edge_kinds": []
        },
        "tiles": []
    }"#;

    assert_wire_schema(&snap, expected);
}

#[test]
fn snapshot_mutation_response_unit() {
    // MutationResponse<T> has no Default (T might not). Use `()` as the
    // payload type — it serializes to `null` — to cover the envelope shape.
    let resp: v1::web::MutationResponse<()> = v1::web::MutationResponse {
        ok: true,
        view: (),
        turn_status: v1::web::TurnStatusBlock::default(),
    };

    let expected = r#"{
        "ok": true,
        "view": null,
        "turn_status": {"turn": 0, "ended": false}
    }"#;

    assert_wire_schema(&resp, expected);
}

#[test]
fn snapshot_api_error_body() {
    // ApiErrorBody::default(). `message` is `#[serde(skip_serializing_if = "Option::is_none")]`
    // so the default form omits it entirely.
    let err = v1::web::ApiErrorBody::default();
    let expected = r#"{"error": ""}"#;
    assert_wire_schema(&err, expected);

    // Populated form keeps `message` in the wire output.
    let err = v1::web::ApiErrorBody {
        error: "bad_request".into(),
        message: Some("expected JSON".into()),
    };
    let expected = r#"{"error": "bad_request", "message": "expected JSON"}"#;
    assert_wire_schema(&err, expected);
}

#[test]
fn json_value_comparison_is_field_order_insensitive() {
    // Sanity: prove the helper itself doesn't depend on field order in the
    // expected literal. If this ever fails, every other test in this file
    // becomes unreliable.
    let v: Value = json!({"a": 1, "b": 2});
    let reversed: Value = serde_json::from_str(r#"{"b": 2, "a": 1}"#).unwrap();
    assert_eq!(v, reversed);
}

// A representative roundtrip doctest-style assertion at the integration-test
// level: every snapshotted type must also deserialize back from the JSON we
// expect to produce. This catches "we serialize fine but can't read our own
// output" bugs.
#[test]
fn roundtrip_default_player_state() {
    let original = v1::web::player_state::PlayerState::default();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: v1::web::player_state::PlayerState = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&original).unwrap(),
        serde_json::to_value(&parsed).unwrap()
    );
}

#[test]
fn roundtrip_default_world_snapshot() {
    let original = v1::web::world::WorldSnapshot::default();
    let json = serde_json::to_string(&original).unwrap();
    let parsed: v1::web::world::WorldSnapshot = serde_json::from_str(&json).unwrap();
    assert_eq!(
        serde_json::to_value(&original).unwrap(),
        serde_json::to_value(&parsed).unwrap()
    );
}
