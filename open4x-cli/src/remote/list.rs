//! `list <kind>` over REST. Server-side `list units` / `list cities`
//! return own + visible foreign entities — wider than the local
//! handler's "own only" filter; the matrix flags this as expected
//! divergence.

use std::path::Path;

use crate::cli::ListKind;
use crate::handlers::parse_ulid;

use super::client::ApiClient;
use super::session::Session;

pub fn list(server: &str, token_file: &Path, kind: &ListKind) -> Result<(), String> {
    let session = Session::load(token_file)?;
    let client = ApiClient::new(server).with_token(session.token);

    let value = match kind {
        ListKind::Units => client.get_json("/api/v1/units")?,
        ListKind::Cities => client.get_json("/api/v1/cities")?,
        ListKind::Production { city } => {
            let raw = parse_ulid(city)?.to_string();
            // City detail carries the buildable items the wireframe
            // surfaces (`production_options` field).
            let body = client.get_json(&format!("/api/v1/cities/{raw}"))?;
            body.get("production_options").cloned().unwrap_or(body)
        }
        ListKind::GreatPeople
        | ListKind::Routes
        | ListKind::Governors
        | ListKind::Buildings
        | ListKind::Improvements => {
            return Err(format!(
                "server mode does not expose 'list {kind:?}' yet (no REST endpoint)"
            ));
        }
    };

    println!("{}", serde_json::to_string_pretty(&value).unwrap());
    Ok(())
}
