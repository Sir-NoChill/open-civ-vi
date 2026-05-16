//! `/api/v1/games/*` — single-player bootstrap.
//!
//! `POST /games/new` is the only authenticated-token-producing endpoint;
//! every subsequent SDK call attaches the returned `token` as a bearer
//! header.

use serde::{Deserialize, Serialize};

use open4x_protocol::v1::ids::{CivId, GameId};

use crate::error::ApiError;
use crate::transport::{Method, Transport};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// `POST /api/v1/games/new` — bootstrap a single-player game and mint a
/// bearer token. Unauthenticated.
pub async fn new_game<T: Transport>(
    t: &T,
    req: &NewGameRequest,
) -> Result<NewGameResponse, ApiError> {
    let body = serde_json::to_vec(req).map_err(|e| ApiError::transport(e.to_string()))?;
    let resp = t
        .request(Method::Post, "/api/v1/games/new", Some(&body))
        .await?;
    serde_json::from_slice(&resp).map_err(|e| ApiError::transport(e.to_string()))
}
