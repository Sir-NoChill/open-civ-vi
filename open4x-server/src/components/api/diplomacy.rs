//! Bindings for `/api/v1/diplomacy*`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::diplomacy::{CivRow, Diplomacy};

pub async fn get(token: Option<&str>) -> Result<Diplomacy, ApiError> {
    fetch_json::<Diplomacy, ()>("GET", "/api/v1/diplomacy", token, None).await
}

pub async fn civ(token: Option<&str>, id: &str) -> Result<CivRow, ApiError> {
    let url = format!("/api/v1/diplomacy/civs/{id}");
    fetch_json::<CivRow, ()>("GET", &url, token, None).await
}
