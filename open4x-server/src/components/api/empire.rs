//! Binding for `GET /api/v1/empire/overview`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::empire_overview::EmpireOverview;

pub async fn get(token: Option<&str>) -> Result<EmpireOverview, ApiError> {
    fetch_json::<EmpireOverview, ()>("GET", "/api/v1/empire/overview", token, None).await
}
