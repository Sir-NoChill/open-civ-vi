# Ongoing Work

> **Current task**: Web UI port — post-plan extensions
> **Plan**: [book/src/roadmap/web-ui.md](./web-ui.md)
> **Status**: All 5 phases of §7 complete. Working through the post-plan
> backlog below. This file is a running log — each cron tick should pick the
> next unchecked item, land a single conventional commit, then update this
> file.
> **Cron**: `04ad50ff` — fires hourly at :13 (session-only, expires
> after 7 days). Cancel with `CronDelete 04ad50ff` when the backlog is
> empty or no longer wanted.

## Web UI port (Leptos + REST)

- [x] Phase 0 — Scaffolding
- [x] Phase 1 — HUD MVP (HexMap rebind deferred)
- [x] Phase 2 — server reads + writes (libciv extensions deferred)
- [x] Phase 3 — server tech/civics surface (libciv extensions + tabs deferred)
- [x] Phase 4 — server outer-loop reads + NotificationRecord ring buffer
      (libciv pending_actions / victory_progress + tabs/drawers deferred)
- [x] Phase 5 — Cleanup
  - [x] Move open4x-webui/ → docs/legacy-wireframe/
  - [x] Drop /api/game/* legacy routes (server/{api,reports}.rs
        modules retained as dead code; full deletion below)
  - [x] Integration tests in open4x-server/tests/rest_api.rs (10 tests)
  - [x] Document API in book/src/multiplayer/web-client.md

## Remaining (post-plan)

These items were deferred from earlier phases. Pick the top unchecked
item next.

### libciv RulesEngine extensions

Strategy: where it fits, expose the new method through `open4x-cli`'s
`status` / `list` subcommand first (cheap testing arena), then plumb
into the server projector.

- [x] `pending_actions(gs, civ) -> Vec<PendingAction>` — replaced the
      hand-rolled "choose_research" check; new
      `build_turn_queue_from_room` calls into the engine and surfaces
      `choose_research` / `choose_civic` (required) plus per-unit and
      per-city advisory items. Exposed via CLI `status pending`.
- [x] `victory_progress(gs) -> Vec<VictoryProgress>` — server session
      now registers all 6 standard conditions; new
      `build_victory_from_room` overlays engine percentages onto the
      stable wire shape. Exposed via CLI `status victory`.
- [x] `available_unit_actions(gs, unit) -> Vec<UnitAction>` — 8-kind
      enum (Move/Attack/Fortify/Sleep/FoundCity/Build/TradeRoute/
      SpreadReligion); new `build_units_from_room` calls into the
      engine per own unit and maps to wire shape. Exposed via CLI
      `status unit-actions --id <ulid>`.
- [x] `preview_combat(gs, attacker, defender_coord) -> CombatPreview` —
      mirrors `attack()`'s effective-CS pipeline (promotions,
      government, policies, GP auras, religion, terrain/walls/siege)
      at rng=1.0; new `build_combat_preview_from_room` calls the
      engine. Exposed via CLI `status combat-preview`.
- [x] `policy_catalogue(gs, civ) -> Vec<PolicyCardEntry>` — walks
      `state.policies` (127 entries) and labels each Active /
      Available / Locked from the civ's `active_policies` /
      `unlocked_policies`. New `build_government_from_room` populates
      the catalogue. Exposed via CLI `status policies`.

### GameAction variants + REST mutations

- [x] `GameAction::AssignCityFocus` + `POST /cities/{id}/focus` —
      `CityFocus` enum (Default/Food/Production/Gold/Science/Culture/
      Faith) on `City`; engine stores the value (auto-assignment
      heuristic not yet driven by it). Wire round-trip via
      `city_data::CityRow.focus`. CLI: `action assign-city-focus`.
- [x] `GameAction::RenameCity` + `POST /cities/{id}/rename` —
      writes `City.name` after trim + 1..=64 char validation +
      ownership check; structured 400 on empty / oversize. CLI:
      `action rename-city`.
- [x] `GameAction::CancelResearch` + `DELETE /tech/research` —
      idempotent pop of `research_queue` front; partial progress
      discarded (matches Civ VI switch-research semantics). CLI:
      `action cancel-research`.
- [x] `GameAction::CancelCivic` + `DELETE /civics/research` —
      idempotent clear of `civic_in_progress` Option; partial progress
      discarded. CLI: `action cancel-civic`.
- [x] `GameAction::ChangeGovernment` + `POST /government/change` —
      switches `current_government` after unlock check; mirrors
      `OneShotEffect::AdoptGovernment`'s policy-eviction logic.
      Structured 400 on unknown / locked / empty. CLI: existing
      `action adopt-government` covers the arena case.

### Client (Leptos)

- [ ] HexMap refactor to consume `WorldSnapshot` directly (currently
      RestGamePage shows a placeholder "World w × h · N tiles" line)
- [ ] `tabs/city.rs` — extend existing, REST-driven
- [ ] `tabs/units.rs` — new
- [ ] `tabs/government.rs` — new
- [ ] `tabs/diplomacy.rs` — new
- [ ] `tabs/empire.rs` — new
- [ ] `tabs/victory.rs` — new
- [ ] drawers: notifications / turn-queue / overlays

### Cleanup

- [ ] Full deletion of `server/{api,reports}.rs` and `types/reports.rs`

## Accounts and Login — ACTIVE

> **Plan**: [book/src/roadmap/accounts-and-login.md](./accounts-and-login.md)
> **Status**: Phase 0 + Phase 1 complete. Phase 2 (`open4x-accounts`
> substrate — sqlite store + magic-link minting + OIDC client +
> atproto resolver + session tokens) starting next.
> **Cron**: `dfdcd4f5` — fires every 10 minutes (session-only, expires
> after 7 days). `CronDelete dfdcd4f5` to cancel.

### In progress
_(this section is the running tracker — items here are picked up by the
next loop tick; mark items done in `accounts-and-login.md` and delete
from this list when complete)_

- [ ] **Phase 5 ▸ Avatar pipeline** — multipart upload on
      Profile, image-rs decode, downscale to 256×256 PNG,
      store under `accounts.avatars/<player_id>.png`. Profile
      shows the uploaded image instead of the initial-letter
      avatar. New `POST /api/v1/me/avatar` route + an
      `avatar_url` field on the MeView wire shape so the SPA
      can read it back.

### Up next (Phase 6)

- [x] **CLI: `open4x-accounts dump-audit`** — `[[bin]]
      open4x-accounts` (gated on `persistence`) ships a clap-driven
      ops binary. `--db <path>` (default `./data/lobby/accounts.sqlite`)
      and `--limit <n>` (default 100). Prints TSV with the hex
      PlayerId display format, never wraps long detail strings,
      escapes embedded tabs/newlines for grep-ability.
- [ ] **Single-binary deploy** — embed migrations + serve dist/
      from the binary's bundled assets (rust-embed) so a deploy is
      just `open4x-lobby` + an `OPEN4X_LOBBY_DATA_DIR`.

### Up next (Phase 5 polish)

- [ ] Real game tile thumbnails (snapshot per server_token, cached).
- [ ] Email verification flow ("Verify email" CTA on unverified
      identities → second magic-link confirm).

- [ ] Per-tile `···` menu: View summary / Copy game ID / Share
      invite link / Archive / Resign.
- [ ] Notes popup: markdown textarea, persisted via
      `POST /api/v1/games/{id}/notes` (route is also still TODO).
- [ ] Sort dropdown (recent ↓ / oldest / by score / by turn).
- [ ] Push filter selection into URL query params for
      shareability + back-button.
- [ ] Phase 4.3 orchestrator — shared-server-multi-room v1 (teach
      open4x-server to validate accounts-issued tokens; lobby
      `POST /games` translates wizard params into a server-side
      bootstrap call).
- [ ] Phase 4.4 SPA wiring — `OngoingGames` reads `/api/v1/games`,
      `+ New game` "⌬ Generate world" → `POST /api/v1/games`.

### Up next (deferred Phase 3 items)

- [ ] OIDC provider buttons in Login wire to
      `GET /auth/oidc/{provider}/start` (depends on Phase 2.3 part 2).
- [ ] Profile "+ link another" + per-identity "unlink" buttons
      wire to the Phase 3.2 identity routes (depends on those
      handlers landing).

### Up next (deferred Phase 2 items)

- [ ] **Phase 2.3 ▸ OIDC code exchange + ID-token verify** — needs
      `openidconnect` + `reqwest` and a mocked-discovery test
      harness. Land after Phase 3 surface stabilises so the lobby
      handler shape is settled before the network plumbing.
- [ ] **Phase 2.3.x ▸ GitHub OAuth2 helper**.
- [ ] **Phase 2.4 ▸ atproto handle/DID resolver + OAuth/DPoP flow**.

### Phase 2 summary

Phase 2.1 ✅ persistence · 2.2 ✅ magic-link + mailer · 2.5 ✅ sessions
· 2.3 partial (config + auth-URL builder ✅; exchange + verify next).
Phase 3 starts now — middleware first, then the email-start /
email-verify pair (which can ship end-to-end against the existing
substrate), then `/me`. Deferred Phase 2 items pick up after Phase 3
demonstrates the integration shape.

## Civsim Non-REPL CLI — ALL 5 PHASES COMPLETE

- [x] Phase 0–5 (see git log).
- 554 tests, 0 failures.

## Changelog (post-plan)

Most recent first. Each entry: `<jj change short> — <subject>`.

- `ownpwskt` — feat(open4x-server): GameAction::ChangeGovernment +
  POST /government/change.
- `nttqzwzx` — feat(open4x-server): GameAction::CancelCivic + DELETE
  /civics/research.
- `oumzyopv` — feat(open4x-server): GameAction::CancelResearch +
  DELETE /tech/research.
- `lynqxskw` — feat(open4x-server): GameAction::RenameCity + POST
  /cities/{id}/rename.
- `pkmywxno` — feat(libciv,open4x-server): GameAction::AssignCityFocus
  + POST /cities/{id}/focus.
- `szyyxqnn` — feat(libciv): RulesEngine::policy_catalogue + populate
  /government catalogue.
- `uvzvovyo` — feat(libciv): RulesEngine::preview_combat + wire
  through web combat-preview.
- `tsmktrmt` — feat(libciv): RulesEngine::available_unit_actions +
  wire through web units.
- `lmuswpsy` — feat(libciv): RulesEngine::victory_progress + register
  6 conditions in server session.
- `lrrxwtmv` — feat(libciv): RulesEngine::pending_actions + wire
  through web turn-queue + CLI `status pending`.
- `qrykmkqp` — feat(open4x-server): NotificationRecord ring buffer +
  DELETE handlers (post-plan).
