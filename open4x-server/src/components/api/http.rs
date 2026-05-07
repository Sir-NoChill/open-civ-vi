//! Fetch helper used by every binding under [`crate::components::api`].
//!
//! Wraps `web_sys::Request` + `Window::fetch_with_request`, attaches the
//! bearer token, JSON-encodes the body, and decodes the response with
//! `serde-wasm-bindgen`. No JS code — pure wasm-bindgen.

use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: Option<String>,
}

impl ApiError {
    pub fn transport(message: impl Into<String>) -> Self {
        Self {
            status: 0,
            code: "transport".to_string(),
            message: Some(message.into()),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{} ({}): {m}", self.code, self.status),
            None => write!(f, "{} ({})", self.code, self.status),
        }
    }
}

/// Issue a single REST request and decode the JSON response.
///
/// `body` is `Some` for write methods (POST/PUT/PATCH) and `None` for reads.
pub async fn fetch_json<T: DeserializeOwned, B: Serialize>(
    method: &str,
    url: &str,
    token: Option<&str>,
    body: Option<&B>,
) -> Result<T, ApiError> {
    let init = RequestInit::new();
    init.set_method(method);

    if let Some(b) = body {
        let json = serde_json::to_string(b).map_err(|e| ApiError::transport(e.to_string()))?;
        init.set_body(&JsValue::from_str(&json));
    }

    let req = Request::new_with_str_and_init(url, &init)
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;

    let headers = req.headers();
    headers
        .set("accept", "application/json")
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;
    if body.is_some() {
        headers
            .set("content-type", "application/json")
            .map_err(|e| ApiError::transport(format!("{e:?}")))?;
    }
    if let Some(t) = token {
        headers
            .set("authorization", &format!("Bearer {t}"))
            .map_err(|e| ApiError::transport(format!("{e:?}")))?;
    }

    let window = web_sys::window().ok_or_else(|| ApiError::transport("no window"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&req))
        .await
        .map_err(|e| ApiError::transport(format!("{e:?}")))?;
    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| ApiError::transport("non-Response result"))?;

    let status = resp.status();
    let text = JsFuture::from(
        resp.text()
            .map_err(|e| ApiError::transport(format!("{e:?}")))?,
    )
    .await
    .map_err(|e| ApiError::transport(format!("{e:?}")))?
    .as_string()
    .unwrap_or_default();

    if !(200..300).contains(&status) {
        let body: crate::types::web::ApiErrorBody = serde_json::from_str(&text).unwrap_or_default();
        return Err(ApiError {
            status,
            code: if body.error.is_empty() {
                "http_error".into()
            } else {
                body.error
            },
            message: body.message,
        });
    }

    serde_json::from_str::<T>(&text).map_err(|e| ApiError::transport(e.to_string()))
}
