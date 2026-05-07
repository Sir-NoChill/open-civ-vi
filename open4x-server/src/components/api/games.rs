//! Bindings for `/api/v1/games/*`.

use serde::{Deserialize, Serialize};

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::ids::{CivId, GameId};

#[derive(Debug, Clone, Default, Serialize)]
pub struct NewGameRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ai: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewGameResponse {
    pub game_id: GameId,
    pub civ_id: CivId,
    pub token: String,
    pub turn: u32,
}

/// `POST /api/v1/games/new` — bootstrap a single-player game and receive a
/// bearer token. Subsequent calls authenticate with `Bearer <token>`.
pub async fn new(req: &NewGameRequest) -> Result<NewGameResponse, ApiError> {
    fetch_json::<NewGameResponse, NewGameRequest>("POST", "/api/v1/games/new", None, Some(req))
        .await
}
