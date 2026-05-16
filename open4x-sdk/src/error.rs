//! Unified error type for SDK callers.
//!
//! Every transport surface — native blocking, native async, wasm fetch —
//! funnels failures through [`ApiError`]. Non-2xx HTTP responses are
//! decoded via [`ApiError::from_response`], which understands the server's
//! `{error, message}` JSON envelope (see
//! [`open4x_protocol::v1::web::ApiErrorBody`]).
//!
//! Transport-level failures (connection refused, DNS, JSON decode of the
//! success body) use [`ApiError::transport`] with `status = 0`.

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerErrorBody {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

impl ApiError {
    /// Build a transport-layer error (no HTTP status to report).
    pub fn transport(msg: impl Into<String>) -> ApiError {
        ApiError {
            status: 0,
            code: "transport".into(),
            message: Some(msg.into()),
        }
    }

    /// Decode a non-2xx HTTP response into an [`ApiError`].
    ///
    /// Tries to parse `body` as `{error, message}` (the
    /// [`open4x_protocol::v1::web::ApiErrorBody`] shape). If the body
    /// isn't JSON or doesn't carry those fields, falls back to the raw
    /// body text as the `message` and a generic `http_error` code.
    pub fn from_response(status: u16, body: &str) -> ApiError {
        match serde_json::from_str::<ServerErrorBody>(body) {
            Ok(parsed) => ApiError {
                status,
                code: parsed.error.unwrap_or_else(|| "http_error".into()),
                message: parsed
                    .message
                    .or_else(|| (!body.is_empty()).then(|| body.to_string())),
            },
            Err(_) => ApiError {
                status,
                code: "http_error".into(),
                message: (!body.is_empty()).then(|| body.to_string()),
            },
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

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_response_decodes_structured_body() {
        let body = r#"{"error":"not_found","message":"game not found"}"#;
        let err = ApiError::from_response(404, body);
        assert_eq!(err.status, 404);
        assert_eq!(err.code, "not_found");
        assert_eq!(err.message.as_deref(), Some("game not found"));
    }

    #[test]
    fn from_response_falls_back_for_non_json() {
        let err = ApiError::from_response(500, "boom");
        assert_eq!(err.status, 500);
        assert_eq!(err.code, "http_error");
        assert_eq!(err.message.as_deref(), Some("boom"));
    }

    #[test]
    fn from_response_handles_partial_body() {
        let err = ApiError::from_response(400, r#"{"error":"bad"}"#);
        assert_eq!(err.code, "bad");
        // No "message" field → fallback to the raw body text.
        assert!(err.message.unwrap().contains("bad"));
    }

    #[test]
    fn transport_uses_zero_status() {
        let err = ApiError::transport("connection refused");
        assert_eq!(err.status, 0);
        assert_eq!(err.code, "transport");
        assert_eq!(err.message.as_deref(), Some("connection refused"));
    }
}
