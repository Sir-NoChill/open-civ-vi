# Crate Split — Extract Protocol, SDK, and Web Client from `open4x-server`

> **Goal**: split the dual-purpose `open4x-server` crate (Axum SSR + Leptos
> CSR + shared wire types, ~9 800 LOC, 30+ optional deps gated by mutually
> exclusive `ssr`/`csr` features) into four single-purpose crates inside the
> existing workspace, with **zero behavioural regression** at every step.
>
> **Status**: planning. No code moves until Phase 0 lands.

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

### Phase 0 — Workspace scaffolding (mechanical)

Add three empty crates to `Cargo.toml`, no code moves yet.

- `open4x-protocol/`: `lib.rs` with `pub mod v1;` and a single `v1/mod.rs`
  stub. `Cargo.toml` depends only on `serde`, `serde_json`, `ulid`.
- `open4x-sdk/`: `lib.rs` with two cfg-gated backend modules:
  `#[cfg(not(target_arch = "wasm32"))] mod native;` and
  `#[cfg(target_arch = "wasm32")] mod wasm;`. Empty stubs.
- `open4x-client-web/`: `lib.rs` with `cdylib` crate-type, single
  `wasm_bindgen(start)` stub. `Cargo.toml` matches the current
  `open4x-server` csr feature set.

**Exit criterion**: `cargo build --workspace` succeeds, all four crates show
in `cargo metadata`. Server and CLI unchanged.

### Phase 1 — Extract `open4x-protocol`

The keystone. Every later phase depends on it.

1. `git mv open4x-server/src/types/* open4x-protocol/src/v1/` (preserves blame).
2. `open4x-protocol/src/lib.rs` re-exports the v1 namespace:
   ```rust
   pub mod v1 {
       pub use crate::coord::*;
       // …
   }
   ```
3. `open4x-server/src/types/mod.rs` becomes a thin shim:
   ```rust
   pub use open4x_protocol::v1::*;
   ```
   This keeps every internal `use crate::types::messages::GameAction;` working
   for the duration of the migration. Removed in Phase 6.
4. `open4x-server/Cargo.toml` adds `open4x-protocol = { path = "../open4x-protocol" }`.

**Tests added**:
- `open4x-protocol/tests/wire_schema.rs` — snapshot JSON for representative
  values of `GameView`, `WorldSnapshot`, `PlayerState`, `MutationResponse`,
  `ApiErrorBody`. Catches accidental field renames.
- Doctests on key types showing roundtrip serde.

**Exit criterion**: `cargo test --workspace` green; `rest_api.rs` untouched.

### Phase 2 — Build `open4x-sdk` (two backends, parallelisable)

The SDK has two source materials already:

- **Native** — `open4x-cli/src/remote/client.rs` (reqwest::blocking) is a
  ready-made template. Lift it into `open4x-sdk/src/native.rs`, then add a
  parallel async client (`reqwest::Client`) for the server-side tests.
- **WASM** — `open4x-server/src/components/api/http.rs` plus the per-resource
  modules (`cities.rs`, `units.rs`, `tech.rs`, …) are the WASM transport.
  Lift `http.rs` into `open4x-sdk/src/wasm.rs`; lift each typed call into
  `open4x-sdk/src/endpoints/*` so both backends call the same function.

Final SDK shape:

```
open4x-sdk/src/
  lib.rs              # pub use endpoints::*; backend selector
  endpoints/
    cities.rs         # async fn get_cities(client: &Client, …) -> Result<…>
    units.rs
    tech.rs
    …
  native.rs           # impl Client over reqwest
  wasm.rs             # impl Client over web_sys::fetch
  error.rs            # ApiError, unified across backends
```

`endpoints/*` functions are generic over a `Transport` trait that both
backends implement; the function bodies serialise the request via
`open4x-protocol` types and return typed results.

**Cargo features**:
- `default = ["native-blocking"]`
- `native-blocking` (reqwest::blocking)
- `native-async` (reqwest)
- `wasm` (web-sys, wasm-bindgen-futures) — for the `wasm32` target

**Tests added**:
- `open4x-sdk/tests/native_roundtrip.rs` — spin up the Axum router via
  `tower::ServiceExt::oneshot` (same pattern as `rest_api.rs`), wrap it in a
  `Transport` shim, hit every endpoint. Verifies the SDK round-trips every
  route the server exposes.
- `open4x-sdk/tests/wasm_smoke.rs` — `wasm-bindgen-test` against a mocked
  `fetch`. Verifies the wasm backend produces identical request payloads.

**Sub-phases that can run concurrently after P1 lands**:
- **2a** native backend (drives CLI integration; also unblocks server tests
  that exercise the SDK in-process).
- **2b** wasm backend (unblocks client-web extraction).

### Phase 3 — Slim `open4x-server`

After SDK and protocol exist, the server has no reason to host UI code or
expose a `cdylib`.

1. Delete `open4x-server/src/components/`, `pages/`, `tabs/`.
2. Drop the `csr` feature, drop `crate-type = ["cdylib", "rlib"]`, drop every
   `dep:leptos`/`dep:wasm-*`/`dep:web-sys`/`dep:js-sys` from
   `open4x-server/Cargo.toml`.
3. Delete `wasm_bindgen(start)` from `lib.rs`.
4. Make `ssr` the only build path; collapse the feature into the default deps
   (or drop the feature flag altogether).
5. Keep `OPEN4X_STATIC_DIR` plumbing in `main.rs`; the served `dist/` now
   comes from `open4x-client-web/dist/`.

**Tests**: `cargo test -p open4x-server` runs `rest_api.rs` unchanged.

**Risk**: the server crate's `tests/rest_api.rs` currently has `#![cfg(feature = "ssr")]`
at the top — drop that line, since the feature no longer exists.

### Phase 4 — Extract `open4x-client-web`

Move the UI half wholesale.

1. `git mv open4x-server/src/components open4x-client-web/src/components`
   (same for `pages/`, `tabs/`).
2. Replace every `use crate::types::*` with `use open4x_protocol::v1::*`.
3. Replace every `crate::components::api::*` call with the equivalent
   `open4x_sdk::endpoints::*` function.
4. Move trunk's entry-point HTML (`open4x-server/index.html`) to
   `open4x-client-web/index.html`.
5. Move the `.cargo/config.toml` `getrandom_backend="wasm_js"` cfg if it
   needs to be crate-local (current setting at repo root is fine — keep it).
6. Update the trunk build command in `MEMORY.md`, `book/src/multiplayer/web-client.md`,
   and `book/src/roadmap/web-ui.md`:
   ```bash
   cd open4x-client-web && trunk build --release
   # serve via:
   OPEN4X_STATIC_DIR=$PWD/open4x-client-web/dist target/release/open4x-server
   ```

**Tests**:
- Existing UI tests (if any in `open4x-server/tests/` were csr-gated — there
  aren't, based on current inventory) move to `open4x-client-web/tests/`.
- Add a `wasm-bindgen-test` smoke test that mounts the root component against
  a mocked SDK transport and asserts the initial render produces non-empty
  DOM.
- Manual checklist (one-time): every wireframe screen listed in
  [`web-ui.md`](./web-ui.md) §2.1 renders identically before/after.

### Phase 5 — `open4x-cli` adopts the SDK

The CLI's `src/remote/client.rs` is the prototype for the native SDK. After
Phase 2 lands, replace it with a thin shim over `open4x_sdk::native`.

1. Delete `open4x-cli/src/remote/client.rs` (the bespoke `ApiClient`).
2. Update `open4x-cli/src/remote/{action,end_turn,list,status,view,session,bootstrap}.rs`
   to call `open4x_sdk::endpoints::*` directly.
3. Add `open4x-sdk = { path = "../open4x-sdk", features = ["native-blocking"] }`
   to `open4x-cli/Cargo.toml`.

**Tests**: the CLI server-mode parity harness in
[`cli-server-mode.md`](./cli-server-mode.md) is the validation. If those
baseline transcripts still match, the SDK is wire-compatible.

### Phase 6 — Cleanup + documentation

1. Delete the `open4x-server/src/types/mod.rs` shim added in Phase 1.
2. Replace every remaining `use crate::types::*` in `open4x-server` with
   `use open4x_protocol::v1::*`.
3. Update `AGENTS.md`:
   - §Workspace crates table (lines 38–44).
   - §Key files table — remove `pages/`/`components/`/`tabs/` rows from the
     server crate, add a new section for `open4x-client-web`.
4. Update `book/src/SUMMARY.md` to add a "Client (Web)" page if the existing
   web-client doc needs splitting.
5. Update `book/src/multiplayer/web-client.md` and
   `book/src/roadmap/web-ui.md` architectural sections (the per-screen plan
   stays valid).
6. Refresh `MEMORY.md` "Web UI build" entry with the new paths.
7. Mark the relevant TODOs in `book/src/roadmap/todo.md` as done.

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

### P0 — Scaffolding
- [ ] Add `open4x-protocol`, `open4x-sdk`, `open4x-client-web` to workspace `members`.
- [ ] Each new crate has a `Cargo.toml` and `src/lib.rs` that builds empty.
- [ ] `cargo build --workspace` succeeds.
- [ ] `cargo test --workspace` still passes (no new tests yet).

### P1 — Protocol
- [ ] `git mv open4x-server/src/types/* open4x-protocol/src/v1/`.
- [ ] `open4x-protocol/src/lib.rs` exposes `pub mod v1`.
- [ ] `open4x-server/src/types/mod.rs` becomes `pub use open4x_protocol::v1::*;`.
- [ ] `open4x-server/Cargo.toml` adds `open4x-protocol` dep.
- [ ] `open4x-protocol/tests/wire_schema.rs` snapshots key types.
- [ ] `rest_api.rs` passes unmodified.

### P2a — SDK native
- [ ] `open4x-sdk/src/native.rs` ports `open4x-cli/src/remote/client.rs`.
- [ ] `open4x-sdk/src/endpoints/*` per resource module.
- [ ] `open4x-sdk/tests/native_roundtrip.rs` hits every `/api/v1/*` route.

### P2b — SDK wasm
- [ ] `open4x-sdk/src/wasm.rs` ports `open4x-server/src/components/api/http.rs`.
- [ ] Endpoints share the same signatures across both backends.
- [ ] `open4x-sdk/tests/wasm_smoke.rs` runs under `wasm-pack test`.

### P3 — Server slim
- [ ] Remove `csr` feature, `cdylib` crate-type, all Leptos/wasm deps.
- [ ] Delete `components/`, `pages/`, `tabs/` (post-P4).
- [ ] `rest_api.rs` `#![cfg(feature = "ssr")]` cfg line removed.
- [ ] `cargo build -p open4x-server` is a clean native-only build.

### P4 — Client web
- [ ] `components/`, `pages/`, `tabs/` moved to `open4x-client-web/src/`.
- [ ] `index.html`, trunk config moved.
- [ ] All `use crate::components::api` → `use open4x_sdk::endpoints`.
- [ ] All `use crate::types` → `use open4x_protocol::v1`.
- [ ] `trunk build --release` produces a working bundle.
- [ ] UI parity checklist walked.

### P5 — CLI uses SDK
- [ ] `open4x-cli/src/remote/client.rs` deleted.
- [ ] Remote subcommands call `open4x_sdk` directly.
- [ ] CLI server-mode parity harness passes; baseline transcripts unchanged.

### P6 — Cleanup + docs
- [ ] `open4x-server/src/types/mod.rs` shim deleted.
- [ ] Remaining `use crate::types` rewrites done.
- [ ] `AGENTS.md` workspace + key-files tables updated.
- [ ] `book/src/multiplayer/web-client.md`, `book/src/roadmap/web-ui.md`,
      `book/src/SUMMARY.md` updated.
- [ ] `MEMORY.md` "Web UI build" entry refreshed.
- [ ] `book/src/roadmap/todo.md` entries closed.

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
