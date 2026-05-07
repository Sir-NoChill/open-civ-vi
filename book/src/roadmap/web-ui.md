# Web UI Roadmap — Leptos port of the `open4x-webui/` wireframe

> **Status**: Server-side reads + writes for all 5 phases complete (40
> endpoints, 10 integration tests, 5 unit tests, both ssr/csr clippy clean).
> Remaining: client tabs (city/units/diplomacy/empire/victory/government),
> HexMap refactor, libciv extensions (preview_combat, available_unit_actions,
> pending_actions, victory_progress, policy_catalogue, AssignCityFocus,
> RenameCity, Cancel*, ChangeGovernment), NotificationRecord ring buffer,
> and Phase 5 final cleanup (delete `open4x-webui/`, drop `/api/game/*`).
> **Goal**: Replace the static `open4x-webui/` HTML wireframe with an interactive
> Leptos/WASM frontend that lives in `open4x-server` and is driven by REST calls
> against the same crate's `ssr` server. Single-player, declarative state on the
> client, mutations go through REST.

---

## 1. Why this exists

We have two artefacts today:

| Artefact                          | What it is                                                                                                            |
|-----------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| `open4x-webui/4X Wireframes.html` | A 3 624-line vanilla-JS wireframe with 14 screens that already render from local JSON mocks (`*.json` next to it).    |
| `open4x-server/src/{pages,components,tabs}` | A working Leptos/CSR app with a HexMap, ~9 tabs, WebSocket auth + a `GameView` projection. Functional but minimal. |

The wireframe is closer to the *visual* target; the Leptos app is closer to the
*technical* target. The plan below walks the wireframe's screens into the
Leptos app one at a time, while turning every `fetch("foo.json")` call into a
typed Rust `GET`/`POST` against `open4x-server`'s REST surface.

The end state must satisfy the user's brief:

1. **Rust-based web framework, minimal JS** — Leptos CSR (already in
   `open4x-server` with the `csr` feature). No new JS files; the wireframe's
   `wireframe.js` / `government.js` / `unit-screen.js` / `map-overlay.js` get
   ported into Leptos components.
2. **Declarative, single-player gameplay loop** — the client holds a snapshot
   of `GameView` (or smaller per-screen views), renders it, and only mutates by
   sending a REST request and replacing its snapshot with the response.
3. **REST as the mutation channel** — `GET /api/v1/...` for reads, `POST/PUT/
   PATCH/DELETE /api/v1/...` for writes. The existing WebSocket is kept for
   AI-demo / future multiplayer but the core single-player loop does not need
   it. (See §6 for the rationale.)

---

## 2. Inventory of what exists

### 2.1 Wireframe screens (target visual surface)

Pulled from `open4x-webui/HANDOFF.md`. Each row is one screen the Leptos app
must end up reproducing.

| Wireframe screen        | Mock file                    | Contains                                                |
|-------------------------|------------------------------|----------------------------------------------------------|
| HUD resource bar        | `player-state.json`          | gold/sci/cul/faith/food/prod, era, happiness, strategics |
| Hex map (HUD)           | `world-snapshot.json`        | sparse tiles around camera, edges, units, cities         |
| Map overlays drawer     | `map-overlays.json`          | overlay toggles, minimap markers                         |
| Unit/Army screen        | `unit-data.json`, `army-data.json` | unit list, promotions, action buttons, combat preview |
| City management         | `city-data.json`, `city-tiles.json` | districts, buildings, prod queue, citizens, tile map  |
| Notifications drawer    | `notifications.json`         | turn-event feed                                          |
| Turn queue drawer       | `turn-queue.json`            | required + optional pending actions                      |
| Tech tree               | `tech-tree.json`             | full tree with prereq lines, status                      |
| Civics tree             | `civics-tree.json`           | full tree with policy unlocks                            |
| Government screen       | `government-policies.json`   | gov form, slot management, full policy catalogue         |
| Diplomacy               | `diplomacy.json`             | civs, city-states, deal draft                            |
| Empire overview         | `empire-overview.json`       | summary stats, city table, trade, religion, sparkline    |
| Victory screen          | `victory.json`               | per-condition progress, leaderboard                      |

### 2.2 Existing Leptos coverage (`open4x-server` with `csr`)

What we already have we keep:

- **`pages/mod.rs`** — `HomePage`, `MapConfigPage`, `SettingsPage`, `PlayersPage`,
  `DemoConfigPage` (all functional, just need to be reachable from the new HUD).
- **`pages/game.rs`** — `GamePage` with TopBar, Sidebar, TileInfo, CityPanel,
  UnitInfo, YieldsPanel, TechPanel. Currently WebSocket-driven.
- **`pages/replay.rs`** — AI-demo replay viewer (independent, leave alone).
- **`components/hexmap.rs`** — SVG hex renderer. Already does selection and
  click dispatch. **This is the foundation we build the new HUD around.**
- **`components/{ws,session,client_auth}.rs`** — WebSocket + Ed25519 auth. Used
  by the AI-demo flow; will remain as the multiplayer transport.
- **`tabs/`** — `GameTab` enum + 9 tabs (Map, DataReports w/ 4 sub-tabs,
  Science, Culture, Governors, GreatPeople, Climate, Players, City). Several
  are placeholders.
- **`types/{view,messages,enums,ids,coord}.rs`** — already serializable types
  shared by `ssr` and `csr`. `GameView` is the canonical client-facing shape.

### 2.3 Existing REST surface (`/api/game/*`)

From `open4x-server/src/main.rs` + `src/server/api.rs`. All require
`Authorization: Bearer <token>` and resolve `(GameId, CivId)` from the token.

| Method | Path                   | Returns                       |
|--------|------------------------|-------------------------------|
| GET    | `/api/game/view`       | full `GameView`               |
| GET    | `/api/game/cities`     | `Vec<CityReportRow>`          |
| GET    | `/api/game/city/{id}`  | full `CityReport`             |
| GET    | `/api/game/resources`  | `ResourceReport`              |
| GET    | `/api/game/units`      | `UnitReport`                  |
| GET    | `/api/game/map-stats`  | terrain/feature/resource counts |
| GET    | `/api/game/players`    | `Vec<PlayerReport>`           |
| GET    | `/api/game/science`    | `ScienceReport`               |
| GET    | `/api/game/culture`    | `CultureReport`               |
| GET    | `/api/game/turn`       | `TurnStatus`                  |

All of these are read-only. **There is no REST mutation surface today.** The
existing WS protocol (see `types/messages.rs::ClientMessage`) covers all the
mutations we need (`GameAction` enum: MoveUnit, FoundCity, QueueProduction,
QueueResearch, AssignPolicy, etc.). Phase 2 below lifts those into REST POSTs.

---

## 3. End-state architecture

```
┌─ Browser (WASM, Leptos CSR) ─────────────────────────────────────┐
│                                                                  │
│   PageRouter                                                     │
│     ├─ HomePage / MapConfigPage / SettingsPage / DemoConfigPage  │
│     └─ GamePage   ◄── owns one Resource<GameSnapshot>            │
│           │                                                      │
│           ├─ TopBar  (player-state slice)                        │
│           ├─ HexMap  (world slice + selection signals)           │
│           ├─ Sidebar (selected tile/unit/city info)              │
│           ├─ TabBar  ──► swaps tab body                          │
│           └─ Drawers (Notifications, TurnQueue, MapOverlays,     │
│                       Research, Civics)                          │
│                                                                  │
│   Every mutation: api::action::*  →  POST /api/v1/...            │
│                   on success: snapshot.refetch()                 │
│                                                                  │
└─────────────────────────┬────────────────────────────────────────┘
                          │ HTTP, JSON, Bearer token
┌─────────────────────────▼────────────────────────────────────────┐
│ open4x-server (axum, ssr)                                        │
│                                                                  │
│   /api/v1/*  ──►  rest::handlers      ──► GameRoom (GameState)   │
│   /ws        ──►  websocket (multi/AI demo, unchanged)           │
│   /          ──►  static ServeDir → trunk-built WASM bundle      │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

Single source of truth: `GameRoom.state: GameState` in
`server/state.rs`. REST handlers project per-screen views from it (existing
`projection::project_game_view`) and apply mutations through `RulesEngine`,
returning the freshly-projected slice.

---

## 4. REST API specification

We harmonise three surfaces into one:

- the wireframe's `api-manifest.json` (what the JS expects today),
- the existing `/api/game/*` endpoints (what we already serve),
- the WS `GameAction` variants (what `RulesEngine` already accepts).

The chosen base path is **`/api/v1`** (the existing `/api/game/*` paths are
kept as deprecated aliases during the transition; remove in Phase 5). All
endpoints require `Authorization: Bearer <token>`. All bodies are JSON.

### 4.1 Reads (return existing typed views)

Every read endpoint returns a slice of `GameView` (already serializable). The
wireframe's `*/v1` schemas can be implemented as serde renames or as thin
wrapper structs in a new `types/web.rs` module — see §5.1.

| Method | Path                          | Returns                                                  | Maps to                                      |
|--------|-------------------------------|----------------------------------------------------------|----------------------------------------------|
| GET    | `/api/v1/player-state`        | resources, turn, era, happiness, strategics              | `GameView.{turn, my_civ.{yields, gold, faith, current_era, strategic_resources}}` |
| GET    | `/api/v1/world/snapshot`      | board metadata + tiles (sparse around `?q&r&radius`)     | `GameView.board` (filtered)                  |
| GET    | `/api/v1/world/tile/:q/:r`    | single tile detail                                       | `GameView.board.tiles.iter().find(...)`      |
| GET    | `/api/v1/map/overlays`        | overlay toggle state, minimap markers                    | new — server-derived from `GameView`         |
| GET    | `/api/v1/units`               | full unit list with action availability                  | `GameView.units` + `RulesEngine::available_unit_actions` (new, see §5.2) |
| GET    | `/api/v1/units/:id`           | single unit detail                                       | filter `GameView.units`                      |
| GET    | `/api/v1/armies`              | army/corps formations + cohesion                         | new — driven by future `Army` system; for now returns `[]` |
| GET    | `/api/v1/combat/preview`      | combat odds. Query: `attacker_id`, `defender_q`, `defender_r` | `RulesEngine::preview_combat` (new helper, see §5.2) |
| GET    | `/api/v1/cities`              | all cities with yields, queue, loyalty                   | `GameView.cities` projected                  |
| GET    | `/api/v1/cities/:id`          | single city full detail                                  | `GameView.cities.iter().find(...)`           |
| GET    | `/api/v1/cities/:id/tiles`    | city tile ownership + work map                           | `City.{territory, worked_tiles, locked_tiles}` |
| GET    | `/api/v1/tech`                | full tech tree + research queue                          | `GameView.{tech_tree, my_civ.{researched_techs, research_queue}}` |
| GET    | `/api/v1/civics`              | civics tree + civic queue                                | `GameView.{civic_tree, my_civ.{completed_civics, civic_in_progress}}` |
| GET    | `/api/v1/government`          | current government, slots, active policies, catalogue    | `GameView.my_civ.{current_government, active_policies}` + new policy catalogue (see §5.2) |
| GET    | `/api/v1/notifications`       | event feed                                               | new — `GameRoom.notifications` queue (see §5.3) |
| GET    | `/api/v1/turn-queue`          | required + skippable pending actions                     | new — `RulesEngine::pending_actions(civ)` (see §5.2) |
| GET    | `/api/v1/diplomacy`           | known civs, city-states, deal draft                      | `GameView.other_civs` + diplomacy projection (new) |
| GET    | `/api/v1/diplomacy/civs/:id`  | single civ detail with modifiers                         | filter `GameView.other_civs`                 |
| GET    | `/api/v1/empire/overview`     | dashboard data                                           | aggregated from `GameView` (new projector)   |
| GET    | `/api/v1/victory`             | per-condition progress + leaderboard                     | new — `RulesEngine::victory_progress`        |

### 4.2 Mutations (each maps 1:1 to a `GameAction` variant)

Every successful mutation returns the relevant freshly-projected slice **plus
a `turn_status` block** so the client can refresh the HUD bar without a
second round-trip:

```json
{ "ok": true, "view": { ... slice ... }, "turn_status": { "turn": 142, "ended": false } }
```

| Method | Path                                          | Body                                                | Maps to `GameAction`                |
|--------|-----------------------------------------------|-----------------------------------------------------|-------------------------------------|
| POST   | `/api/v1/turn/end`                            | `{}`                                                | `EndTurn`                           |
| POST   | `/api/v1/units/:id/action`                    | `{action_id, target_q?, target_r?}`                 | dispatched per `action_id` to one of `MoveUnit`, `Attack`, `FoundCity`, `PlaceImprovement`, etc. |
| POST   | `/api/v1/cities/:id/production`               | `{item_id, item_type: "unit"|"building"|"wonder"}`  | `QueueProduction`                   |
| DELETE | `/api/v1/cities/:id/production/:pos`          | —                                                   | `CancelProduction`                  |
| POST   | `/api/v1/cities/:id/citizens`                 | `{focus: "default"|"food"|"prod"|"gold"|"sci"}`     | new, see §5.2 (`AssignCityFocus`)   |
| PATCH  | `/api/v1/cities/:id/rename`                   | `{name: string}`                                    | new, see §5.2 (`RenameCity`)        |
| POST   | `/api/v1/tech/research`                       | `{tech_id}`                                         | `QueueResearch` (replace front)     |
| POST   | `/api/v1/tech/queue`                          | `{tech_id}`                                         | `QueueResearch` (append)            |
| DELETE | `/api/v1/tech/queue/:tech_id`                 | —                                                   | new, see §5.2 (`CancelResearch`)    |
| POST   | `/api/v1/civics/research`                     | `{civic_id}`                                        | `QueueCivic` (replace front)        |
| POST   | `/api/v1/civics/queue`                        | `{civic_id}`                                        | `QueueCivic` (append)               |
| DELETE | `/api/v1/civics/queue/:civic_id`              | —                                                   | new, see §5.2 (`CancelCivic`)       |
| POST   | `/api/v1/government/change`                   | `{government_id}`                                   | new, see §5.2 (`ChangeGovernment`)  |
| PUT    | `/api/v1/government/policies`                 | `{policies: [{slot, policy_id}]}`                   | bulk `AssignPolicy`                 |
| POST   | `/api/v1/diplomacy/civs/:id/action`           | `{action: "declare_war"|"make_peace"|...}`          | `DeclareWar` / `MakePeace` / new diplomacy actions |
| PUT    | `/api/v1/diplomacy/deal`                      | `{civ_id, you_give[], they_give[], duration_turns}` | new, see §5.2 (`UpdateDealDraft`)   |
| POST   | `/api/v1/diplomacy/deal/propose`              | —                                                   | new (`ProposeDeal`)                 |
| POST   | `/api/v1/map/overlays/:id/toggle`             | `{active: bool}`                                    | client-only state, persisted in `localStorage` (see §6.4) |
| DELETE | `/api/v1/notifications/:id`                   | —                                                   | new (`DismissNotification`)         |
| DELETE | `/api/v1/notifications`                       | —                                                   | new (`DismissAllNotifications`)     |
| POST   | `/api/v1/turn-queue/:id/skip`                 | `{}`                                                | new (`SkipQueueItem`)               |
| POST   | `/api/v1/armies`                              | `{name, unit_ids[]}`                                | new (`FormArmy`) — Phase 4         |

### 4.3 Error model

Standard for every endpoint:

```
2xx  → typed JSON (per table above)
400  → { "error": "<machine_code>", "message": "<human>", ... }
401  → { "error": "missing_or_invalid_token" }
404  → { "error": "not_found" }
409  → { "error": "rule_violation", "rule": "<RulesEngine error variant>" }
```

The end-turn endpoint is the one place where 400 carries structured data:

```
POST /api/v1/turn/end
  → 400 { "error": "unresolved_required_actions", "items": [...] }
```

### 4.4 What stays on WebSocket

The `/ws` endpoint and the entire `ClientMessage`/`ServerMessage` enum stay
untouched. They are used by:

- the existing **AI demo** (`pages/replay.rs`),
- future hot-seat / multiplayer (out of scope here).

The single-player loop never opens a WebSocket. This is the simplification the
brief asks for.

---

## 5. Server-side work

### 5.1 New shared types in `types/web.rs`

The wireframe ships its own JSON schemas (`player-state/v1`,
`world-snapshot/v1`, etc.). We mirror those shapes verbatim — not the existing
`reports::*` types — so the wireframe HTML can be retired without rewriting
its data semantics.

```rust
// types/web.rs (compiles for both ssr and csr)
pub mod player_state {
    pub struct PlayerState { turn, turn_max, era, era_progress,
                             resources: Resources, happiness, strategic: Map<String, u32> }
    pub struct Resources { gold, science, culture, faith, food, production }   // each Bucket{value:Option<i32>, per_turn:i32}
}
pub mod world {
    pub struct WorldSnapshot { world: WorldMeta, camera: Camera, legend: Legend, tiles: Vec<TileView> }
    pub struct TileView { q, r, terrain, yields, appeal, flood, fog, owner,
                          city: Option<TileCity>, unit: Option<TileUnit>,
                          improvement, resource, edges: HashMap<HexDir, Vec<EdgeKind>> }
}
pub mod tech_tree { pub struct TechTreeView { techs: Vec<TechNode>, research_queue: Vec<String> } }
// … one module per wireframe schema (notifications, turn_queue, government, diplomacy,
//   empire_overview, victory, city_data, city_tiles, unit_data, army_data, map_overlays).
```

These types are the **wire** types. They are derived from `GameView` by
projector functions (next section), not stored directly.

### 5.2 New projectors in `server/web_projection.rs`

One pure function per endpoint, of the shape:

```rust
pub fn build_player_state(view: &GameView, room: &GameRoom) -> PlayerState;
pub fn build_world_snapshot(view: &GameView, q: i32, r: i32, radius: u32) -> WorldSnapshot;
pub fn build_units(view: &GameView, rules: &DefaultRulesEngine, gs: &GameState, civ: CivId) -> UnitData;
pub fn build_cities(view: &GameView) -> CityData;
pub fn build_city_tiles(view: &GameView, city: CityId) -> CityTiles;
pub fn build_tech_tree(view: &GameView) -> TechTreeView;
pub fn build_civics_tree(view: &GameView) -> CivicsTreeView;
pub fn build_government(view: &GameView, rules: &DefaultRulesEngine) -> GovernmentPolicies;
pub fn build_diplomacy(view: &GameView, gs: &GameState, civ: CivId) -> Diplomacy;
pub fn build_empire_overview(view: &GameView, gs: &GameState, civ: CivId) -> EmpireOverview;
pub fn build_victory(view: &GameView, gs: &GameState, rules: &DefaultRulesEngine) -> Victory;
pub fn build_notifications(room: &GameRoom, civ: CivId) -> Notifications;
pub fn build_turn_queue(view: &GameView, gs: &GameState, rules: &DefaultRulesEngine, civ: CivId) -> TurnQueue;
pub fn build_map_overlays(view: &GameView) -> MapOverlays;
```

Each delegates to `RulesEngine` for derived numbers (yields, combat
preview, eligible policies) so all rule logic stays in `libciv`.

### 5.3 New libciv extensions

A handful of `RulesEngine` methods need to exist before the projectors can
return real data. Each is a separate, small libciv PR — none are gated by
each other.

| New `RulesEngine` method                                       | Drives                                  |
|----------------------------------------------------------------|-----------------------------------------|
| `available_unit_actions(&self, gs, unit) -> Vec<UnitAction>`   | `unit-data.json#actions`                |
| `preview_combat(&self, gs, attacker, defender_coord) -> CombatPreview` | `/api/v1/combat/preview`        |
| `pending_actions(&self, gs, civ) -> Vec<TurnQueueItem>`        | `/api/v1/turn-queue`                    |
| `victory_progress(&self, gs) -> Vec<VictoryProgress>`          | `/api/v1/victory`                       |
| `policy_catalogue(&self, gs, civ) -> Vec<PolicyCardEntry>`     | `/api/v1/government`                    |

A handful of new `GameAction` variants on top:

| New `GameAction`                       | Server route                                     |
|----------------------------------------|--------------------------------------------------|
| `AssignCityFocus { city, focus }`      | `POST /api/v1/cities/:id/citizens`               |
| `RenameCity { city, name }`            | `PATCH /api/v1/cities/:id/rename`                |
| `CancelResearch { tech }`              | `DELETE /api/v1/tech/queue/:tech_id`             |
| `CancelCivic { civic }`                | `DELETE /api/v1/civics/queue/:civic_id`          |
| `ChangeGovernment { government }`      | `POST /api/v1/government/change`                 |
| `UpdateDealDraft { civ, draft }`       | `PUT /api/v1/diplomacy/deal`                     |
| `ProposeDeal { civ }`                  | `POST /api/v1/diplomacy/deal/propose`            |
| `DismissNotification { id }`           | `DELETE /api/v1/notifications/:id`               |
| `DismissAllNotifications`              | `DELETE /api/v1/notifications`                   |
| `SkipQueueItem { id }`                 | `POST /api/v1/turn-queue/:id/skip`               |
| `FormArmy { name, units }`             | `POST /api/v1/armies`                            |

A new server-only ring buffer of notifications needs to live on `GameRoom`:

```rust
// server/state.rs additions
pub struct GameRoom {
    // ...existing fields...
    pub notifications: VecDeque<NotificationRecord>, // max ~64, oldest evicted
    pub map_overlay_prefs: HashMap<CivId, OverlayPrefs>, // per-player toggles
}
```

`NotificationRecord` is generated whenever `RulesEngine::advance_turn` returns
deltas the user should see (research complete, unit defeated, city captured,
etc.). It is built from `Vec<StateDelta>`.

### 5.4 New router wiring (`main.rs`)

```rust
let v1 = Router::new()
    // reads
    .route("/player-state", get(rest::player_state))
    .route("/world/snapshot", get(rest::world_snapshot))
    .route("/world/tile/{q}/{r}", get(rest::world_tile))
    .route("/map/overlays", get(rest::map_overlays))
    .route("/units", get(rest::units))
    .route("/units/{id}", get(rest::unit_detail))
    .route("/armies", get(rest::armies))
    .route("/combat/preview", get(rest::combat_preview))
    .route("/cities", get(rest::cities))
    .route("/cities/{id}", get(rest::city_detail))
    .route("/cities/{id}/tiles", get(rest::city_tiles))
    .route("/tech", get(rest::tech))
    .route("/civics", get(rest::civics))
    .route("/government", get(rest::government))
    .route("/notifications", get(rest::notifications))
    .route("/turn-queue", get(rest::turn_queue))
    .route("/diplomacy", get(rest::diplomacy))
    .route("/diplomacy/civs/{id}", get(rest::diplomacy_civ))
    .route("/empire/overview", get(rest::empire_overview))
    .route("/victory", get(rest::victory))
    // writes
    .route("/turn/end", post(rest::end_turn))
    .route("/units/{id}/action", post(rest::unit_action))
    .route("/cities/{id}/production", post(rest::queue_production))
    .route("/cities/{id}/production/{pos}", delete(rest::cancel_production))
    .route("/cities/{id}/citizens", post(rest::assign_focus))
    .route("/cities/{id}/rename", patch(rest::rename_city))
    .route("/tech/research", post(rest::set_research))
    .route("/tech/queue", post(rest::queue_research))
    .route("/tech/queue/{tech_id}", delete(rest::cancel_research))
    .route("/civics/research", post(rest::set_civic))
    .route("/civics/queue", post(rest::queue_civic))
    .route("/civics/queue/{civic_id}", delete(rest::cancel_civic))
    .route("/government/change", post(rest::change_government))
    .route("/government/policies", put(rest::set_policies))
    .route("/diplomacy/civs/{id}/action", post(rest::diplomacy_action))
    .route("/diplomacy/deal", put(rest::update_deal))
    .route("/diplomacy/deal/propose", post(rest::propose_deal))
    .route("/map/overlays/{id}/toggle", post(rest::toggle_overlay))
    .route("/notifications/{id}", delete(rest::dismiss_notification))
    .route("/notifications", delete(rest::dismiss_all))
    .route("/turn-queue/{id}/skip", post(rest::skip_queue_item))
    .route("/armies", post(rest::form_army));

let app = Router::new()
    .nest("/api/v1", v1)
    .route("/ws", get(server::ws::ws_handler))
    .route("/health", get(|| async { "ok" }))
    .nest("/api/game", legacy_api_routes())   // deprecated; remove in Phase 5
    .fallback_service(ServeDir::new(&static_dir))
    .layer(CorsLayer::permissive())
    .with_state(state);
```

---

## 6. Client-side work (Leptos)

### 6.1 Bindings layer (`client/api/`)

A new module `open4x-server/src/components/api/` contains one Rust function
per REST endpoint. Reads return `Result<T, ApiError>`. Writes return
`Result<MutationResponse<T>, ApiError>` where `T` is the affected slice.

```rust
// components/api/mod.rs
pub mod world;
pub mod cities;
pub mod tech;
pub mod civics;
pub mod government;
pub mod units;
pub mod diplomacy;
pub mod empire;
pub mod victory;
pub mod notifications;
pub mod turn_queue;
pub mod map_overlays;
pub mod player_state;

// components/api/cities.rs (sample)
pub async fn get_cities(token: &str) -> Result<CityData, ApiError> { fetch_json("GET", "/api/v1/cities", token, None).await }
pub async fn queue_production(token: &str, city: CityId, body: QueueProductionBody)
    -> Result<MutationResponse<CityData>, ApiError> { fetch_json("POST", &format!("/api/v1/cities/{city}/production"), token, Some(body)).await }
```

A single `fetch_json` helper in `components/api/http.rs` wraps `web_sys::Fetch`,
attaches the bearer token, and decodes JSON via `serde-wasm-bindgen`. No JS.

### 6.2 Reactive snapshots

We do NOT keep `GameView` in one giant signal. Instead each screen owns a
Leptos `Resource` that fetches its slice:

```rust
let cities = Resource::new(refresh_token, |_| async { api::cities::get_cities(&token).await });
```

A single `RwSignal<u64>` `refresh_token` is bumped after every successful
mutation; that triggers the affected `Resource`s to refetch in parallel. Slices
are independent so a tech-tree action does not redownload the world snapshot.

### 6.3 Component map

This is the wireframe → Leptos translation table. New file paths under
`open4x-server/src/`.

| Wireframe screen        | New Leptos file                           | Replaces                          |
|-------------------------|-------------------------------------------|-----------------------------------|
| HUD resource bar        | `components/hud/topbar.rs`                | `pages/game.rs::TopBar`           |
| Hex map                 | (existing) `components/hexmap.rs`         | —                                 |
| Map overlays drawer     | `components/hud/overlays_drawer.rs`       | `map-overlay.js`                  |
| Notifications drawer    | `components/hud/notifications_drawer.rs`  | wireframe inline JS               |
| Turn queue drawer       | `components/hud/turn_queue_drawer.rs`     | wireframe inline JS               |
| Tile/unit info sidebar  | `components/hud/sidebar.rs`               | `pages/game.rs::Sidebar`          |
| Unit/Army screen        | `tabs/units.rs`                           | `unit-screen.js`                  |
| City management         | `tabs/city.rs` (extend existing)          | inline JS                         |
| Tech tree (full)        | `tabs/science.rs` (extend existing)       | inline JS                         |
| Civics tree             | `tabs/culture.rs` (extend existing)       | inline JS                         |
| Government screen       | `tabs/government.rs` **(new)**            | `government.js`                   |
| Diplomacy               | `tabs/diplomacy.rs` **(new)**             | inline JS                         |
| Empire overview         | `tabs/empire.rs` **(new)**                | inline JS                         |
| Victory                 | `tabs/victory.rs` **(new)**               | inline JS                         |

The placeholder tabs (`Governors`, `GreatPeople`, `Climate`) stay as
placeholders in this scope and are tracked separately in `todo.md`.

### 6.4 State that stays on the client

Per the brief: "the user-visible state being the only thing on the client
computer." That includes:

- camera position + zoom — `web_sys::Storage` (`localStorage`) keyed by game id;
- selected tile / selected unit — Leptos `RwSignal`;
- map overlay toggles — `localStorage` (also pushed to server via
  `POST /map/overlays/:id/toggle` so a future multiplayer/spectator can pick
  them up, but server is not the source of truth for this);
- which drawer is open — Leptos `RwSignal`.

Everything else round-trips through REST.

---

## 7. Phasing

Each phase ends in a working, mergeable state. No phase requires more than
one libciv PR + one open4x-server PR.

### Phase 0 — Scaffolding (prep, no behaviour change) ✅

1. [x] Add `types/web.rs` with empty modules for each wireframe schema
   (compiles for both `ssr` and `csr`).
2. [x] Add `server/web_projection.rs` with stub functions that return
   `Default`.
3. [x] Add `server/rest/` as a new module mirroring `server/api.rs` style.
   Wire `/api/v1/health` only.
4. [x] Add `components/api/` with `http.rs` and one stub call
   (`get_player_state`).
5. [x] Add `trunk` build instructions to `book/src/getting-started.md`.

**Done when**: `cargo build --workspace` clean and the server boots with both
`/api/game/*` (existing) and `/api/v1/health` reachable.

### Phase 1 — HUD MVP ✅

Goal: replace `pages/game.rs::TopBar` and `Sidebar` with REST-driven versions.
Scope is intentionally small to validate the bindings layer end-to-end.

1. [x] Implement `build_player_state` projector + `GET /api/v1/player-state`.
2. [x] Implement `build_world_snapshot` + `GET /api/v1/world/snapshot`.
3. [x] Implement `components/api/{player_state,world}.rs`.
4. [x] Rewrite the HUD topbar (in new `pages/rest_game.rs::RestGamePage`) to
   consume `LocalResource<PlayerState>`. Existing `pages/game.rs` (WS-based)
   left intact so the AI demo / future MP path keeps working.
5. [~] HexMap re-bound: deferred to Phase 2 — the existing `components/hexmap.rs`
   is tightly coupled to `GameView`; the Phase 1 page renders a placeholder
   showing tile counts to prove the snapshot fetch loop works. Phase 2 will
   refactor `hexmap` to take a `WorldSnapshot` directly.
6. [x] Wire `POST /api/v1/turn/end` and the End Turn button.

**Bonus** (not in original Phase 1 list, needed for the loop to actually run):

7. [x] `POST /api/v1/games/new` — bootstraps a single-player session and mints
   a bearer token. Without this the new HUD has nothing to authenticate
   against. Marked as a Phase 1 extension in the changelog.
8. [x] WASM entry point — `lib.rs::start()` (gated on `csr`) calls
   `console_error_panic_hook::set_once()` and mounts `RestGamePage` to
   `<body>`. The Leptos crate had components but no actual `start` shim
   before this; `trunk serve` would previously have produced an empty page.

**Done when**: starting a single-player game shows the HUD bar populated from
REST and clicking End Turn advances the turn without using the WebSocket. ✅
Verified via `curl` against a live `target/release/open4x-server`.

### Phase 2 — Cities + Units (the core gameplay loop)

1. [x] Projectors: `build_cities`, `build_city_tiles`, `build_units`.
2. [ ] New `RulesEngine` methods: `available_unit_actions`, `preview_combat`
   (current `build_combat_preview` is a heuristic stub flagged in its `note`
   field — replace with libciv-driven version).
3. [ ] New `GameAction` variants: `AssignCityFocus`, `RenameCity`.
4. [x] REST reads for `/api/v1/cities/*`, `/api/v1/units/*`,
   `/api/v1/combat/preview`. ✅
5. [x] REST writes: `POST /cities/{id}/production`,
   `DELETE /cities/{id}/production/{pos}`, `POST /units/{id}/action`
   (move, attack, found_city; fortify/sleep stubbed). ✅ smoke-tested.
6. [ ] Port `tabs/city.rs` into the new shape.
7. [ ] Add `tabs/units.rs`.
8. [ ] Add `components/hud/sidebar.rs`.

**Done when**: a player can found cities, queue production, work tiles, move
units, and engage combat purely through REST. WebSocket is unused for
single-player.

### Phase 3 — Research & policy stacks

1. [x] Projectors: `build_tech_tree`, `build_civics_tree`, `build_government`
   (catalogue still empty; needs libciv exposure).
2. [x] REST reads `GET /tech`, `GET /civics`, `GET /government`. ✅
3. [x] REST mutations `POST /tech/research`, `POST /civics/research`. ✅
4. [ ] New `GameAction` variants: `CancelResearch`, `CancelCivic`,
   `ChangeGovernment`. Bulk policy assignment via existing `AssignPolicy`
   applied N times in one route handler.
5. [ ] Port `tabs/science.rs` and `tabs/culture.rs` to render full trees
   with prereq lines (the wireframe's CSS gradients can be reused — copy
   them into `index.html` `<style>`).
6. [ ] Add `tabs/government.rs` with slot management + catalogue browser.

**Done when**: the player can research a tech, complete a civic, change
government, slot policies, and confirm — all through REST.

### Phase 4 — Outer loop screens

1. [x] Projectors: `build_diplomacy`, `build_empire_overview`, `build_victory`,
   `build_notifications`, `build_turn_queue`, `build_map_overlays`. ✅
2. [ ] `RulesEngine::pending_actions`, `RulesEngine::victory_progress`
   (current `build_turn_queue` and `build_victory` are pure projections;
   neither calls into libciv yet — the conditions array uses a static
   placeholder list of the 6 builtin victories with `player_pct = 0`.)
3. [ ] `NotificationRecord` ring buffer in `GameRoom` populated from
   `advance_turn` deltas. `/notifications` currently returns an empty list.
4. [ ] New tabs: `tabs/diplomacy.rs`, `tabs/empire.rs`, `tabs/victory.rs`.
5. [ ] Drawers: `components/hud/{notifications,turn_queue,overlays}_drawer.rs`.
6. [x] Block End Turn when `turn-queue` has `required: true` items —
   `POST /api/v1/turn/end` returns 400 with the structured payload. ✅

**Done when**: the wireframe's HUD drawers and the four outer-loop screens
all render real data and accept user input.

### Phase 5 — Cleanup

1. [ ] Delete `open4x-webui/` (or move to `docs/legacy-wireframe/` if we
   want to keep it as a visual reference).
2. [ ] Remove the `/api/game/*` legacy routes and `server/reports.rs` if
   nothing else consumes them. (Currently the integration tests still
   read `/api/game/view` to look up unit-type IDs — gated on a new
   `/api/v1/registry` endpoint that should expose the same data.)
3. [ ] Remove `/ws` from the single-player code path entirely. (Keep it
   for AI demo and for the future multiplayer roadmap.)
4. [x] Add integration tests under `open4x-server/tests/rest_api.rs`
   that spin up `AppState`, mint a token, and exercise each route. ✅
   10 tests covering health/auth/games_new/player-state/world-snapshot/
   cities/units/diplomacy/turn-queue/victory/end-turn/research/
   production round-trip. All green.
5. [ ] Document the API in `book/src/multiplayer/web-client.md`.

**Done when**: `cargo test -p open4x-server --features ssr` covers every
endpoint and the wireframe directory is gone.

---

## 8. Open questions

- **Tile coordinates.** The wireframe uses axial `(q, r)` and treats `s` as
  derived. Our `HexCoord` is cube `(q, r, s)`. We strip `s` on the wire and
  recompute it server-side. (Already what `types/coord.rs` essentially does;
  just needs to be the documented convention.)
- **Pagination for /world/snapshot.** Wireframe loads ~30 tiles. Big maps need
  bounded responses. Phase 1 ships with a hard `radius` cap of 32 and
  paginates by re-fetching as the camera moves. Revisit if profiling shows
  it's a problem.
- **Notifications backfill on join.** When a player rejoins a game mid-turn,
  do they see historical notifications? Phase 4 default: only notifications
  for the *current* turn are kept. Older ones are lost.
- **Trunk vs. wasm-pack.** The existing crate already uses `trunk`'s
  `index.html`. We stick with trunk; the build command becomes
  `trunk build --release --features csr` and the artefacts go in
  `open4x-server/dist/` (already what `OPEN4X_STATIC_DIR` defaults to).
- **CSS source of truth.** The wireframe's `styles.css` is comprehensive and
  matches the existing `index.html` `<style>` block roughly 60%. We copy the
  wireframe's CSS into `index.html` and delete the duplicates as components
  are ported. By Phase 5 there is one CSS source of truth.

---

## 9. What this plan deliberately does NOT do

- No multiplayer changes. The WS protocol stays as is.
- No new game systems (armies, governors, climate, great-people stubs stay
  stubs; their tabs remain placeholders until those systems land in libciv).
- No save/load over REST yet. Save remains a JSON download in the topbar.
- No accessibility or i18n pass — that is a separate roadmap item.
- No mobile layout — the wireframe is desktop-only and so is the port.

---

## 10. Changelog

Running record of work performed against this plan, newest at top.

### Phase 5 partial — REST integration tests (2026-05-07)

- **Commit `feat(open4x-server): RestGamePage surfaces full API state (Phase 1+ client)`**:
  expanded the demo HUD to read every Phase 2-4 slice (cities, units,
  tech, turn queue) and added a "Pick Research" helper button. Cleared
  the 3 pre-existing csr clippy warnings as a drive-by.
- **Commit `feat(open4x-server): integration tests for /api/v1 surface (Phase 5 prep)`**:
  - Extracted the `/api/v1` route table into
    `server::rest::v1_router()` so `main.rs` and the test rig share one
    source of truth. `main.rs` now `.nest("/api/v1", v1_router())`.
  - Added `open4x-server/tests/rest_api.rs` with 10 in-process
    integration tests that hit the router via `tower::ServiceExt::oneshot`
    (no TCP listener, no port, parallel-safe). Coverage:
    - `health_is_unauthenticated`
    - `unauthenticated_endpoint_rejects_missing_token` (401 path)
    - `games_new_mints_token_and_player_state_uses_it`
    - `world_snapshot_returns_tiles_and_dimensions`
    - `cities_units_and_diplomacy_round_trip`
    - `end_turn_blocks_when_required_action_pending` (400 + structured
      `unresolved_required_actions` body)
    - `end_turn_advances_after_research_chosen` (200 + turn 0 → 1)
    - `production_queue_and_cancel_round_trip` (POST + DELETE)
    - `turn_queue_lists_required_choose_research_on_fresh_game`
    - `victory_includes_six_conditions_and_ranks_player`
  - `tower`, `http-body-util`, and `hex` added as `[dev-dependencies]`.
  - **Note**: `production_queue_and_cancel_round_trip` still pulls a
    unit-type id from the legacy `/api/game/view` endpoint because
    `/api/v1` doesn't yet expose `unit_type_defs` directly. Phase 5 task
    2 (drop legacy `/api/game/*`) is gated on adding a
    `GET /api/v1/registry` endpoint that exposes the unit/building
    catalogues — flagged in the changelog.

### Phase 4 — End Turn 400 enforcement (2026-05-07)

- **Commit `feat(open4x-server): enforce required turn-queue items on POST /turn/end (Phase 4 polish)`**:
  - The end-turn handler now projects the turn queue *before* mutating
    state. Any item with `required: true` causes a 400 with the body
    `{"error":"unresolved_required_actions","items":[...]}` exactly as
    §4.3 specifies. The previous implementation packed the items into
    the generic `ApiErrorBody.message` field, which the wireframe
    contract didn't expect.
  - Smoke-tested:
    - Fresh game → `POST /turn/end` → 400 with the
      `choose_research` item (no tech queued yet).
    - After `POST /tech/research` → 200 OK, `turn: 1`.

### Phase 4 — Outer-loop reads (2026-05-07)

- **Commit `feat(open4x-server): outer-loop reads (Phase 4 server)`**:
  - `build_diplomacy` projects every known civ into `{id, name, leader,
    relation, cities_known}` rows. `relation` maps `DiplomaticStatus` →
    "At War / Unfriendly / Neutral / Friendly / Allied". Modifiers and
    treaties stay empty until libciv exposes them.
  - `build_empire_overview` aggregates the player's cities, total
    population, treasury (current + per-turn from
    `my_civ.yields.gold`), military unit count (own combat units), and
    the per-city table.
  - `build_victory` builds a leaderboard from `view.scores`, sets the
    player's rank, and emits the 6 builtin victory conditions with
    placeholder `player_pct = 0` (real progress lands when libciv ships
    `RulesEngine::victory_progress`).
  - `build_turn_queue` derives required vs. skippable items from the
    player view: a required `choose_research` if the queue is empty, plus
    one optional row per own unit with movement remaining. Per the plan's
    §4.3 contract, the End Turn handler will refuse to advance with any
    `required: true` item — the libciv-backed enforcement comes once
    `RulesEngine::pending_actions` lands; for now the heuristic is server
    side.
  - `build_notifications` returns an empty list; `NotificationRecord`
    ring buffer + `advance_turn` delta wiring is a follow-up.
  - `build_map_overlays` returns a fixed list of toggleable overlays
    (yields, appeal, religion, borders, resources). Toggle persistence is
    client-side only (per §6.4).
  - REST handlers: `GET /diplomacy`, `GET /diplomacy/civs/{id}`,
    `GET /empire/overview`, `GET /victory`, `GET /notifications`,
    `GET /turn-queue`. Existing `GET /map/overlays` now returns real
    data.
  - Client bindings:
    `components/api/{diplomacy,empire,victory,notifications}.rs`.
  - Smoke-tested end-to-end: 3-civ game showed Babylon + Greece in
    `/diplomacy`, the player's Rome civ ranked 1/3 in `/victory` with
    score 7, `/turn-queue` flagged the missing research target as
    required, and `/empire/overview` returned `{cities:1, military_units:1,
    treasury:0}`.

### Phase 3 — Tech / Civics / Government reads + research mutations (2026-05-07)

- **Commit `feat(open4x-server): tech + civics REST surface (Phase 3 server)`**:
  - `build_tech_tree` projects every tech node with status ∈ {done, current,
    available, locked} computed from `researched_techs` + `research_queue`
    front. Era is inferred heuristically from cost (50 → Ancient,
    100–199 → Medieval, etc.) until libciv exposes era per node — flagged
    in `era_for_tech_cost`.
  - `build_civics_tree` mirrors the tech projection over
    `civic_in_progress` + `completed_civics`.
  - `build_government` projects current government, era, and a placeholder
    slots block (2/2/1/0). Catalogue is empty until libciv ships a policy
    catalogue helper — flagged for follow-up.
  - REST handlers: `GET /tech`, `POST /tech/research` (→
    `GameAction::QueueResearch`), `GET /civics`,
    `POST /civics/research` (→ `GameAction::QueueCivic`),
    `GET /government`.
  - Client bindings `components/api/{tech,government}.rs`.
  - Smoke-tested: `/tech` lists Bronze Working / Machinery / etc. with
    correct status; `POST /tech/research` queued the first available tech
    (queue went `[]` → `[<tech_id>]`); `/government` returned the Chiefdom
    block.

### Phase 2 — Cities + Units (server side, 2026-05-07)

- **Commit `feat(open4x-server): cities + units REST surface (Phase 2 reads)`**:
  - Real projectors `build_cities`, `build_city_tiles`, `build_units`,
    `build_armies` (stub), `build_combat_preview` (heuristic — `note` field
    flags the libciv replacement).
  - Wire-type expansions in `types/web.rs` (city queue, owner, worked-tile
    count; unit hp/mp/strength/category/domain/actions; combat preview).
  - REST handlers `cities`, `city_detail`, `city_tiles`, `units`,
    `unit_detail`, `armies`, `combat_preview`.
  - Client bindings in `components/api/{cities,units}.rs`.
  - Two new projector tests bringing the total to 5.
  - **ID-on-the-wire fix**: switched every emission to
    `*.as_ulid().to_string()` (the newtype Display wraps as `Type(ULID)`
    which broke `/cities/{id}` and `/units/{id}` path parsing). Bare ULID
    strings everywhere now.
- **Commit `feat(open4x-server): mutation routes for production + unit actions (Phase 2 writes)`**:
  - `POST /api/v1/cities/{id}/production` — accepts `{item_id, item_type}`,
    routes through existing `GameAction::QueueProduction`. Returns the
    updated `CityRow` in the standard `MutationResponse` envelope.
    Supports `item_type` ∈ {unit, building, wonder, project}; district is
    plain-enum and gets a `400 invalid_item_type` for now.
  - `DELETE /api/v1/cities/{id}/production/{pos}` — routes to
    `GameAction::CancelProduction`.
  - `POST /api/v1/units/{id}/action` — body `{action_id, target_q?,
    target_r?, name?}`. Dispatches to `MoveUnit`, `Attack` (defender
    resolved by coord lookup against the player's `GameView`), or
    `FoundCity`. `fortify`/`sleep` are stubbed as 202-no-op until libciv
    grows the matching `GameAction`s.
  - `mutate_room` helper applies a closure under a `DashMap::get_mut`
    lock and translates `Result<(), String>` errors into 400
    `rule_violation` responses.
  - Client bindings in `components/api/cities.rs::queue_production` /
    `cancel_production` and `components/api/units.rs::dispatch`.
  - Smoke-tested end-to-end against `target/release/open4x-server`:
    move warrior (1,2) → (2,2) drained mp 2 → 0; queue Warrior in capital;
    delete queue position 0 returned the city row with empty queue.
  - No libciv changes in this commit (mutations all use existing
    `GameAction` variants, already covered by the WS path and tests).

### Phase 1 — HUD MVP (2026-05-07)

- **Commit `feat(open4x-server): wire HUD MVP REST surface (Phase 1, server-side)`**:
  - Real `build_player_state(view, turn_limit)` projecting yields, gold, faith,
    era, strategics out of `GameView.my_civ`.
  - Real `build_world_snapshot(view, q, r, radius)` projecting board metadata,
    cities, units, terrain labels, owner names, and per-tile fog. Topology →
    `wrap_x`/`wrap_y` mapping. Optional radius cap at 32 (radius `0` = "no
    filter"). 3 unit tests in `web_projection::tests` cover yields, topology,
    and cube hex-distance radius filtering.
  - REST handlers in `server/rest/handlers.rs` (`player_state`,
    `world_snapshot`, `world_tile`, `map_overlays` stub, `end_turn`) backed by
    `auth::auth_or_401` → `project_game_view`. End-turn calls
    `GameRoom::resolve_turn` and returns the standard `MutationResponse` envelope.
  - Client bindings `components/api/{player_state,world,turn}.rs` over the
    Phase 0 `fetch_json` helper.
- **Commit `feat(open4x-server): POST /api/v1/games/new bootstraps single-player session`**:
  - Unauthenticated handler that creates a fresh `GameRoom` via
    `build_server_session`, mints a random pubkey + bearer token, and returns
    `{game_id, civ_id, token, turn}`. Required for the REST single-player loop
    since there was no other way to obtain a bearer token without going through
    the WS auth handshake.
- **Commit `feat(open4x-server): REST-driven HUD topbar + map (Phase 1, client-side)`**:
  - `components/api/games.rs` binding for `POST /games/new`.
  - `pages/rest_game.rs::RestGamePage` — minimal Leptos page that bootstraps a
    game on mount, owns three signals (`token`, `tick`, error trackers) and two
    `LocalResource`s (`player_state`, `snapshot`) keyed on `tick`. End Turn
    button bumps the tick to refetch every resource in parallel.
  - `lib.rs::start()` — first actual WASM entry point in the crate. The Leptos
    components existed before but were never mounted; `trunk serve` would
    previously have rendered an empty `<body>`.
  - HexMap rewrite intentionally deferred to Phase 2 — current page renders a
    "World w × h · N tiles in view" placeholder so the snapshot fetch loop is
    observable.
  - Live smoke test against `target/release/open4x-server`: `/api/v1/health` →
    OK, `POST /games/new` → token, `GET /player-state` → real yields, `GET
    /world/snapshot?radius=2` → 4 tiles around origin, `POST /turn/end` →
    `{turn: 1}`.

### Phase 0 — Scaffolding (2026-05-07)

- **Commit `docs(web-ui): import wireframe and add Leptos port roadmap`** —
  brought in `open4x-webui/` as a visual reference and wrote this plan.
- **Commit `feat(open4x-server): scaffold /api/v1 REST surface (Phase 0)`**:
  - `open4x-server/src/types/web.rs` — wire-type modules for every wireframe
    schema (`player_state`, `world`, `tech_tree`, `civics_tree`, `government`,
    `diplomacy`, `empire_overview`, `victory`, `city_data`, `city_tiles`,
    `unit_data`, `army_data`, `notifications`, `turn_queue`, `map_overlays`)
    plus the shared `MutationResponse<T>` envelope and `ApiErrorBody`. All
    types are `Default` stubs; field schemas land in Phase 1+.
  - `open4x-server/src/server/web_projection.rs` — one builder stub per wire
    type. Each returns `Default::default()` for now.
  - `open4x-server/src/server/rest/{mod,auth,handlers}.rs` — new REST module.
    `auth.rs` wraps `api_token::resolve_token` into the `Result<_, ApiError>`
    shape Axum expects; `handlers.rs` ships only `health` for Phase 0.
  - `open4x-server/src/main.rs` — wired `/api/v1/health` next to the legacy
    `/api/game/*` routes.
  - `open4x-server/src/components/api/{mod,http,player_state}.rs` — client
    bindings layer. `http.rs` is a wasm-bindgen `fetch_json` helper (no JS
    shim). `player_state::get` is the first stub call.
  - `book/src/getting-started.md` — documented the trunk build (`--features
    csr --no-default-features`) and added a pointer to this roadmap.
  - `cargo build -p open4x-server` (ssr) and
    `cargo build -p open4x-server --no-default-features --features csr
    --target wasm32-unknown-unknown` both pass clean.
  - `cargo clippy -p open4x-server -- -D warnings` (ssr) clean. csr clippy
    surfaces 3 pre-existing `uninlined_format_args` warnings in
    `tabs/{city,culture,science}.rs` — unrelated to this scaffolding;
    tracked for cleanup in a later phase.

### Open API decisions resolved during Phase 0

- The `MutationResponse<T>` and `ApiErrorBody` envelopes from §4.2/§4.3 now
  exist concretely in `types/web.rs` so every Phase 1+ handler can return
  them without rediscussion.
- `/api/v1/health` is intentionally **unauthenticated** (mirrors the existing
  `/health`). Every other `/api/v1/*` endpoint will go through
  `rest::auth::auth_or_401` and so requires a bearer token.
- Tile coordinates on the wire use axial `(q, r)` only (no `s`), per §8 — the
  `world::TileCoord` and `world::TileView` types reflect this.
