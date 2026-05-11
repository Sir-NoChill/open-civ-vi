# CLI Server Mode (Parity Harness)

> **Goal**: a second execution mode for `open4x-cli` that talks to a
> running `open4x-server` over HTTP instead of running the rules engine
> in-process. The CLI surface stays identical — same subcommands, same
> JSON output shapes — so we can drive both modes against the same
> script and diff the results to evaluate parity between the embedded
> engine and the client/server projection.

## Why

The local CLI exercises `libciv` directly: it loads a save file from
disk, calls `RulesEngine` methods, applies diffs, and writes the file
back. The Leptos web client takes a very different path — it speaks
REST against `open4x-server`, which holds the canonical `GameState` in
memory and projects per-civ "views" through `web_projection::*`.

Today the two paths share `libciv` but diverge in:

- which fields are exposed to the player (fog-of-war filter, owner
  vs. foreign visibility, action availability),
- how IDs are serialised (raw ULID vs. `TypeName(ULID)` debug print),
- how errors surface (free-text `RulesError` strings vs. structured
  `{error, message}` JSON),
- how mutations advance state (CLI applies a diff inline; server runs
  `room.apply_action` then re-projects).

A parity harness lets us:

1. Catch projector regressions — when `web_projection::build_*` drops
   a field, the diff against local CLI output makes it obvious.
2. Validate that REST mutations produce the same downstream state as
   the equivalent local action.
3. Reuse the (large) CLI test scripts to exercise the server.

## Non-goals

- **Replacing** the local CLI mode. Local mode stays the primary tool
  for fast iteration on `libciv`.
- **Multiplayer**. Server mode here drives the same single-player
  bootstrap (`POST /api/v1/games/new`) that the Leptos client uses.
  Hot-seat and AI-vs-AI flows belong to the WebSocket surface and are
  out of scope.
- **Save-file interop**. Local saves are not portable to server games
  and vice-versa — a server game lives in `AppState.games` for the
  process lifetime (until the persistence layer lands).

## CLI surface

A new global flag pair:

```text
open4x [--server <URL>] [--token-file <PATH>] <subcommand> ...
```

- `--server <URL>` (env: `OPEN4X_SERVER_URL`): switches every
  subcommand to the remote dispatcher. Without it, the CLI behaves
  exactly as before.
- `--token-file <PATH>` (env: `OPEN4X_TOKEN_FILE`): JSON file holding
  `{game_id, civ_id, token, turn}` from `POST /games/new`. `new-game`
  *writes* this file; every other subcommand *reads* it. Defaults to
  `./.open4x-session.json`.

`--game-file` and `--player` remain on every subcommand for backward
compatibility but are ignored when `--server` is set (the bearer
token already identifies the game + civ).

### Bootstrap flow

```bash
# Start a fresh game on the server. Writes ./.open4x-session.json.
open4x --server http://localhost:3001 \
       new-game --width 40 --height 24 --seed 42 \
                --player Rome --ai Babylon

# Subsequent commands pick up the token automatically.
open4x --server http://localhost:3001 status yields
open4x --server http://localhost:3001 list units
open4x --server http://localhost:3001 action move \
       --unit 01HPQR... --to-q 5 --to-r 3
open4x --server http://localhost:3001 end-turn
```

### Output parity

Where the server already returns the right data, the remote handler
prints the server JSON verbatim (after re-pretty-printing through
`serde_json::to_string_pretty` so diffs are clean). Where the local
JSON shape is richer than the wire shape (e.g. local `status techs`
returns researched + in-progress; the server's `/tech` returns the
full tree with status), the remote handler keeps the **server**
shape and we surface the difference as an expected divergence in
the parity matrix below — fixing the divergence is a separate
project-tier task, not blocking this harness.

## Parity matrix

Each row maps a CLI subcommand to its REST endpoint. Status:

- ✅ supported by both today
- 🟡 supported on the server but emits a different JSON shape than
  the local handler (parity diff is expected and tracked)
- ⛔ no REST equivalent yet — remote mode returns
  `Err("server mode does not yet support <subcommand>")`

### Top-level

| Subcommand | REST | Status | Notes |
|---|---|---|---|
| `new-game` | `POST /games/new` | 🟡 | Server bootstrap accepts only `{display_name, width, height, seed, num_ai, turn_limit}` — multiple human players + per-player civ names are not exposed. The CLI maps `--player[0]` → `display_name`, `--ai.len()` → `num_ai`; extra `--player` entries error out. |
| `end-turn` | `POST /turn/end` | ✅ | Server runs AI + advances on submit (single-player mode). |
| `view` | `GET /world/snapshot?radius=0` | 🟡 | Local `view` emits the legacy `PlayerView` shape (units, cities, civs as flat arrays). Server emits the wireframe `WorldSnapshot`. Diff is structural, intentional. |

### `action <kind>`

| ActionKind | REST | Status |
|---|---|---|
| `Move` | `POST /units/{id}/action` (`action_id=move`) | ✅ |
| `Attack` | `POST /units/{id}/action` (`action_id=attack`) | ✅ |
| `FoundCity` | `POST /units/{id}/action` (`action_id=found_city`) | ✅ |
| `Build` | `POST /cities/{id}/production` | ✅ (unit/building/wonder/project; districts not yet wired) |
| `CancelProduction` | `DELETE /cities/{id}/production/{pos}` | 🟡 (CLI cancels the front; REST takes an explicit index — CLI sends `0`) |
| `Research` | `POST /tech/research` | ✅ |
| `CancelResearch` | `DELETE /tech/research` | ✅ |
| `StudyCivic` | `POST /civics/research` | ✅ |
| `CancelCivic` | `DELETE /civics/research` | ✅ |
| `AdoptGovernment` | `POST /government/change` | ✅ |
| `AssignCityFocus` | `POST /cities/{id}/focus` | ✅ |
| `RenameCity` | `POST /cities/{id}/rename` | ✅ |
| `CityBombard` / `TheologicalCombat` / `PromoteUnit` / `RockBandPerform` / `PlaceDistrict` / `PlaceImprovement` / `PlaceRoad` / `AssignCitizen` / `UnassignCitizen` / `ClaimTile` / `ReassignTile` / `AssignPolicy` / `Declare/Make/FormAlliance` / `Assign/EstablishTradeRoute` / all religion / all great people / all governors / `CompleteScienceMilestone` / all barbarian | — | ⛔ no REST mutation yet. Adding each is a `GameAction` enum variant + REST route + `room.apply_action` arm — see `book/src/roadmap/ongoing.md` for the running checklist. |

### `status <kind>`

| StatusKind | REST | Status |
|---|---|---|
| `Yields` | `GET /player-state` | 🟡 (local emits a flat `{food, production, gold, science, culture, faith}`; server emits `{turn, era, resources: {gold: {value, per_turn}, ...}, happiness, strategic}`) |
| `Pending` | `GET /turn-queue` | ✅ |
| `Victory` | `GET /victory` | ✅ |
| `Policies` | `GET /government` (`catalogue` field) | ✅ |
| `UnitActions { id }` | `GET /units/{id}` (`actions` field) | ✅ |
| `CombatPreview` | `GET /combat/preview?attacker_id&defender_q&defender_r` | ✅ |
| `Diplomacy` | `GET /diplomacy` | 🟡 |
| `Techs` / `Civics` | `GET /tech`, `GET /civics` | 🟡 (server returns full tree) |
| `City { id }` / `Unit { id }` / `Tile { q, r }` | `GET /cities/{id}`, `GET /units/{id}`, `GET /world/tile/{q}/{r}` | 🟡 |
| `Scores` / `Congress` | — | ⛔ no REST equivalent. |

### `list <kind>`

| ListKind | REST | Status |
|---|---|---|
| `Units` | `GET /units` | 🟡 (server returns own + visible foreign units; local returns only own) |
| `Cities` | `GET /cities` | 🟡 (same own/foreign distinction) |
| `Production { city }` | `GET /cities/{id}` (uses `production_options`) or `GET /registry` | ✅ |
| `Buildings` / `GreatPeople` / `Routes` / `Governors` / `Improvements` | — | ⛔ no REST equivalent. |

The ⛔ rows on `action` and `status`/`list` are not blockers for the
harness — server mode simply errors out cleanly on those subcommands
and the local mode keeps working. As more REST mutations land (the
roadmap in `ongoing.md` already tracks them), the matrix promotes
rows from ⛔ to ✅.

## Architecture

### Crate changes

`open4x-cli/Cargo.toml`:

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "json", "rustls-tls"] }
```

`rustls-tls` keeps the binary self-contained (no system OpenSSL
dependency) and works fine inside the `rust:slim` Docker base.
`blocking` keeps the CLI synchronous — there's no need for an async
runtime in a one-shot CLI invocation.

### Module layout

New module tree under `open4x-cli/src/`:

```
remote/
  mod.rs           — re-exports + dispatch (`run_remote(...)`)
  client.rs        — `ApiClient` (reqwest wrapper, bearer auth, error map)
  session.rs       — `SessionFile` (load/save the token JSON)
  bootstrap.rs     — POST /games/new
  end_turn.rs      — POST /turn/end
  view.rs          — GET /world/snapshot
  status.rs        — every `StatusKind` arm
  list.rs          — every `ListKind` arm
  action.rs        — every `ActionKind` arm with a REST equivalent
```

Existing handlers under `handlers/` stay untouched — they're the
local path. `main.rs` learns a single dispatch fork:

```rust
if let Some(server) = parsed.server.as_deref() {
    remote::run_remote(server, &parsed.token_file, parsed.command)?;
} else {
    // existing local dispatch
}
```

### `ApiClient`

```rust
pub struct ApiClient {
    base: String,            // "http://localhost:3001"
    token: Option<String>,   // Authorization: Bearer <token>
    inner: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base: impl Into<String>) -> Self { ... }
    pub fn with_token(mut self, t: String) -> Self { self.token = Some(t); self }

    // Generic verbs — all return Value so handlers can pretty-print.
    pub fn get_json(&self, path: &str) -> Result<Value, RemoteError> { ... }
    pub fn post_json(&self, path: &str, body: &Value) -> Result<Value, RemoteError> { ... }
    pub fn delete(&self, path: &str) -> Result<Value, RemoteError> { ... }
}

pub enum RemoteError {
    Network(reqwest::Error),
    Status { code: u16, body: Value },   // server's {error, message}
}
```

Errors map to the same `Result<(), String>` surface the local
handlers use, so `main.rs`'s `eprintln!("Error: ...")` path stays
common.

### Session file

```json
{
  "server": "http://localhost:3001",
  "game_id": "01HPQR...",
  "civ_id":  "01HPQR...",
  "token":   "8f3...",
  "turn":    0
}
```

Written atomically (write to `<path>.tmp`, then rename) by
`new-game`. Read by every other subcommand. If the file is missing
when a non-`new-game` subcommand runs in remote mode, the CLI prints
a friendly error pointing back at `new-game`.

### Output

Remote handlers reuse `crate::output::print_result` where the wire
shape already matches `ActionResult { ok, turn, deltas, error }`
(mutations) and fall through to `serde_json::to_string_pretty` for
read-only queries. Either way, the **stdout shape from local and
remote modes is JSON**, so a parity test just diffs the two streams.

## Phased rollout

### Phase 0 — scaffolding (this PR)

- [ ] Add `reqwest` dep + `--server` / `--token-file` global flags
- [ ] `remote::client::ApiClient` + `remote::session::SessionFile`
- [ ] `remote::bootstrap::new_game` (POST /games/new + write session)
- [ ] `remote::end_turn::end_turn` (POST /turn/end)
- [ ] Remote dispatch fork in `main.rs`
- [ ] Smoke test against `cargo run -p open4x-server` on localhost

### Phase 1 — read coverage

- [ ] `remote::view::view` → `GET /world/snapshot?radius=0`
- [ ] `remote::status` arms: `Yields`, `Pending`, `Victory`, `Policies`,
      `UnitActions`, `CombatPreview`, `Techs`, `Civics`, `City`, `Unit`,
      `Tile`, `Diplomacy`
- [ ] `remote::list` arms: `Units`, `Cities`, `Production`
- [ ] Stub-out `Scores`, `Congress`, and the unsupported `list` arms
      with a clean "not supported in server mode" error

### Phase 2 — write coverage (REST today)

- [ ] `remote::action` arms: `Move`, `Attack`, `FoundCity`, `Build`,
      `CancelProduction`, `Research`, `CancelResearch`, `StudyCivic`,
      `CancelCivic`, `AdoptGovernment`, `AssignCityFocus`, `RenameCity`
- [ ] Every other `ActionKind` variant returns a clean
      "not supported in server mode" error

### Phase 3 — parity harness

- [ ] `tests/parity.rs` integration test: spin up an in-process server
      via `tower::ServiceExt::oneshot` (already used in
      `open4x-server/tests/rest_api.rs`), drive a fixed action
      sequence locally and against the server, diff the JSON.
- [ ] Allowlist the structural-by-design diffs (player-state vs.
      yields, world snapshot vs. player view) so the test only fails
      on regressions.

### Phase 4 — promote ⛔ rows as REST grows

Each new `GameAction` variant landing in
`book/src/roadmap/ongoing.md` brings a corresponding `remote::action`
arm. Track the promotions inline in this file's parity matrix.

## Docker harness

A tiny compose stack lives under `dockerfiles/`:

```
dockerfiles/
  README.md
  server.Dockerfile      # multi-stage build of open4x-server (ssr-only,
                         # no csr/dist — API-only mode)
  cli.Dockerfile         # multi-stage build of open4x-cli
  docker-compose.yml     # `server` + `cli` services
  cli-entrypoint.sh      # exec wrapper so `docker compose run cli ...`
                         # forwards args directly to the open4x binary
```

Usage:

```bash
cd dockerfiles
docker compose build
docker compose up -d server          # API on http://server:3001 inside the network
docker compose run --rm cli new-game --width 30 --height 18 --seed 1 \
                                     --player Rome --ai Babylon
docker compose run --rm cli status yields
docker compose run --rm cli list units
docker compose run --rm cli action move --unit <ID> --to-q 5 --to-r 3
docker compose run --rm cli end-turn
docker compose down
```

The session JSON lives in a named volume (`open4x-session`) mounted
into the `cli` service at `/work/.open4x-session.json`, so successive
`docker compose run cli ...` invocations share the same token.

The server image:

- builds with `--features ssr --no-default-features` to skip the
  `csr`/`web-sys` dependency tree,
- runs with no `OPEN4X_STATIC_DIR` set (the `dist/` fallback returns
  404s for the SPA, which is fine — we want API-only),
- exposes port 3001 on the compose network.

The CLI image:

- builds `open4x-cli` only,
- ships an entrypoint that reads `OPEN4X_SERVER_URL` (set in compose
  to `http://server:3001`) so users don't need to pass `--server` on
  every invocation.

## Known divergences (parity matrix tracking)

Anything in this list is **expected** to differ between local and
remote modes. Promoting an item out of this list means either the
local handler now emits the server's shape or the server now emits
the local handler's shape.

1. `view` — `PlayerView` (local) vs. `WorldSnapshot` (remote).
2. `status yields` — flat yield map (local) vs.
   `/player-state.resources` buckets with per-turn deltas (remote).
3. `status techs` / `status civics` — researched + active
   (local) vs. full tree with per-node status (remote).
4. `list units` / `list cities` — own-only (local) vs. own + visible
   foreign (remote).
5. ID rendering — `UnitId(01HPQR...)` debug format (local) vs. raw
   `01HPQR...` string (remote). The local handler accepts both forms
   on input; the remote handler always emits the raw form.
6. Error envelope — `"<RulesError debug print>"` string (local) vs.
   `{error, message}` object (remote). Both surface as a non-zero
   exit code with `eprintln!("Error: ...")`.
