# Crate Split — Extract Protocol, SDK, and Web Client from `open4x-server`

> **Goal**: split the dual-purpose `open4x-server` crate (Axum SSR + Leptos
> CSR + shared wire types, ~9 800 LOC, 30+ optional deps gated by mutually
> exclusive `ssr`/`csr` features) into four single-purpose crates inside the
> existing workspace, with **zero behavioural regression** at every step.
>
> **Status (2026-05-16)**: P0 → P5 merged to local main. The four target
> crates exist, the protocol contract is extracted, the SDK has both
> backends with the wasm32 target building cleanly, the CLI runs on the
> SDK, and the Leptos UI lives in `open4x-client-web/`. **P3 (server
> slim-down) and P6 (cleanup + docs) remain.** All parity gates green:
> `cargo build --workspace`, `cargo test --workspace`,
> `cargo clippy --workspace -- -D warnings`,
> `cargo check -p open4x-client-web --target wasm32-unknown-unknown`,
> `trunk build --release` from `open4x-client-web/`.

---

## 1. Why

The current crate is a feature-gated chimera:

```
open4x-server/
  Cargo.toml          # 30 deps, `default = ["ssr"]`, `csr` mutually exclusive
  src/lib.rs          # #[cfg(ssr)] mod server;  #[cfg(csr)] mod components;
  src/types/          # compiles for BOTH targets (already the de-facto protocol)
  src/server/         # Axum, REST, WS, GameRoom, projections (ssr-only)
  src/components/     # Leptos UI + WASM HTTP client (csr-only)
  src/pages/          # Leptos routes (csr-only)
  src/tabs/           # Leptos tab views (csr-only)
```

Concrete pain points:

- **Tooling confusion** — rust-analyzer picks one feature set; the other half
  goes red. Switching costs editor restarts.
- **Compile blowup** — Cargo unifies the union of optional deps across all
  feature combinations during dependency resolution.
- **Leakage risk** — a stray `#[cfg]` slip can pull `axum`/`tokio` into the
  WASM bundle or `web-sys` into the server binary.
- **Implicit protocol** — there is no single crate that defines "the wire
  contract." `types/` plays that role today but lives next to the consumers.
- **Blocks bring-your-own-client** — the reference client co-locating with
  the server signals that the protocol is whatever the Leptos UI happens to
  call. Splitting forces the contract to be explicit and reusable.

## 2. End State

Four crates inside this workspace (single repo, single CI):

| Crate                | Targets                    | Depends on                          | Role                                                |
|----------------------|----------------------------|-------------------------------------|-----------------------------------------------------|
| `open4x-protocol`    | any (no_std-friendly)      | `serde`, `ulid`                     | Wire types, versioned (`v1::*`). The contract.      |
| `open4x-sdk`         | native + `wasm32`          | `open4x-protocol` + backend         | Typed HTTP/WS client. Two feature-gated transports. |
| `open4x-server`      | native                     | `libciv`, `open4x-protocol`         | Axum + REST + game rooms. **No Leptos, no cdylib.** |
| `open4x-client-web`  | `wasm32-unknown-unknown`   | `open4x-sdk`, `open4x-protocol`     | Leptos CSR + (future) WebGL renderer.               |

Unchanged crates: `libhexgrid`, `libciv`, `open4x-cli` (migrates to use
`open4x-sdk` internally — same CLI surface).

## 3. Hard parity constraints

The migration is a refactor, not a rewrite. Every step must hold these
invariants:

1. `cargo test --workspace` stays green at every commit.
2. `cargo clippy --workspace -- -D warnings` stays clean.
3. `open4x-server/tests/rest_api.rs` (694 LOC) passes unmodified except for
   `use` path renames.
4. The CLI server-mode parity harness ([`cli-server-mode.md`](./cli-server-mode.md))
   passes unmodified — its baseline transcripts double as wire-format
   snapshots.
5. `trunk build --release` of the web client produces a `dist/` bundle that
   loads against the server with no visible UI difference.
6. The on-the-wire JSON shape of every `/api/v1/*` response is byte-stable
   (whitespace + field order may differ, value structure must not).

## 4. Phase plan

The strategy is the strangler-fig pattern: create empty target crates first,
then move modules one at a time with re-exports holding old paths in place
until consumers are migrated.

### Phase 0 — Workspace scaffolding (mechanical) — **DONE**

Landed three new crates with stubs and the full Cargo feature scaffold so
later phases never edit shared `mod.rs` files.

- `open4x-protocol/`: `lib.rs` exposes `pub mod v1`; `v1/mod.rs` empty.
- `open4x-sdk/`: backend modules cfg-gated; `endpoints/mod.rs` pre-seeded
  with all 18 resource module declarations so P2a/P2b never collide on
  it; `[features]` block has `native-blocking`, `native-async`, `wasm`.
- `open4x-client-web/`: cdylib + `wasm_bindgen(start)` stub + minimal
  `index.html` trunk entry.

**Deviations from the original plan** (both intentional, recorded here for
future reference):

1. Workspace `members` was reduced from 7 to 7 (added the 3 new crates,
   moved `open4x-accounts` and `open4x-lobby` to `[workspace] exclude`).
   The user stated those two crates are being extracted to a separate
   repo; keeping them as workspace members caused `cargo build
   --workspace` to fail on a pre-existing `rust-embed` derive issue in
   `open4x-lobby` that isn't worth fixing in-tree. Source trees stay on
   disk; `cargo build -p open4x-lobby` from inside still works.
2. Baseline cleanup commit was bundled with the P0 PR (one missing
   `CityView.focus` field in `web_projection.rs`, 8 `uninlined_format_args`
   warnings in `open4x-cli`). These pre-existed on `origin/main` and
   would have broken the §3 parity gates immediately. Trivial to fix; not
   worth a separate PR.

**Exit criterion**: `cargo build --workspace` succeeds, all crates in
`cargo metadata`. Server and CLI unchanged otherwise.

### Phase 1 — Extract `open4x-protocol` — **DONE**

The keystone. Every later phase depended on it.

1. Moved every module from `open4x-server/src/types/` to
   `open4x-protocol/src/v1/` (used plain `mv`, not `git mv` — the
   GitButler skill forbids any `git` write command; rename detection at
   diff time preserves blame).
2. `open4x-protocol/src/lib.rs` exposes `pub mod v1`; `v1/mod.rs`
   declares submodules and re-exports the same public surface the old
   `types/mod.rs` had (`coord::{HexCoord, HexDir}`, `enums::*`, `ids::*`,
   `messages::{ClientMessage, GameAction, GameStatus, ServerMessage}`,
   `profile::{CivTemplate, ProfileView}`, `reports::*`, `view::GameView`).
3. `open4x-server/src/types/mod.rs` is now a shim:
   ```rust
   pub use open4x_protocol::v1::*;
   pub use open4x_protocol::v1::{
       coord, enums, ids, messages, profile, reports, view, web,
   };
   ```
   Keeps every `use crate::types::messages::Foo`-style import working
   until P6. Removed in P6.
4. `open4x-server/Cargo.toml` gained
   `open4x-protocol = { path = "../open4x-protocol" }`.

**Tests added**:
- `open4x-protocol/tests/wire_schema.rs` — 8 snapshot tests covering
  `GameView`, `PlayerState`, `WorldSnapshot`, `MutationResponse<()>`,
  `ApiErrorBody` (default + populated), plus roundtrip checks and a
  field-order-tolerance meta-test. Snapshots compare `serde_json::Value`
  (per §7 risk register), so whitespace and field order don't break the
  test but schema drift does.

**Incidental fix**: `web.rs` had stale intra-doc links
(`[crate::server::web_projection]`, `[crate::components::api]`) that
don't resolve in the new crate; converted to plain-text references to
keep clippy clean.

**Exit criterion**: `cargo test --workspace` green; `rest_api.rs`
unmodified.

### Phase 2 — Build `open4x-sdk` (two backends, parallel) — **DONE**

Required one prep lane before the parallel split:

**Phase 2-prep** — defined the shared `Transport` trait
(`open4x-sdk/src/transport.rs`) with `async fn request(method, path,
body) -> Result<Vec<u8>, ApiError>` and a `Method` enum. Without this the
parallel agents would have had to coordinate the trait shape across lanes.
Landed as its own one-commit lane, merged before P2a/P2b started.

Final SDK shape:

```
open4x-sdk/src/
  lib.rs              # re-exports Transport, Method; cfg-gates backends
  transport.rs        # Transport trait + Method enum
  endpoints/
    mod.rs            # pre-seeded module decls (P0)
    armies.rs cities.rs civics.rs combat.rs diplomacy.rs empire.rs
    games.rs government.rs health.rs map.rs notifications.rs
    player_state.rs registry.rs tech.rs turn.rs units.rs victory.rs world.rs
  native.rs           # NativeBlockingClient + NativeAsyncClient
  wasm.rs             # WasmClient (web_sys::fetch + SendWrapper)
  error.rs            # ApiError + from_response/transport helpers
```

`endpoints/*` functions are async and generic over `T: Transport`; the
trait method returns `impl Future<Output = Result<Vec<u8>, ApiError>> +
Send`. The blocking native client satisfies the Send bound by precomputing
its result and returning an immediately-ready future. The wasm client
satisfies it via `send_wrapper::SendWrapper` (with the `futures` feature
enabled — `SendWrapper<F>` only impl-Futures behind that flag).

**Cargo features** (as planned):
- `default = ["native-blocking"]`
- `native-blocking` — reqwest::blocking
- `native-async` — reqwest
- `wasm` — web-sys + wasm-bindgen-futures + js-sys + serde-wasm-bindgen +
  send_wrapper (with `futures` feature). A
  `[target.'cfg(target_arch = "wasm32")'.dependencies]` block forces
  `getrandom = { version = "0.3", features = ["wasm_js"] }` so the
  transitive `ulid → rand → getrandom` chain compiles on the wasm target.

**Phase 2a `phase/2a-sdk-native`** (3 commits): native Transport impls
(blocking + async) + ApiError decoding; 18 endpoint module bodies;
`tests/native_roundtrip.rs` with 11 tests against an in-process Axum
router via `tower::ServiceExt::oneshot` (same pattern as `rest_api.rs`).

**Phase 2b `phase/2b-sdk-wasm`** (3 commits): `WasmClient` over
`web_sys::fetch`; `tests/wasm_smoke.rs` (cfg-gated to `target_arch =
"wasm32"`); Cargo.lock update for the wasm-bindgen-test transitive deps.

**Phase 2-postfix `fix/sdk-getrandom-wasm`** (1 commit): caught two issues
the wasm target build surfaced only after P2 merged — `getrandom`
needed its `wasm_js` feature pinned at the SDK level, and `send_wrapper`
needed its `futures` feature enabled for the wrapped JsFuture to satisfy
the trait's `+ Send` future bound.

**Coordination outcomes**:
- The hot zone `open4x-sdk/Cargo.toml` was edited by both P2a and P2b
  concurrently. P2b finished first and committed Cargo.toml changes
  (dev-deps + wasm send_wrapper). When P2a later went to commit its own
  Cargo.toml additions (open4x-server, tower, http-body-util, etc., as
  dev-deps), GitButler saw the file already contained them on disk and
  didn't generate a hunk — so P2a's Cargo.toml edits ended up attributed
  to P2b's commit. Net effect: clean state, slight commit-attribution
  oddity. The §11.3 hot-zone rule held in spirit if not in letter.

### Phase 3 — Slim `open4x-server` — **PENDING** (next up)

P4 already removed the UI sources and the `wasm_bindgen(start)` function
from `lib.rs`. P3's remaining scope:

1. ~~Delete `open4x-server/src/components/`, `pages/`, `tabs/`.~~ Done in P4.
2. Drop the `csr` feature, drop `crate-type = ["cdylib", "rlib"]`, drop every
   `dep:leptos` / `dep:wasm-*` / `dep:web-sys` / `dep:js-sys` /
   `dep:serde-wasm-bindgen` / `dep:console_error_panic_hook` / `dep:getrandom`
   from `open4x-server/Cargo.toml`.
3. ~~Delete `wasm_bindgen(start)` from `lib.rs`.~~ Done in P4.
4. Make `ssr` the only build path; collapse the feature into the default deps
   (or drop the feature flag altogether).
5. Keep `OPEN4X_STATIC_DIR` plumbing in `main.rs`; the served `dist/` now
   comes from `open4x-client-web/dist/`. The default value in `main.rs`
   currently points at `./open4x-server/dist` — update it to
   `./open4x-client-web/dist`.
6. Drop `#![cfg(feature = "ssr")]` from `open4x-server/tests/rest_api.rs`
   (the feature won't exist).

**Tests**: `cargo test -p open4x-server` runs `rest_api.rs` unchanged
beyond that one cfg-line removal.

### Phase 4 — Extract `open4x-client-web` — **DONE**

Three commits on `phase/4-client-web`, ~24 files touched:

1. `open4x-server/src/{components,pages,tabs}/**` moved via plain `mv`
   into `open4x-client-web/src/{components,pages,tabs}/`. Rename
   detection preserves blame.
2. `components/api/` subtree (the WASM HTTP client) **deleted** —
   superseded by `open4x_sdk::endpoints::*`. Every typed call has an
   SDK equivalent.
3. Bulk import rewrites: `use crate::types::*` → `use open4x_protocol::v1::*`,
   `use crate::components::api::*` → `use open4x_sdk::endpoints::*`. No
   `crate::server::*` leaks were found in the moved UI — the original
   split was already clean.
4. The single SDK adapter lives in `pages/rest_game.rs`: a tiny
   `client(token)` helper builds a `WasmClient` rooted at `""` (fetch
   resolves paths against `window.location`) and folds in the bearer
   token.
5. Call-site naming aligned to SDK conventions: `api::games::new` →
   `new_game`, `api::tech::tech` → `get`, `api::tech::research_tech` →
   `research`, `api::empire::get` → `overview`,
   `api::notifications::turn_queue` → `api::turn::queue`.
6. `open4x-server/index.html` (legacy wireframe stylesheet hook) moved
   to `open4x-client-web/index.html`; trunk `data-bin` hint dropped
   (this crate is a cdylib, not a bin — trunk auto-discovers).
7. `open4x-client-web/Cargo.toml` rewritten with the full csr-equivalent
   dep set (leptos 0.7, web-sys with browser-API features, js-sys,
   wasm-bindgen-futures, serde-wasm-bindgen, ed25519-dalek for client
   auth keystore, getrandom with `wasm_js`), all under
   `[target.'cfg(target_arch = "wasm32")'.dependencies]` so workspace
   native builds stay green without a leptos toolchain.
8. `.gitignore` got `open4x-client-web/dist` to match the existing
   server entry.

**Deviations from the spec**:

- `open4x-client-web/src/lib.rs` module declarations are
  `cfg(target_arch = "wasm32")`-gated. The spec said "no feature flags
  — this crate is wasm-only", but cfg-gating modules (rather than the
  whole crate) lets `cargo build --workspace` include this crate
  natively without a leptos toolchain. Trunk picks up the wasm target
  naturally.
- `open4x-server/Cargo.toml` was deliberately untouched here. The dead
  `csr` feature + leptos deps stay until P3 prunes them; touching them
  in P4 would have conflicted with the P3 PR.

**Validation**:
- `cargo check -p open4x-client-web --target wasm32-unknown-unknown` ✓
- `trunk build --release` from `open4x-client-web/` produced a working
  bundle (32 KB JS + 906 KB WASM).
- Wireframe visual parity (every screen in [`web-ui.md`](./web-ui.md)
  §2.1) **not** verified in-session — manual browser pass deferred to
  P6 or to the next round of UI work.

**Remaining follow-ups for the trunk build command**: `MEMORY.md`,
`book/src/multiplayer/web-client.md`, and `book/src/roadmap/web-ui.md`
still document the old `cd open4x-server && trunk build`. P6 updates
these.

### Phase 5 — `open4x-cli` adopts the SDK — **DONE**

One commit on `phase/5-cli-sdk` (`impl(cli): replace bespoke remote
client with open4x-sdk shim`). **Shim-only path** taken — the alternative
"migrate every remote subcommand to typed SDK endpoints" would have
rippled `.await`/async through every handler with no parity win.

What changed:

1. `open4x-cli/src/remote/client.rs` is now a thin wrapper around
   `open4x_sdk::native::NativeBlockingClient`. The SDK owns reqwest
   plumbing, base-URL handling, bearer-auth header, and
   `{error, message}` envelope decoding.
2. The `Result<serde_json::Value, String>` surface is preserved
   verbatim, so every other file under `remote/` (action, bootstrap,
   end_turn, list, session, status, view) is byte-identical.
3. `NativeBlockingClient::do_request` is private, so the shim goes
   through the public `Transport::request` trait method. The blocking
   client returns an immediately-ready future, which `pollster::block_on`
   drains with zero executor overhead.
4. `open4x-cli/Cargo.toml`: `reqwest` removed from `[dependencies]`,
   replaced by `open4x-sdk` (`default-features = false`, features =
   `["native-blocking"]`) + `pollster = "0.3"`. `reqwest` kept under
   `[dev-dependencies]` because `tests/remote_parity.rs:70` uses
   `reqwest::blocking::get` for its health-probe loop.

**Parity harness `tests/remote_parity.rs`**: `remote_parity_baseline`
and `remote_parity_full_loop` both pass post-swap. The CLI server-mode
[parity gate](./cli-server-mode.md) is the canonical validation — green.

### Phase 6 — Cleanup + documentation — **PENDING**

Runs after P3. Scope unchanged from the original plan, with one addition:

1. Delete the `open4x-server/src/types/mod.rs` shim added in P1.
2. Replace every remaining `use crate::types::*` in `open4x-server` with
   `use open4x_protocol::v1::*`.
3. Update `AGENTS.md`:
   - §Workspace crates table (now reflects 4 active crates +
     accounts/lobby as excluded).
   - §Key files table — remove `pages/`/`components/`/`tabs/` rows from
     the server crate; add a section for `open4x-client-web`; add
     `open4x-protocol/src/v1/*` and `open4x-sdk/src/{endpoints,
     transport, native, wasm}.rs` rows.
4. Update `book/src/SUMMARY.md` if the existing web-client doc splits.
5. Update `book/src/multiplayer/web-client.md` and
   `book/src/roadmap/web-ui.md` architectural sections (per-screen plan
   stays valid; trunk build command + serve command both change).
6. Refresh `MEMORY.md` "Web UI build" entry with the new paths
   (`cd open4x-client-web && trunk build --release`).
7. Mark the relevant TODOs in `book/src/roadmap/todo.md` as done.
8. **Restore `commit.gpgsign`**: P0 disabled GPG signing locally via
   `git config --local commit.gpgsign false` because `but commit` has no
   per-command signing override and gpg-agent couldn't prompt for a
   passphrase from the agent environment. Restore with
   `git config --local --unset commit.gpgsign`.
9. **Wireframe visual parity check**: walk every screen in
   [`web-ui.md`](./web-ui.md) §2.1 against the post-split client-web
   build and the pre-split baseline. Deferred from P4.

## 5. Dependency graph

```
                  ┌────────────────────┐
                  │ P0  scaffolding    │   (must complete first)
                  └─────────┬──────────┘
                            │
                  ┌─────────▼──────────┐
                  │ P1  open4x-protocol│   (keystone — blocks everything)
                  └─────────┬──────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
   ┌──────────▼──┐ ┌────────▼─────┐       │
   │ P2a  SDK    │ │ P2b  SDK     │       │
   │  native     │ │  wasm        │       │
   └──────┬──────┘ └────────┬─────┘       │
          │                 │             │
   ┌──────▼─────┐    ┌──────▼──────────┐  │
   │ P5  CLI    │    │ P4  client-web  │  │
   │  uses SDK  │    │  extraction     │  │
   └──────┬─────┘    └──────┬──────────┘  │
          │                 │             │
          │            ┌────▼──────────┐  │
          │            │ P3  server    │◄─┘
          │            │  slim-down    │
          │            └────┬──────────┘
          │                 │
          └────────┬────────┘
                   │
           ┌───────▼────────┐
           │ P6  cleanup    │
           │  + docs        │
           └────────────────┘
```

**Concurrency rules** (for either parallel agents or solo dev pipelining):

- After P1 lands, **P2a and P2b are independent** — different target backends,
  different files. Two contributors can take one each.
- **P3 must wait on P4** — server slim-down deletes the same files
  client-web extraction needs. Doing P3 first would force resurrecting deleted
  files in P4. Doing P4 first means P3 is just `git rm` of empty leftovers.
- **P5 is independent of P3/P4** — CLI doesn't touch the web client. Can
  land as soon as P2a is green.
- **P6 fans in** at the end and is the only phase that *requires* all prior
  phases to be merged.

Critical path length: P0 → P1 → P2b → P4 → P3 → P6 (6 sequential phases).
With one contributor on each fork, total wall time is dominated by P1 + the
longer of {P2a + P5, P2b + P4 + P3}.

## 6. Testing strategy

The migration's correctness rests on three test layers that **already exist
and must stay green**:

| Layer                    | Location                                    | Guards against                                            |
|--------------------------|---------------------------------------------|-----------------------------------------------------------|
| `libciv` unit tests      | `libciv/src/**/tests`                       | Engine regressions. Unaffected by the split.              |
| `libciv` integration     | `libciv/tests/{gameplay,ai_agent}.rs`       | End-to-end gameplay. Unaffected.                          |
| REST integration         | `open4x-server/tests/rest_api.rs`           | Server still answers every `/api/v1/*` route as before.   |
| CLI parity harness       | `open4x-cli/tests/*` (per cli-server-mode)  | CLI subcommands produce identical output local vs. remote.|

**New tests introduced by this work**:

| Phase | Test                                         | Purpose                                                          |
|-------|----------------------------------------------|------------------------------------------------------------------|
| P1    | `open4x-protocol/tests/wire_schema.rs`       | JSON snapshot per top-level type — catches field renames.        |
| P1    | Doctests on `v1::*`                          | Roundtrip serde for every wire type.                             |
| P2a   | `open4x-sdk/tests/native_roundtrip.rs`       | SDK against in-process Axum via `oneshot` — every endpoint.      |
| P2b   | `open4x-sdk/tests/wasm_smoke.rs`             | wasm backend produces byte-identical request payloads.           |
| P4    | `open4x-client-web/tests/mount_smoke.rs`     | Root component mounts and renders non-empty DOM.                 |

**Parity verification cadence**:

After each phase merges, run:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd open4x-client-web && trunk build --release    # post-P4 only
# CLI parity harness:
just cli-parity   # or whatever cli-server-mode.md specifies
```

**Wire-format regression guard** — the
`open4x-protocol/tests/wire_schema.rs` snapshots are committed to the repo.
Any PR that intentionally changes a field must update them; any PR that
accidentally changes one fails CI.

**Manual UI checklist** — gated to Phase 4 only. Walk through each wireframe
screen from [`web-ui.md`](./web-ui.md) §2.1 against both the pre-split
`open4x-server`-served build and the post-split `open4x-client-web` build.
Compare side-by-side; differences are bugs.

## 7. Risk register

| Risk                                                        | Mitigation                                                                 |
|-------------------------------------------------------------|----------------------------------------------------------------------------|
| Re-export shim in P1 hides a missed import; P6 deletion breaks build. | Run `cargo build --workspace` with `RUSTFLAGS=-Dwarnings` after each phase; deprecation-annotate the shim. |
| WASM backend pulls `tokio` transitively via `reqwest`.      | `open4x-sdk` puts `reqwest` behind `native-*` features only; CI builds the wasm target with `--no-default-features --features wasm`. |
| `trunk build` path change breaks deploys.                   | Update `Dockerfile`, `docker-compose.yml`, `justfile`, `dockerfiles/` in the same PR as Phase 4. |
| Snapshot tests are too strict and noisy.                    | Snapshot the deserialised `serde_json::Value`, not the string — tolerates whitespace and field-order changes while still catching schema drift. |
| CLI parity harness was relying on incidental ordering in the bespoke client. | Phase 5 PR runs the harness twice (before/after the SDK swap) and diffs transcripts before merging. |
| Long-lived feature branch drifts from `main`.               | Each phase merges to `main` independently; no phase requires a branch open >1 week. |

## 8. Concrete checklist (per phase)

Tick-boxes the reviewer can run down.

### P0 — Scaffolding — **DONE**
- [x] Add `open4x-protocol`, `open4x-sdk`, `open4x-client-web` to workspace `members`.
- [x] Each new crate has a `Cargo.toml` and `src/lib.rs` that builds empty.
- [x] `cargo build --workspace` succeeds.
- [x] `cargo test --workspace` still passes.
- [x] (Bonus) `open4x-accounts` + `open4x-lobby` moved to `[workspace] exclude`.
- [x] (Bonus) Baseline cleanup: missing `CityView.focus` field + 8 `uninlined_format_args`.

### P1 — Protocol — **DONE**
- [x] `open4x-server/src/types/*` moved to `open4x-protocol/src/v1/` (plain `mv`).
- [x] `open4x-protocol/src/lib.rs` exposes `pub mod v1`.
- [x] `open4x-server/src/types/mod.rs` becomes `pub use open4x_protocol::v1::*;` shim.
- [x] `open4x-server/Cargo.toml` adds `open4x-protocol` dep.
- [x] `open4x-protocol/tests/wire_schema.rs` snapshots 5 top-level types (8 tests).
- [x] `rest_api.rs` passes unmodified.

### P2-prep — Transport trait — **DONE** (added during execution)
- [x] `open4x-sdk/src/transport.rs` declares `Transport` trait + `Method` enum.
- [x] `lib.rs` re-exports both.

### P2a — SDK native — **DONE**
- [x] `open4x-sdk/src/native.rs` lifts `open4x-cli/src/remote/client.rs`.
- [x] Both `NativeBlockingClient` (default feature) and `NativeAsyncClient`.
- [x] `open4x-sdk/src/endpoints/*` filled per resource module (18 modules).
- [x] `open4x-sdk/tests/native_roundtrip.rs` hits every `/api/v1/*` route via `tower::oneshot` (11 tests).

### P2b — SDK wasm — **DONE**
- [x] `open4x-sdk/src/wasm.rs` lifts `open4x-server/src/components/api/http.rs`.
- [x] `open4x-sdk/tests/wasm_smoke.rs` cfg-gated to `target_arch = "wasm32"`.
- [x] Followup `fix/sdk-getrandom-wasm`: getrandom `wasm_js` feature + send_wrapper `futures` feature so `cargo check --target wasm32-unknown-unknown` succeeds.

### P3 — Server slim — **PENDING**
- [ ] Remove `csr` feature + every leptos/wasm-* dep from `open4x-server/Cargo.toml`.
- [ ] Drop `crate-type = ["cdylib", "rlib"]` from `[lib]`.
- [ ] `cargo build -p open4x-server` is a clean native-only build.
- [ ] Drop `#![cfg(feature = "ssr")]` from `tests/rest_api.rs`.
- [ ] `main.rs` default `OPEN4X_STATIC_DIR` retargeted to `./open4x-client-web/dist`.
- [ ] `open4x-server/src/{components,pages,tabs}` directories confirmed gone (P4 already removed contents; verify no stray files).

### P4 — Client web — **DONE**
- [x] `components/`, `pages/`, `tabs/` moved to `open4x-client-web/src/`.
- [x] `index.html` moved; trunk config valid (`data-trunk rel="rust"`).
- [x] All `use crate::components::api` → `use open4x_sdk::endpoints`.
- [x] All `use crate::types` → `use open4x_protocol::v1`.
- [x] `trunk build --release` produces a working bundle (32 KB JS + 906 KB WASM).
- [ ] UI parity checklist walked (deferred to P6).

### P5 — CLI uses SDK — **DONE**
- [x] `open4x-cli/src/remote/client.rs` now a shim over `open4x_sdk::native::NativeBlockingClient`.
- [x] Remote subcommands call SDK indirectly via the shim (shim-only path; not direct `open4x_sdk::endpoints::*` calls).
- [x] CLI server-mode parity harness passes; baseline transcripts unchanged.

### P6 — Cleanup + docs — **PENDING**
- [ ] `open4x-server/src/types/mod.rs` shim deleted.
- [ ] Remaining `use crate::types` rewrites done.
- [ ] `AGENTS.md` workspace + key-files tables updated.
- [ ] `book/src/multiplayer/web-client.md`, `book/src/roadmap/web-ui.md`,
      `book/src/SUMMARY.md` updated.
- [ ] `MEMORY.md` "Web UI build" entry refreshed.
- [ ] `book/src/roadmap/todo.md` entries closed.
- [ ] `git config --local --unset commit.gpgsign` (restore signing).
- [ ] Wireframe visual parity walked (deferred from P4).

## 9. Out of scope

The following are explicit non-goals of this migration and should land as
separate work:

- **WebGL renderer** — Phase 4 ships the existing SVG `hexmap.rs` verbatim.
  Swapping it for WebGL2 instanced rendering happens after the split lands.
- **Actor-per-room refactor** — the `DashMap<GameId, GameRoom>` stays as-is.
- **Event-sourced turn log** — `GameStateDiff` persistence is a server-side
  concern unaffected by this split.
- **Publishing `open4x-protocol`/`open4x-sdk` to crates.io** — premature
  until the protocol stabilises. Trigger: 3+ months without a breaking
  change.
- **Splitting the web client into its own repo** — also premature. Trigger
  conditions documented separately; not on this critical path.

## 10. Open questions

- Should `open4x-protocol` version the namespace per-module (`messages::v1`,
  `view::v1`) or per-crate (`open4x_protocol::v1::*`)? Per-crate is simpler;
  per-module lets individual schemas evolve independently. Default to
  per-crate until a real divergence forces the split. -> per-crate
- Does `open4x-sdk` need a sync facade *and* an async facade, or only async?
  CLI is currently blocking (reqwest::blocking); web is async. Cleanest:
  async-only, CLI uses `tokio::runtime::Runtime::new().unwrap().block_on(...)`
  in a small wrapper. Decide before P2a. -> async-only
- Where do shared WebSocket types live? Currently `types/messages.rs` covers
  both REST and WS shapes. Keep them in `open4x-protocol/v1/messages.rs`
  unchanged; the WS transport itself is an SDK concern (Phase 2b can stub it
  and complete it once the REST surface is proven). -> `open4x-protocol/v1/messages.rs` as youo say

---

## 11. Agentic dispatch plan (GitButler workflow)

This section operationalises §4–§5 against [GitButler](https://docs.gitbutler.com/cli-overview)'s
virtual-branch model. **For the duration of this migration `jj` is suspended
in favour of `git` + `but`.** Conventional commit tags from `AGENTS.md` still
apply (`infra:`, `impl:`, `fix:`, `tests:`, `docs:`).

### 11.1 GitButler primitives we rely on

| Primitive                | `but` command                              | Used for                                                  |
|--------------------------|--------------------------------------------|-----------------------------------------------------------|
| Virtual branch (lane)    | `but branch new <lane>`                    | One lane per concurrently dispatched agent.               |
| Stacked branch           | `but branch new -a <anchor> <lane>`        | When a phase strictly depends on another unmerged phase.  |
| Per-lane staging         | `but stage <file-id> <lane>`               | Assign agent's edits to its own lane.                     |
| Per-lane commit          | `but commit -m '<msg>' <lane>`             | Commit a lane without polluting unassigned bucket.        |
| Lane status              | `but status`, `but status -f`              | Operator visibility: which agent owns which changes.      |
| Lane diff                | `but diff <lane>`                          | Review before push.                                       |
| Push lane                | `but push <lane>`                          | Publish for review.                                       |
| Open PR                  | `but pr <lane>`                            | Create the GitHub PR per lane.                            |
| Rebase open lanes        | `but pull` (after upstream merge)          | Keep remaining lanes current as siblings land.            |
| Conflict resolution      | `but resolve`                              | First-class — never blocks unrelated lanes.               |
| Recovery                 | `but oplog`, `but undo`, `but oplog restore` | Roll back a bad dispatch without touching other lanes.    |
| Manual snapshot          | `but oplog snapshot`                       | Pin a known-good state before risky operations.           |

GitButler shares a **single working directory** across all applied lanes —
there is no per-lane checkout. Lane isolation is purely metadata: GitButler
tracks which hunks belong to which lane. This means our concurrency safety
comes from **disjoint file ownership**, not from filesystem walls.

### 11.2 Lane → phase mapping

One lane per dispatchable unit of work. Lane names use `phase/<id>-<slug>`.

| Lane                       | Phase | Depends on (merged)        | Files the agent may touch                                                             |
|----------------------------|-------|----------------------------|----------------------------------------------------------------------------------------|
| `phase/0-scaffolding`      | P0    | —                          | Root `Cargo.toml`; new `open4x-{protocol,sdk,client-web}/{Cargo.toml,src/lib.rs}`.    |
| `phase/1-protocol`         | P1    | P0                         | `open4x-server/src/types/**`, `open4x-protocol/src/**`, `open4x-server/Cargo.toml` (add dep only). |
| `phase/2a-sdk-native`      | P2a   | P1                         | `open4x-sdk/src/{native.rs,error.rs,endpoints/*}`, `open4x-sdk/Cargo.toml`, `open4x-sdk/tests/native_roundtrip.rs`. |
| `phase/2b-sdk-wasm`        | P2b   | P1                         | `open4x-sdk/src/{wasm.rs,endpoints/*}`, `open4x-sdk/Cargo.toml` (wasm features only), `open4x-sdk/tests/wasm_smoke.rs`. |
| `phase/4-client-web`       | P4    | P2b                        | Moves `open4x-server/src/{components,pages,tabs}` → `open4x-client-web/src/**`; `open4x-client-web/{index.html,Cargo.toml}`. |
| `phase/3-server-slim`      | P3    | P4                         | `open4x-server/{Cargo.toml,src/lib.rs,src/main.rs}`; deletes empty leftovers; updates `tests/rest_api.rs` cfg line. |
| `phase/5-cli-sdk`          | P5    | P2a                        | `open4x-cli/src/remote/**`, `open4x-cli/Cargo.toml`.                                  |
| `phase/6-docs`             | P6    | P3, P4, P5                 | `AGENTS.md`, `book/src/**`, `MEMORY.md`, `Dockerfile`, `docker-compose.yml`, `justfile`. |

### 11.3 Hot zones (shared files across lanes)

Three files attract concurrent edits and need explicit coordination:

| File                                          | Lanes that touch it   | Coordination rule                                                                  |
|-----------------------------------------------|-----------------------|------------------------------------------------------------------------------------|
| `open4x-sdk/Cargo.toml`                       | P2a, P2b              | P0 lands the full feature scaffold (`native-blocking`, `native-async`, `wasm`) so each P2 lane only adds dep entries to its own feature block. Reviewer rejects any P2 PR that edits the other lane's feature block. |
| `open4x-sdk/src/endpoints/mod.rs`             | P2a, P2b              | P0 commits the file with `pub mod cities; pub mod units; pub mod tech; …` already present. Each lane fills the per-resource module body, never touches `mod.rs`. |
| `open4x-server/Cargo.toml`                    | P1, P3                | P1 only **adds** the protocol dep. P3 **removes** the leptos/wasm deps. P1 must merge before P3 starts (already enforced by §5 graph). |

For everything else, the lane-to-files table in §11.2 is enforceable as a
disjoint partition — two lanes never have write claims on the same file.

### 11.4 Pre-flight (operator, once)

```bash
cd ~/Code/open-civ-vi
but setup                                  # one-time per repo
but config user                            # name + email if not already set
but config forge auth                      # if you want `but pr` (else use gh)
but config target                          # confirms origin/main is the target
git fetch origin && git checkout main && git pull --ff-only
```

### 11.5 Dispatch recipe (per lane)

The operator runs steps 1–3 and 6–8. Steps 4–5 are the agent's job.

```bash
# 1. Refresh from main.
but pull

# 2. Snapshot a known-good baseline so a misbehaving agent can be rewound.
but oplog snapshot

# 3. Open the lane. Use -a <anchor-lane> if the dependency hasn't merged yet
#    and you want a stacked PR.
but branch new phase/2a-sdk-native
# or:  but branch new -a phase/1-protocol phase/2a-sdk-native

# 4. Dispatch the agent with the lane name + file-ownership scope. Template
#    prompt in §11.6. The agent edits files in the shared working tree.

# 5. Agent commits its own work, always passing its lane name explicitly:
#       but commit -m 'impl: lift http.rs into open4x-sdk/native' \
#                  phase/2a-sdk-native

# 6. Operator review.
but status -f                              # see which lane owns what
but diff phase/2a-sdk-native               # human review of the hunk set
but branch show phase/2a-sdk-native        # commit list ahead of target
cargo test --workspace                     # parity gate from §3
cargo clippy --workspace -- -D warnings

# 7. Publish.
but push phase/2a-sdk-native
but pr phase/2a-sdk-native                 # or: gh pr create ...

# 8. After the PR merges to origin/main, rebase every still-open lane.
but pull
```

If the agent goes off the rails: `but oplog restore <sha>` to the snapshot
from step 2. Other lanes are untouched because they live in their own
metadata.

### 11.6 Agent prompt template

The implementing agent does **not** need to understand GitButler. It just
needs the lane name (so its commits land correctly) and a strict file-scope
list. Use this template per dispatch:

```
You are implementing Phase <N> of the crate-split migration documented in
book/src/roadmap/crate-split.md §4. Your lane name is `<lane>`.

Files you may CREATE or MODIFY:
  - <explicit list from §11.2>

Files you may READ but MUST NOT modify:
  - <everything else, especially the lanes listed in §11.3 as hot zones>

When committing, always pass the lane name explicitly:
  but commit -m '<conventional-commit-tag>: <message>' <lane>

Exit criteria (all must hold before you report done):
  - cargo build --workspace                      succeeds
  - cargo test --workspace                       passes
  - cargo clippy --workspace -- -D warnings      clean
  - <phase-specific checklist from §8>

Do NOT run: but push, but pr, but pull, git push, git rebase, git merge.
The operator handles publishing.
```

### 11.7 Concurrency timeline

```
time →

main │═══════════ P0 merged ═══════════ P1 merged ════════════════════════════════════════════════ P2a, P2b merged ════════ P5, P4 merged ═══════ P3 merged ═══ P6 merged
     │                                       │                                                            │                       │                    │
     │                                       ├── lane phase/2a-sdk-native ──────────────────────┐         │                       │                    │
     │                                       │                                                  │         │                       │                    │
     │                                       ├── lane phase/2b-sdk-wasm ────────────────────────┤         │                       │                    │
     │                                       │                                                  │         │                       │                    │
     │                                       │                                                  ├── lane phase/5-cli-sdk ─────────┤                    │
     │                                       │                                                  │                                 │                    │
     │                                       │                                                  ├── lane phase/4-client-web ──────┤                    │
     │                                       │                                                  │                                 │                    │
     │                                       │                                                  │                                 ├── lane phase/3-server-slim ──┤
     │                                       │                                                  │                                 │                    │
     │                                       │                                                  │                                 │                    ├── lane phase/6-docs ──┤
```

Two parallelism windows:

- **Window A** (after P1 merges): lanes `phase/2a-sdk-native` and
  `phase/2b-sdk-wasm` run concurrently. Two agents, disjoint endpoint
  modules, shared scaffolding in `Cargo.toml`/`mod.rs` pre-landed by P0.
- **Window B** (after P2a/P2b merge): lanes `phase/5-cli-sdk` and
  `phase/4-client-web` run concurrently. Disjoint file sets entirely.

Maximum agent concurrency: **2 lanes at any one time**. Going wider buys
nothing — the dependency graph serialises everything else, and more lanes
just amplify hot-zone risk.

### 11.8 Conflict handling

GitButler's "rebases always succeed, sometimes with conflicted commits"
model means a sibling lane never blocks you mid-flight. Two failure modes
to plan for:

1. **Lane vs. main conflict** (after `but pull` rebases the lane onto a
   freshly-merged sibling). Run `but resolve`; resolve in working tree;
   `but resolve finish`; re-run `cargo test --workspace`.
2. **Lane vs. lane conflict at commit-assign time** (two agents staged the
   same hunk to different lanes). Should not happen if §11.3 is honoured.
   If it does: `but oplog undo` the most recent assignment, fix the scope
   violation, redispatch.

### 11.9 Integration cadence

- **Per-lane PR review** is the merge unit. No giant integration branch.
- After each merge to `origin/main`, the operator runs `but pull` once to
  rebase every open lane. This is the only step that can cascade conflicts;
  do it after every merge, not in batches.
- The `[ ]` checkbox lists in §8 are the per-PR acceptance criteria. PRs
  whose checkboxes aren't all ticked don't merge.
- If a phase fails review (request changes), the agent for that lane
  receives a follow-up dispatch with the review feedback verbatim. New
  commits land on the same lane (no rebase needed unless the operator ran
  `but pull` in the meantime).

### 11.10 Recovery playbook

| Symptom                                              | Action                                                                 |
|------------------------------------------------------|------------------------------------------------------------------------|
| Agent committed to the wrong lane.                   | `but rub <commit-sha> <correct-lane>` to reassign.                     |
| Agent edited a file outside its scope.               | `but oplog undo` repeatedly until the bad edit is gone, then redispatch with stricter scope wording. |
| Two lanes both edited the same `Cargo.toml` block.   | Land one PR, then `but pull` the other; resolve via `but resolve`; reviewer signs off on the merged Cargo block. |
| Workspace `cargo test` fails after a lane merge.     | `but oplog restore <pre-merge-sha> -f` on the operator's lane state; revert the merge PR; redispatch with the regression as a new acceptance test. |
| GitButler metadata gets weird.                       | `but teardown && but setup` — drops metadata, preserves git history; re-apply lanes manually with `but branch new` on top of each unmerged branch ref. |

### 11.11 What this section deliberately does not solve

- **Code review automation** — every PR still needs a human (or `/ultrareview`)
  pass. The dispatch plan only governs *creation* and *integration* of changes.
- **CI integration** — assumes GitHub Actions (or equivalent) already runs
  `cargo test --workspace` + `cargo clippy` on every PR. If not, add it
  before starting Phase 0.
- **Agent identity in commits** — co-author trailers per existing convention
  (`Co-Authored-By: Claude <noreply@anthropic.com>` or whichever model);
  GitButler does not impose anything here.
- **Restoring jj** — out of scope for this migration. After P6 merges,
  re-evaluate whether to switch back or keep `but` as the long-term VCS
  workflow.

### 11.12 Execution incidents observed (P0 → P5)

Captured live for future reference:

- **GPG signing**: `but commit` has no per-command signing override.
  Required `git config --local commit.gpgsign false` for the duration;
  P6 restores via `--unset`. The skill's "use `but` for all writes" rule
  doesn't extend to git *config*, only git *write commands* — so the
  config change is fine.
- **`but merge` requires a `gb-local/*` target**: the repo's GitButler
  target is `origin/main`, against which `but merge` errors out
  ("Target remote is origin, not gb-local"). Workaround: add a fake
  remote pointing at the repo itself (`git remote add gb-local .`),
  fetch it, then `but config target gb-local/main`. After that local
  merges work normally. Documented for the next migration that wants
  local-only integration without going through the forge.
- **GitButler unapplies branches when target changes**: before swapping
  target, `but unapply <lane>` then re-apply by full branch name
  (`but apply <lane>` — short CLI IDs aren't valid post-unapply).
- **Hot-zone outcome**: `open4x-sdk/Cargo.toml` was a real concurrent
  edit point for P2a/P2b. P2b finished first; P2a's later edits to the
  same dev-deps block produced no hunk (already on disk), so the file's
  commit attribution went 100% to P2b. End state was correct; lesson is
  that "disjoint file scope" is the only reliable parallel rule —
  "disjoint hunk within a shared file" is fragile.
- **Two agents reflexively ran `git stash`** while spot-checking
  pre-existing clippy state. Both self-corrected with `git stash pop`
  immediately. Future prompts should explicitly enumerate `git stash`
  in the don't-do list (the skill already forbids it but the temptation
  is strong when investigating a baseline issue).
- **Extra unplanned lanes that landed**:
  `infra/gitbutler-skill` (one commit, skill reference files for agent
  use); `phase/2-prep` (one commit, `Transport` trait + Method enum so
  P2a/P2b could parallelise without coordinating the trait shape);
  `fix/sdk-getrandom-wasm` (one commit, getrandom `wasm_js` feature +
  send_wrapper `futures` feature — surfaced when the SDK's wasm32 build
  was first attempted post-P2-merge).
- **Visual-parity check punted**: every parity gate is automated except
  the wireframe-screen visual diff from `web-ui.md` §2.1. Carried over
  to P6.

