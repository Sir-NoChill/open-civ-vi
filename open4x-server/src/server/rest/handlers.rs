//! REST handler functions for `/api/v1/*`.
//!
//! Phase 1 wires the HUD MVP: `health`, `player_state`, `world_snapshot`,
//! `world_tile`, and `end_turn`. Subsequent phases extend the surface from
//! `book/src/roadmap/web-ui.md` §4.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::server::api_token::{generate_token, ApiTokenRecord};
use crate::server::projection::project_game_view;
use crate::server::rest::auth::{auth_or_401, ApiError};
use crate::server::state::{AppState, GameRoom, GameRoomConfig, PlayerRecord};
use crate::server::web_projection;
use crate::types::ids::{CivId, GameId};
use crate::types::messages::{CreateGameRequest, GameStatus};
use crate::types::view::GameView;
use crate::types::web::{MutationResponse, TurnStatusBlock};

#[derive(Serialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub api: &'static str,
}

/// `GET /api/v1/health` — unauthenticated liveness check.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        api: "v1",
    })
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Project a fresh `GameView` for the authenticated player and pull the
/// configured `turn_limit` from the room.
fn view_and_turn_limit(
    state: &Arc<AppState>,
    game_id: GameId,
    civ_id: CivId,
) -> Result<(GameView, Option<u32>), ApiError> {
    let room = state
        .games
        .get(&game_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("game not found"))?;
    let libciv_civ_id = libciv::CivId::from_ulid(civ_id.as_ulid());
    let view = project_game_view(&room.state, libciv_civ_id);
    Ok((view, room.config.turn_limit))
}

fn turn_status_block(state: &Arc<AppState>, game_id: GameId) -> TurnStatusBlock {
    let turn = state
        .games
        .get(&game_id)
        .map(|r| r.state.turn)
        .unwrap_or(0);
    TurnStatusBlock { turn, ended: false }
}

// ── POST /games/new — bootstrap a single-player game over REST ───────────────

#[derive(Deserialize, Default)]
pub struct NewGameRequest {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default)]
    pub num_ai: Option<u32>,
    #[serde(default)]
    pub turn_limit: Option<u32>,
}

#[derive(Serialize)]
pub struct NewGameResponse {
    pub game_id: GameId,
    pub civ_id: CivId,
    pub token: String,
    pub turn: u32,
}

/// `POST /api/v1/games/new` — create a fresh single-player game and mint a
/// bearer token for it. Unauthenticated; intended for the single-player REST
/// loop (multiplayer keeps using the WS auth handshake).
pub async fn new_game(
    State(state): State<Arc<AppState>>,
    Json(req): Json<NewGameRequest>,
) -> Result<impl IntoResponse, ApiError> {
    use rand::Rng;

    let display_name = req.display_name.clone().unwrap_or_else(|| "Player".into());

    // Generate an anonymous pubkey for this single-player session.
    let mut rng = rand::rng();
    let pubkey: [u8; 32] = rng.random();

    // Register a PlayerRecord so build_server_session can look it up.
    state.players.insert(
        pubkey,
        PlayerRecord {
            pubkey,
            display_name: display_name.clone(),
            selected_template: state.templates[0].id,
            games_played: 0,
        },
    );

    let game_id = GameId::from_ulid(ulid::Ulid::new());

    let create_req = CreateGameRequest {
        name: format!("{display_name}'s game"),
        width: req.width.unwrap_or(40),
        height: req.height.unwrap_or(24),
        seed: req.seed.unwrap_or(42),
        num_ai: req.num_ai.unwrap_or(1),
        max_players: 1,
        turn_limit: req.turn_limit.or(Some(500)),
    };

    let session = crate::server::session::build_server_session(&create_req, &pubkey, &state, game_id);
    let civ_id = session
        .players
        .first()
        .map(|s| s.civ_id)
        .ok_or_else(|| crate::server::rest::auth::bad_request("no_player_slot", "session has no player"))?;

    let (tx, _rx) = broadcast::channel(64);
    let room = GameRoom {
        game_id,
        name: create_req.name.clone(),
        state: session.state,
        rules: libciv::DefaultRulesEngine,
        players: session.players,
        ai_agents: session.ai_agents,
        status: GameStatus::InProgress,
        config: GameRoomConfig {
            max_players: 1,
            turn_limit: create_req.turn_limit,
        },
        tx,
    };

    let initial_turn = room.state.turn;
    state.games.insert(game_id, room);

    // Mint and store the bearer token.
    let token = generate_token();
    state.api_tokens.insert(
        token.clone(),
        ApiTokenRecord {
            token: token.clone(),
            pubkey,
            game_id,
            civ_id: CivId::from_ulid(civ_id.as_ulid()),
        },
    );

    Ok((
        StatusCode::CREATED,
        Json(NewGameResponse {
            game_id,
            civ_id: CivId::from_ulid(civ_id.as_ulid()),
            token,
            turn: initial_turn,
        }),
    ))
}

// ── /player-state ────────────────────────────────────────────────────────────

pub async fn player_state(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, turn_limit) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_player_state(&view, turn_limit)))
}

// ── /world/snapshot ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WorldSnapshotQuery {
    pub q: Option<i32>,
    pub r: Option<i32>,
    pub radius: Option<u32>,
}

pub async fn world_snapshot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<WorldSnapshotQuery>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let q = params.q.unwrap_or(0);
    let r = params.r.unwrap_or(0);
    // Default radius 0 = "all explored". The plan caps the radius at 32.
    let radius = params.radius.unwrap_or(0).min(32);
    Ok(Json(web_projection::build_world_snapshot(&view, q, r, radius)))
}

// ── /world/tile/{q}/{r} ──────────────────────────────────────────────────────

pub async fn world_tile(
    State(state): State<Arc<AppState>>,
    Path((q, r)): Path<(i32, i32)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    let snapshot = web_projection::build_world_snapshot(&view, q, r, 1);
    snapshot
        .tiles
        .into_iter()
        .find(|t| t.q == q && t.r == r)
        .map(Json)
        .ok_or_else(|| crate::server::rest::auth::not_found("tile not in player view"))
}

// ── POST /turn/end ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EndTurnView {
    pub turn: u32,
}

pub async fn end_turn(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let libciv_civ_id = libciv::CivId::from_ulid(civ_id.as_ulid());

    let mut room = state
        .games
        .get_mut(&game_id)
        .ok_or_else(|| crate::server::rest::auth::not_found("game not found"))?;

    // Mark this player's slot submitted.
    if let Some(slot) = room.players.iter_mut().find(|s| s.civ_id == libciv_civ_id) {
        slot.submitted_turn = true;
    }

    // For Phase 1 single-player, advance immediately. Multiplayer gating
    // (room.all_submitted()) lives on the WS path and is out of scope here.
    room.resolve_turn();

    let new_turn = room.state.turn;
    drop(room);

    Ok((
        StatusCode::OK,
        Json(MutationResponse {
            ok: true,
            view: EndTurnView { turn: new_turn },
            turn_status: TurnStatusBlock {
                turn: new_turn,
                ended: false,
            },
        }),
    ))
}

// ── Phase 2+ stubs ───────────────────────────────────────────────────────────

#[allow(dead_code)]
pub async fn map_overlays(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let (game_id, civ_id) = auth_or_401(&state, &headers)?;
    let (view, _) = view_and_turn_limit(&state, game_id, civ_id)?;
    Ok(Json(web_projection::build_map_overlays(&view)))
}
