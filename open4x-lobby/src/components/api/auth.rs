//! Bindings for `/api/v1/auth/*`.

use serde::{Deserialize, Serialize};

use super::http::{ApiError, fetch_json};

#[derive(Debug, Clone, Serialize)]
pub struct EmailStartBody {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailStartResp {
    pub ok: bool,
    pub message: String,
}

/// `POST /api/v1/auth/email/start`. The browser sends the request
/// over the same origin as the SPA; the lobby returns 202 + a
/// `magic_link_sent` ack.
pub async fn email_start(email: String) -> Result<EmailStartResp, ApiError> {
    let body = EmailStartBody { email };
    fetch_json::<EmailStartResp, EmailStartBody>(
        "POST",
        "/api/v1/auth/email/start",
        Some(&body),
    )
    .await
}

/// `POST /api/v1/auth/signout`.
pub async fn signout() -> Result<(), ApiError> {
    fetch_json::<serde_json::Value, ()>("POST", "/api/v1/auth/signout", None)
        .await
        .map(|_| ())
}
