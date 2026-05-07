//! Bindings for `/api/v1/cities/*`.

use serde::Serialize;

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::city_data::{CityData, CityRow};
use crate::types::web::city_tiles::CityTiles;
use crate::types::web::MutationResponse;

pub async fn list(token: Option<&str>) -> Result<CityData, ApiError> {
    fetch_json::<CityData, ()>("GET", "/api/v1/cities", token, None).await
}

pub async fn detail(token: Option<&str>, id: &str) -> Result<CityRow, ApiError> {
    let url = format!("/api/v1/cities/{id}");
    fetch_json::<CityRow, ()>("GET", &url, token, None).await
}

pub async fn tiles(token: Option<&str>, id: &str) -> Result<CityTiles, ApiError> {
    let url = format!("/api/v1/cities/{id}/tiles");
    fetch_json::<CityTiles, ()>("GET", &url, token, None).await
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueProductionBody {
    pub item_id: String,
    pub item_type: String, // "unit" | "building" | "wonder" | "project"
}

/// `POST /api/v1/cities/{id}/production`
pub async fn queue_production(
    token: Option<&str>,
    id: &str,
    body: &QueueProductionBody,
) -> Result<MutationResponse<CityRow>, ApiError> {
    let url = format!("/api/v1/cities/{id}/production");
    fetch_json::<MutationResponse<CityRow>, QueueProductionBody>("POST", &url, token, Some(body))
        .await
}

/// `DELETE /api/v1/cities/{id}/production/{pos}`
pub async fn cancel_production(
    token: Option<&str>,
    id: &str,
    pos: usize,
) -> Result<MutationResponse<CityRow>, ApiError> {
    let url = format!("/api/v1/cities/{id}/production/{pos}");
    fetch_json::<MutationResponse<CityRow>, ()>("DELETE", &url, token, None).await
}
