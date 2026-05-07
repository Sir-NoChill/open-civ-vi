//! Binding for `GET /api/v1/government`.

use crate::components::api::http::{ApiError, fetch_json};
use crate::types::web::government::GovernmentPolicies;

pub async fn get(token: Option<&str>) -> Result<GovernmentPolicies, ApiError> {
    fetch_json::<GovernmentPolicies, ()>("GET", "/api/v1/government", token, None).await
}
