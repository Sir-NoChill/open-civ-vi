//! `GET /api/v1/combat/preview` — predicted combat outcome for a tile target.

use open4x_protocol::v1::web::combat_preview::CombatPreview;

use crate::error::ApiError;
use crate::transport::{Method, Transport};

pub async fn preview<T: Transport>(
    t: &T,
    attacker_id: &str,
    defender_q: i32,
    defender_r: i32,
) -> Result<CombatPreview, ApiError> {
    let url = format!(
        "/api/v1/combat/preview?attacker_id={attacker_id}&defender_q={defender_q}&defender_r={defender_r}"
    );
    let body = t.request(Method::Get, &url, None).await?;
    serde_json::from_slice(&body).map_err(|e| ApiError::transport(e.to_string()))
}
