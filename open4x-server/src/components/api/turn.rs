//! Binding for `POST /api/v1/turn/end`.

use serde::{Deserialize, Serialize};

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::MutationResponse;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EndTurnView {
    pub turn: u32,
}

const PATH: &str = "/api/v1/turn/end";

#[derive(Serialize)]
struct Empty {}

/// `POST /api/v1/turn/end` — single-player advance. Returns the resolved
/// turn number plus the standard mutation envelope.
pub async fn end(token: Option<&str>) -> Result<MutationResponse<EndTurnView>, ApiError> {
    fetch_json::<MutationResponse<EndTurnView>, Empty>("POST", PATH, token, Some(&Empty {})).await
}
