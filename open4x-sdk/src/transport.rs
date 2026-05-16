//! Transport abstraction shared by the `native` and `wasm` backends.
//!
//! Each backend implements [`Transport`]; the per-resource endpoint
//! functions in [`crate::endpoints`] are generic over `T: Transport` and
//! serialise/deserialise via [`open4x_protocol`] types.
//!
//! The trait is async to give the wasm backend a natural fit. The
//! blocking native backend can wrap an `async fn` call in
//! `tokio::runtime::Runtime::block_on` internally.

use crate::error::ApiError;

/// HTTP methods this SDK uses. Kept as an enum (not a string) so callers
/// can't accidentally typo a method name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }
}

/// Backend-agnostic HTTP transport.
///
/// Implementations are responsible for:
/// - prefixing `path` with the base URL the client was constructed with
///   (relative paths start with `/`, e.g. `/api/v1/cities`),
/// - attaching the bearer token if one is configured,
/// - setting `Content-Type: application/json` when `body` is `Some`,
/// - returning the raw response body bytes on success (caller decodes),
/// - translating non-2xx responses into [`ApiError`] using the server's
///   `{error, message}` JSON shape when present.
pub trait Transport {
    /// Execute one HTTP request. `body` is JSON-encoded bytes for write
    /// methods or `None` for reads.
    fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&[u8]>,
    ) -> impl std::future::Future<Output = Result<Vec<u8>, ApiError>> + Send;
}
