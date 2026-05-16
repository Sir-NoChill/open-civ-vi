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

