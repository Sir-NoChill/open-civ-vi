# Web Client

The `open4x-server` crate (with the `csr` feature) provides a browser-based
game client built with Leptos and compiled to WebAssembly.

The crate is dual-purpose: with `--features ssr` it builds a native Axum
server binary; with `--features csr --target wasm32-unknown-unknown` it
builds the cdylib that becomes the SPA bundle. The `[[bin]]` entry in
`Cargo.toml` is gated on `ssr`, so trunk's wasm pipeline picks up the
library's `lib.rs::start()` (`#[wasm_bindgen(start)]`) instead.

## Technology Stack

- **Leptos** (v0.7, `csr` feature) -- reactive UI framework
- **wasm-bindgen** -- Rust/JavaScript interop
- **web-sys** -- `Window::fetch_with_request`, WebSocket, Storage
- **ed25519-dalek** -- client-side key generation and signing (multiplayer)
- **Trunk** -- WASM build tool and dev server

## Two transport surfaces

The client runs against two transport surfaces depending on the use case:

| Surface | Used by | Mounted in `main.rs` |
|---|---|---|
| `/api/v1/*` REST | single-player gameplay (current focus) | `Router::nest("/api/v1", server::rest::v1_router())` |
| `/ws` WebSocket | AI demo, future multiplayer | `Router::route("/ws", get(ws::ws_handler))` |

Single-player flow goes through REST exclusively. The WebSocket and the
`ClientMessage`/`ServerMessage` enums are kept untouched for the AI demo and
future hot-seat / multiplayer plans (out of scope here).

## REST API surface (`/api/v1/*`)

All endpoints (besides `/health`) require `Authorization: Bearer <token>`. A
token is minted by `POST /api/v1/games/new`, which also creates the
`GameRoom`. Returns shape: `{ game_id, civ_id, token, turn }`.

Read endpoints (GET):

| Path | Returns |
|---|---|
| `/health` | `{ ok, api: "v1" }` (unauthenticated) |
| `/player-state` | turn, era, yields, gold, faith, happiness, strategics |
| `/world/snapshot?q&r&radius` | board metadata + tiles within `radius` |
| `/world/tile/{q}/{r}` | single tile detail |
| `/map/overlays` | overlay toggle catalogue |
| `/cities` | all known cities (own + foreign) |
| `/cities/{id}` | single city |
| `/cities/{id}/tiles` | territory + worked + center flag |
| `/units` | all visible units, with allowed action menu |
| `/units/{id}` | single unit |
| `/armies` | army formations (stub for Phase 4) |
| `/combat/preview?attacker_id&defender_q&defender_r` | combat odds |
| `/tech` | tech tree with status (done/current/available/locked) |
| `/civics` | civic tree, same shape |
| `/government` | current gov + slots + active policies |
| `/diplomacy` | known civs and city-states |
| `/diplomacy/civs/{id}` | single civ |
| `/empire/overview` | dashboard aggregates |
| `/victory` | conditions + leaderboard |
| `/notifications` | event feed (ring buffer pending) |
| `/turn-queue` | required + optional pending actions |
| `/registry` | unit-type and building catalogues |

Write endpoints:

| Method | Path | Body | Maps to |
|---|---|---|---|
| POST | `/games/new` | `{display_name?, width?, height?, seed?, num_ai?, turn_limit?}` | bootstrap |
| POST | `/turn/end` | `{}` | `EndTurn` (400 if required queue items pending) |
| POST | `/units/{id}/action` | `{action_id, target_q?, target_r?, name?}` | `MoveUnit` / `Attack` / `FoundCity` |
| POST | `/cities/{id}/production` | `{item_id, item_type}` | `QueueProduction` |
| DELETE | `/cities/{id}/production/{pos}` | — | `CancelProduction` |
| POST | `/tech/research` | `{tech_id}` | `QueueResearch` |
| POST | `/civics/research` | `{civic_id}` | `QueueCivic` |

### Response envelopes

Successful mutations return:

```json
{ "ok": true, "view": <slice>, "turn_status": { "turn": N, "ended": false } }
```

Errors return one of:

```
401 → { "error": "missing_or_invalid_token" }
404 → { "error": "not_found", "message": "..." }
400 → { "error": "<machine_code>", "message": "..." }
400 → { "error": "unresolved_required_actions", "items": [...] }   (only on /turn/end)
```

The full plan and changelog live in
[`book/src/roadmap/web-ui.md`](../roadmap/web-ui.md).

## Client architecture

The client mirrors the wireframe's contract verbatim through wire types in
`open4x-server/src/types/web.rs`. Each REST endpoint has:

- a wire type (e.g. `types::web::city_data::CityData`)
- a server projector (e.g. `server::web_projection::build_cities`)
- a route handler (e.g. `server::rest::handlers::cities`)
- a client binding (e.g. `components::api::cities::list`)

The browser-side bindings layer (`components::api::*`) wraps
`web_sys::Request` through a single `fetch_json(method, url, token, body)`
helper. There is no JavaScript shim — every call is pure Rust + wasm-bindgen.

### Reactive state

Per `book/src/roadmap/web-ui.md` §6.2, screens own `LocalResource`s keyed on a
shared refresh tick. Mutations bump the tick, refetching every active slice in
parallel. There is no monolithic `GameView` signal on the client; each tab is
free to subscribe only to the slices it needs.

## Pages

| Page | Component | Description |
|------|-----------|-------------|
| REST single-player | `RestGamePage` | Default: bootstraps via `/games/new`, renders the topbar + sidebar from REST |
| Home (legacy) | `HomePage` | Main menu (kept for the WS demo flow) |
| Map Config | `MapConfigPage` | Set map size, seed, AI count |
| Game (WS) | `GamePage` | Original WebSocket-driven page |
| Demo Config | `DemoConfigPage` | Configure AI-vs-AI demo parameters |
| Replay | `ReplayPage` | Animate and visualize demo game results |
| Settings | `SettingsPage` | User preferences (deferred) |
| Players | `PlayersPage` | Player list (deferred) |

## WebSocket Client (legacy / multiplayer)

```rust
struct WsClient { socket: Rc<WebSocket> }
```

Used for the AI demo. The connection flow does an Ed25519 challenge handshake
(`Challenge` → `Authenticate` → `AuthSuccess`) and then exchanges
`ClientMessage` / `ServerMessage` JSON. Single-player REST does not touch
this path.

## Build

```bash
rustup target add wasm32-unknown-unknown          # one-time
cargo install trunk                                # one-time

cd open4x-server
trunk serve --features csr --no-default-features  # dev server
trunk build --release --features csr --no-default-features
```

Then start the native server:

```bash
cargo run -p open4x-server   # serves /api/*, /ws, and dist/
```

The WASM target requires `getrandom_backend="wasm_js"`, configured in
`.cargo/config.toml` for `wasm32-unknown-unknown`. This ensures all
transitive deps agree on the WASM random backend.

## Configuration

`OPEN4X_STATIC_DIR` (default `./open4x-server/dist`) tells the server where
to find the trunk-built bundle. `PORT` (default `3001`) sets the listen
address. The client connects to the same origin for both REST and WS.
