//! `GET /api/v1/player-state` — top-bar HUD numbers for the active civ.

use open4x_protocol::v1::web::player_state::PlayerState;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn get<T: Transport>(t: &T) -> Result<PlayerState, ApiError> {
    let body = t.request(Method::Get, "/api/v1/player-state", None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
