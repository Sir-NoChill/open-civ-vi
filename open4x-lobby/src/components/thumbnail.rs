//! Real-game tile thumbnails (Phase 5 polish).
//!
//! The browser fetches `/api/v1/games/{id}/thumbnail` (same-origin,
//! cookie auth). The lobby's handler issues a Bearer-authed
//! request against `<server_url>/api/v1/world/snapshot` server-
//! side, reduces the response to a `(q, r, terrain)` triple grid,
//! and returns it as JSON. The SPA caches the response in an
//! in-session `RwSignal<HashMap<game_id, ThumbnailGrid>>` so
//! repeat mounts (filter changes, sort flips) don't re-fetch.

#![cfg(feature = "csr")]

use std::collections::HashMap;

use leptos::prelude::*;
use serde::Deserialize;

use super::api::http::fetch_json;

/// One tile reduced to what the minimap needs.
#[derive(Debug, Clone)]
pub struct ThumbCell {
    pub q: i32,
    pub r: i32,
    pub terrain: String,
}

/// Reduced shape: world dimensions + the tile cells. Cached in
/// the [`ThumbnailCache`] under the per-game `server_token`.
#[derive(Debug, Clone)]
pub struct ThumbnailGrid {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<ThumbCell>,
}

/// Per-token state in the cache. `Pending` covers the in-flight
/// case so repeat mounts during a fetch don't all kick off
/// duplicate requests.
#[derive(Clone)]
pub enum ThumbnailEntry {
    Pending,
    Ready(ThumbnailGrid),
    Failed(String),
}

#[derive(Clone)]
pub struct ThumbnailCache {
    inner: RwSignal<HashMap<String, ThumbnailEntry>, LocalStorage>,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        Self {
            inner: RwSignal::new_local(HashMap::new()),
        }
    }

    /// Snapshot read; cheap-ish (Hash clone). The component only
    /// re-renders when the signal is `.get()`-ed in a reactive
    /// scope, so callers should call this from inside a
    /// `move ||` closure.
    pub fn get_entry(&self, token: &str) -> Option<ThumbnailEntry> {
        self.inner.with(|m| m.get(token).cloned())
    }

    pub fn set_entry(&self, token: String, entry: ThumbnailEntry) {
        self.inner.update(|m| {
            m.insert(token, entry);
        });
    }
}

impl Default for ThumbnailCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Mount once at the App root. Subsequent `expect_context` calls
/// in any descendant return the same shared cache.
pub fn provide_thumbnail_cache() {
    provide_context(ThumbnailCache::new());
}

pub fn use_thumbnail_cache() -> ThumbnailCache {
    expect_context::<ThumbnailCache>()
}

/// Wire shape returned by `GET /api/v1/games/{id}/thumbnail`.
/// Mirrors [`ThumbnailGrid`] for direct deserialization.
#[derive(Deserialize)]
struct WireThumb {
    width: u32,
    height: u32,
    #[serde(default)]
    cells: Vec<WireCell>,
}

#[derive(Deserialize)]
struct WireCell {
    q: i32,
    r: i32,
    terrain: String,
}

/// Same-origin fetch via the lobby's thumbnail proxy. Cookie auth
/// (the lobby's session) replaces the cross-origin Bearer hand-
/// shake. The proxy does the bearer'd hop to the game server
/// server-side using the games row's stored `server_token`.
pub async fn fetch_thumbnail(game_id: &str) -> Result<ThumbnailGrid, String> {
    let url = format!("/api/v1/games/{game_id}/thumbnail");
    let wire: WireThumb = fetch_json::<WireThumb, ()>("GET", &url, None)
        .await
        .map_err(|e| e.to_string())?;
    let cells = wire
        .cells
        .into_iter()
        .map(|c| ThumbCell {
            q: c.q,
            r: c.r,
            terrain: c.terrain,
        })
        .collect();
    Ok(ThumbnailGrid {
        width: wire.width,
        height: wire.height,
        cells,
    })
}

/// Kick off (or no-op if already cached) a background fetch for
/// `game_id`. Idempotent: pending or ready entries are left alone.
/// Returns immediately; the cache signal updates when the fetch
/// resolves and any subscribed view re-renders.
pub fn ensure_fetched(cache: &ThumbnailCache, game_id: String) {
    if game_id.is_empty() {
        return;
    }
    if cache.get_entry(&game_id).is_some() {
        return;
    }
    cache.set_entry(game_id.clone(), ThumbnailEntry::Pending);
    let cache_clone: ThumbnailCache = cache.clone();
    leptos::task::spawn_local(async move {
        match fetch_thumbnail(&game_id).await {
            Ok(grid) => cache_clone.set_entry(game_id, ThumbnailEntry::Ready(grid)),
            Err(e) => cache_clone.set_entry(game_id, ThumbnailEntry::Failed(e)),
        }
    });
}

/// Map a terrain name (possibly composite like `Grassland+Hills`)
/// to one of the existing landing/CSS classes used by the design's
/// `.svg-map .water` / `.land` / `.land-self` rules. Substring
/// matching keeps things forgiving when the engine adds new
/// terrain variants without the SPA needing to be retrained.
pub fn terrain_class(terrain: &str) -> &'static str {
    let t = terrain.to_lowercase();
    if t.contains("ocean")
        || t.contains("coast")
        || t.contains("water")
        || t.contains("lake")
        || t.contains("reef")
    {
        "water"
    } else if t.contains("hill") || t.contains("mountain") {
        "land-self"
    } else {
        // Grassland / Plains / Tundra / Desert / Snow / etc.
        "land"
    }
}

