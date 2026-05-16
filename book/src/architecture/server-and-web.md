# Server & Web Client

The multiplayer stack is split across four single-purpose crates:

| Crate | Target | Role |
|-------|--------|------|
| `open4x-protocol` | any | Versioned wire types under `v1::*` — the contract between server and clients. |
| `open4x-sdk` | native + `wasm32` | Typed HTTP client with two transports (`native-blocking` / `native-async` via `reqwest`, `wasm` via `web_sys::fetch`). |
| `open4x-server` | native | Axum REST + WebSocket server, `GameRoom` orchestration, fog-of-war projection. No Leptos, no cdylib. |
| `open4x-client-web` | `wasm32-unknown-unknown` | Leptos CSR frontend (cdylib) — consumes the SDK. |

> **History**: a previous iteration merged three legacy crates (`open4x-api`, `open4x-server`, `open4x-web`) into one dual-purpose crate gated by `ssr` / `csr` feature flags. The [crate-split roadmap](../roadmap/crate-split.md) records how that crate was decomposed back into single-purpose crates with an explicit protocol surface.

## Wire Protocol Types (`open4x-protocol`)

All wire types live under `open4x_protocol::v1::*` and derive `Serialize`/`Deserialize`. The `tests/wire_schema.rs` snapshots guard against accidental schema drift — any PR that intentionally changes a field must update them; any PR that accidentally changes one fails CI.

### Key Types

| Type | Description |
|------|-------------|
| `ClientMessage` | All messages the client can send (auth, actions, lobby) |
| `ServerMessage` | All messages the server can send (views, results, errors) |
| `GameAction` | Player actions: move, attack, found city, build, research, trade, diplomacy |
| `GameView` | Full game state projection for one player (fog-of-war filtered) |
| `BoardView` | Tile grid with terrain, features, resources, visibility |
| `CivView` | Full detail for the player's own civilization |
| `PublicCivView` | Limited info for other civilizations |
| `CityView` | City detail (full for own cities, limited for foreign) |
| `UnitView` | Unit state (position, health, movement) |
| `TechTreeView` | Tech tree with research status |
| `ProfileView` | Player profile with display name and civ template |
| `CivTemplate` | Civilization definition (name, leader, abilities, uniques) |

Per-endpoint slice types live under `v1::web::*` (e.g. `web::city_data::CityData`, `web::player_state::PlayerState`).

### GameAction Variants

```rust
enum GameAction {
    MoveUnit { unit, to },
    Attack { attacker, defender },
    FoundCity { settler, name },
    PlaceImprovement { coord, improvement },
    QueueProduction { city, item },
    CancelProduction { city, index },
    EstablishTradeRoute { trader, destination },
    QueueResearch { tech },
    QueueCivic { civic },
    AssignCitizen { city, tile, lock },
    UnassignCitizen { city, tile },
    DeclareWar { target },
    MakePeace { target },
    AssignPolicy { policy },
}
```

## Typed Client (`open4x-sdk`)

The SDK provides a `Transport` trait + concrete backends + one async function per `/api/v1/*` resource:

```text
open4x-sdk/src/
  transport.rs        — `Transport` trait + `Method` enum
  error.rs            — `ApiError` + helpers for decoding `{error, message}` envelopes
  native.rs           — `NativeBlockingClient` / `NativeAsyncClient`  (features: native-blocking, native-async)
  wasm.rs             — `WasmClient` over `web_sys::fetch`             (feature: wasm; wasm32 only)
  endpoints/          — one module per resource (cities, units, tech, ...); functions generic over `T: Transport`
```

Both the native CLI (`open4x-cli/src/remote/client.rs` is a thin shim over `NativeBlockingClient`) and the browser client (`open4x-client-web`) consume the same `endpoints/*` functions.

## Server (`open4x-server`)

### Architecture

```
HTTP Server (Axum)
+-- GET /ws            -> WebSocket upgrade
+-- GET /health        -> Health check ("ok")
+-- GET /api/v1/*      -> REST API (bearer token auth)
+-- POST /api/v1/games/new -> bootstrap a single-player session (unauthenticated)
+-- Static files       -> Trunk-built frontend from OPEN4X_STATIC_DIR
```

The full REST route list lives in [`book/src/multiplayer/web-client.md`](../multiplayer/web-client.md). The shared route table is `open4x_server::server::rest::v1_router()` so `main.rs` and `tests/rest_api.rs` use one source of truth.

### State Management

```rust
struct AppState {
    games: DashMap<GameId, GameRoom>,           // concurrent map of active games
    players: DashMap<[u8; 32], PlayerRecord>,   // persistent player profiles
    api_tokens: DashMap<String, ApiTokenRecord>, // REST API bearer tokens
    templates: Vec<CivTemplate>,                // built-in civ definitions
}
```

Each `GameRoom` holds a full `GameState`, `DefaultRulesEngine`, player slots, AI agents, a notification ring buffer, and a broadcast channel for push updates.

### WebSocket Flow

1. **Authentication**: Server sends `Challenge { nonce }` -> client signs with Ed25519 private key -> server verifies -> `AuthSuccess { session_token, profile }`
2. **Lobby**: `ListGames` -> `GamesList`, `CreateGame` -> `GameCreated`, `JoinGame` -> `GameJoined { view }`
3. **Gameplay**: `Action(GameAction)` -> `ActionResult { ok, error }`, `EndTurn` -> `TurnResolved { new_turn, view }`

The single-player REST loop never opens a WebSocket; `/ws` is reserved for the AI demo and future multiplayer.

### Fog-of-War Projection

The `server/web_projection.rs` module converts internal `GameState` into per-endpoint wire slices:
- Only explored tiles appear in the board view
- Only visible units are included
- Own cities show full detail; foreign cities show limited info
- The player's own civ shows full research/government/yield detail; others show public summary only

### Deployment

The server is containerized with a multi-stage Dockerfile:
1. Build `open4x-server` binary (native, no feature flags)
2. Build the WASM frontend with `trunk build --release` from `open4x-client-web/`
3. Package into Debian slim runtime image; set `OPEN4X_STATIC_DIR` to the bundle path

Exposes port `3001`. Persistent game data stored in a Docker volume at `/app/data`.

## Frontend (`open4x-client-web`)

### Tab-Based UI

The game interface uses a tab system instead of a sidebar overlay:

| Tab | Description |
|-----|-------------|
| Map | Hex viewport with tile/unit info sidebar |
| Data Reports | Sub-tabs: Cities, Resources, Units, Map Stats |
| Science | Tech tree grid with status coloring and click-to-research |
| Culture | Civic tree grid with inspiration tracking |
| Governors | Governor management (placeholder) |
| Great People | Great person tracking (placeholder) |
| Climate | Climate monitoring (placeholder) |
| Players | Opponent data with diplomacy status |
| City | Individual city management with production queue |

### WebSocket Client

```rust
struct WsClient {
    socket: Rc<WebSocket>,
}
```

The client connects to the server's `/ws` endpoint, performs Ed25519 authentication, and exchanges `ClientMessage`/`ServerMessage` JSON frames. Used for the AI demo / future multiplayer; the single-player loop uses REST via `open4x-sdk` instead.

### SDK Adapter

`open4x-client-web/src/pages/rest_game.rs` defines a tiny `client(token)` helper that builds a `WasmClient` rooted at `""` (fetch resolves paths against `window.location`) and folds in the bearer token. Every typed call goes through `open4x_sdk::endpoints::*`.

### Hex Map Renderer

The `components/hexmap.rs` module renders the game board as SVG:
- Pointy-top hexagons colored by terrain type
- Click handlers for tile and unit selection
- Movement and attack interactions

### Build

```bash
cd open4x-client-web
trunk build --release        # emits dist/
```

Trunk auto-discovers the cdylib via the `<link data-trunk rel="rust">` hint in `index.html`. `getrandom`'s `wasm_js` feature is pinned in `open4x-sdk`'s wasm32 dependency block so the transitive `ulid → rand → getrandom` chain compiles for wasm without any workspace-level rustflag.
