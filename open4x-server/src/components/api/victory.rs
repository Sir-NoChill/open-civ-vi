//! Binding for `GET /api/v1/victory`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::victory::Victory;

pub async fn get(token: Option<&str>) -> Result<Victory, ApiError> {
    fetch_json::<Victory, ()>("GET", "/api/v1/victory", token, None).await
}
