//! Bindings for `/api/v1/units/*`, `/api/v1/armies`, `/api/v1/combat/preview`.

use serde::Serialize;

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::army_data::ArmyData;
use crate::types::web::combat_preview::CombatPreview;
use crate::types::web::unit_data::{Unit, UnitData};
use crate::types::web::MutationResponse;

pub async fn list(token: Option<&str>) -> Result<UnitData, ApiError> {
    fetch_json::<UnitData, ()>("GET", "/api/v1/units", token, None).await
}

pub async fn detail(token: Option<&str>, id: &str) -> Result<Unit, ApiError> {
    let url = format!("/api/v1/units/{id}");
    fetch_json::<Unit, ()>("GET", &url, token, None).await
}

pub async fn armies(token: Option<&str>) -> Result<ArmyData, ApiError> {
    fetch_json::<ArmyData, ()>("GET", "/api/v1/armies", token, None).await
}

pub async fn combat_preview(
    token: Option<&str>,
    attacker_id: &str,
    defender_q: i32,
    defender_r: i32,
) -> Result<CombatPreview, ApiError> {
    let url = format!(
        "/api/v1/combat/preview?attacker_id={attacker_id}&defender_q={defender_q}&defender_r={defender_r}"
    );
    fetch_json::<CombatPreview, ()>("GET", &url, token, None).await
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UnitActionBody {
    pub action_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_q: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_r: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// `POST /api/v1/units/{id}/action` — dispatch a unit action. The view in the
/// returned `MutationResponse` is the updated `Unit`, or `null` for actions
/// that don't affect the unit (fortify/sleep stub).
pub async fn dispatch(
    token: Option<&str>,
    id: &str,
    body: &UnitActionBody,
) -> Result<MutationResponse<serde_json::Value>, ApiError> {
    let url = format!("/api/v1/units/{id}/action");
    fetch_json::<MutationResponse<serde_json::Value>, UnitActionBody>("POST", &url, token, Some(body))
        .await
}
