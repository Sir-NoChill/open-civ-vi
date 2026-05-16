//! Unified error type for SDK callers.
//!
//! Phase 0 (scaffolding): minimal shape only. Phase 2 wires the typed
//! `{error, message}` server body and per-backend transport errors.

#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: u16,
    pub code: String,
    pub message: Option<String>,
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
