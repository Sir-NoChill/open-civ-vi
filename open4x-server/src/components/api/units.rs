//! Bindings for `/api/v1/units/*`, `/api/v1/armies`, `/api/v1/combat/preview`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::army_data::ArmyData;
use crate::types::web::combat_preview::CombatPreview;
use crate::types::web::unit_data::{Unit, UnitData};

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
