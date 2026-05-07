//! Stub binding for `GET /api/v1/player-state`. Real call lands in Phase 1.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::player_state::PlayerState;

const PATH: &str = "/api/v1/player-state";

pub async fn get(token: Option<&str>) -> Result<PlayerState, ApiError> {
    fetch_json::<PlayerState, ()>("GET", PATH, token, None).await
}
