//! `view` over REST: `GET /world/snapshot` (radius=0 = all explored).
//! Note this returns the wireframe `WorldSnapshot` shape, not the
//! local `PlayerView` shape — see the parity matrix for details.

use std::path::Path;

use super::client::ApiClient;
use super::session::Session;

pub fn view(server: &str, token_file: &Path) -> Result<(), String> {
    let session = Session::load(token_file)?;
    let client = ApiClient::new(server).with_token(session.token);

    let resp = client.get_json("/api/v1/world/snapshot?radius=0")?;
    println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    Ok(())
}
