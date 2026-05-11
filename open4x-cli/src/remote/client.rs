//! Thin HTTP wrapper used by every remote handler. Single source of
//! truth for: base URL handling, bearer-auth header, and translating
//! the server's `{error, message}` body into a flat `String`.

use serde_json::Value;

pub struct ApiClient {
    base: String,
    token: Option<String>,
    inner: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base: impl Into<String>) -> Self {
        let mut base = base.into();
        while base.ends_with('/') {
            base.pop();
        }
        Self {
            base,
            token: None,
            inner: reqwest::blocking::Client::builder()
                .build()
                .expect("failed to build reqwest client"),
        }
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base, path)
        } else {
            format!("{}/{}", self.base, path)
        }
    }

    fn maybe_auth(&self, req: reqwest::blocking::RequestBuilder) -> reqwest::blocking::RequestBuilder {
        match &self.token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    pub fn get_json(&self, path: &str) -> Result<Value, String> {
        let resp = self
            .maybe_auth(self.inner.get(self.url(path)))
            .send()
            .map_err(|e| format!("GET {path}: {e}"))?;
        decode(resp, &format!("GET {path}"))
    }

    pub fn post_json(&self, path: &str, body: &Value) -> Result<Value, String> {
        let resp = self
            .maybe_auth(self.inner.post(self.url(path)).json(body))
            .send()
            .map_err(|e| format!("POST {path}: {e}"))?;
        decode(resp, &format!("POST {path}"))
    }

    pub fn delete_json(&self, path: &str) -> Result<Value, String> {
        let resp = self
            .maybe_auth(self.inner.delete(self.url(path)))
            .send()
            .map_err(|e| format!("DELETE {path}: {e}"))?;
        decode(resp, &format!("DELETE {path}"))
    }
}

/// Drain the response. On non-2xx, surface the server's structured
/// `{error, message}` body if present, else the raw text.
fn decode(resp: reqwest::blocking::Response, ctx: &str) -> Result<Value, String> {
    let status = resp.status();
    let text = resp
        .text()
        .map_err(|e| format!("{ctx}: failed to read response body: {e}"))?;

    if status.is_success() {
        if text.is_empty() {
            return Ok(Value::Null);
        }
        return serde_json::from_str(&text)
            .map_err(|e| format!("{ctx}: invalid JSON response ({e}): {text}"));
    }

    let body: Value = serde_json::from_str(&text).unwrap_or(Value::String(text.clone()));
    let err_code = body.get("error").and_then(|v| v.as_str()).unwrap_or("http_error");
    let msg = body
        .get("message")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| text.clone());
    Err(format!("{ctx} -> {} {}: {} ({})", status.as_u16(), err_code, msg, body))
}
