//! Bindings for `/api/v1/cities/*`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::city_data::{CityData, CityRow};
use crate::types::web::city_tiles::CityTiles;

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
