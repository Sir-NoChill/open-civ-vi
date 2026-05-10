# Accounts and Login — `open4x-lobby` + `open4x-accounts`

> **Status**: Phase 0 complete (workspace split + paper-theme SPA scaffold +
> screen ports). All subsequent phases pending.
> **Goal**: A self-hostable pre-game surface (landing → login → menu → new
> game wizard → profile) that authenticates real users, manages their
> identities, lists their ongoing games, and orchestrates per-game
> `open4x-server` instances. Distinct from the in-game runtime — see
> §1 for the rationale.

---

## 1. Why this exists

The Open4X stack splits cleanly into two concerns:

| Concern             | Lifetime                  | Persistence              | Crate                |
|---------------------|---------------------------|--------------------------|----------------------|
| In-game runtime     | one game session          | in-memory `GameRoom`     | `open4x-server`      |
| Pre-game surface    | spans many games          | account / identity DB    | `open4x-lobby`       |
| Identity substrate  | shared between the above  | account / identity DB    | `open4x-accounts`    |

`open4x-server` is single-purpose: hold one `GameRoom`, mutate via
`/api/v1/*`, project to clients. Auth there is anonymous bearer tokens
minted by `POST /games/new`. That's correct for the in-game runtime; it's
inadequate for a multi-game lobby where users come back over weeks, link
their email + GitHub + atproto to one identity, manage friends, and
resume games on multiple devices.

The lobby owns: auth (email magic-link, OIDC, atproto), user identity
graph, ongoing-games index, new-game wizard, profile and preferences. It
is the **only** thing the user touches before they're in a game.

Visual reference: `docs/open4x-landing/project/hifi/`. Paper theme, IBM
Plex Mono / Sans / Serif. Distinct from the in-game dark theme by design
— stepping from lobby into a game is a deliberate context switch.

---

## 2. Inventory

### 2.1 Phase 0 — done

- [x] Workspace members `open4x-lobby` (dual ssr/csr, mirrors
      `open4x-server` Cargo layout) and `open4x-accounts` (plain library)
- [x] `open4x-accounts` type skeleton: `PlayerId` (16-hex dot-grouped),
      `Identity` enum (Email · OpenId · Atproto), `Account`, `Preferences`,
      `MagicLinkToken`
- [x] `open4x-lobby` Leptos SPA scaffold: `index.html` with IBM Plex
      Google-Fonts links, `styles.css` copied verbatim from
      `docs/open4x-landing/project/hifi/styles.css`
- [x] Shared primitives ported: `Btn`, `Tag`, `Toggle`, `Segmented`,
      `Panel` / `PanelHead`, `MiniMap`. Popup is stubbed as `Trigger`
      (renders the underlined help-trigger as a `<span class="trigger">`
      with a `title=` fallback)
- [x] Screens — full ports: **Landing**, **Login**, **Profile**
- [x] Screens — partial ports: **MenuShell** (sidebar + secondary-nav
      placeholders), **OngoingGames** (tile grid with sample data baked
      in), **NewGame** (5-step strip + Map step + Review step; Civ /
      Rules / Players steps render placeholder bodies)
- [x] App shell with `RwSignal<Screen>` route switcher (Landing | Login |
      Menu) + nested `RwSignal<MenuTab>` (Ongoing | NewGame | Profile)
- [x] Trunk + Axum boot path — `trunk build --release --features csr
      --no-default-features` produces a 709 KB wasm + 24 KB hashed CSS;
      `cargo run -p open4x-lobby` serves on `:3002`

### 2.2 Pending

Everything else. See §4.

---

## 3. End-state architecture

```
                                       ┌─ DB (sqlite default · pg opt) ─┐
                                       │   accounts · identities ·      │
                                       │   sessions · games_index       │
                                       └────────────▲───────────────────┘
                                                    │
┌─ open4x-accounts (lib) ──────────────┐  reads /   │
│                                      │  writes    │
│   PlayerId · Identity · Account      │◄───────────┘
│   MagicLink minting · OIDC client    │
│   atproto resolver · session tokens  │
│                                      │
└──────────▲────────────────────▲──────┘
           │                    │
           │ uses               │ validates inbound tokens
           │                    │
┌──────────┴───────┐   ┌────────┴──────────────┐
│ open4x-lobby     │   │ open4x-server         │
│ (axum + leptos)  │   │ (axum + leptos)       │
│                  │   │                       │
│ /landing /login  │   │ /api/v1/* + /ws       │
│ /menu /api/v1/me │   │ in-game runtime       │
│ /api/v1/games ──►│   │                       │
│ orchestrates     │──►│ POST /games/new       │
│ per-game server  │   │ (token-validated)     │
└──────────────────┘   └───────────────────────┘
```

The accounts crate is the substrate. The lobby crate consumes it for the
pre-game surface and exposes the orchestration HTTP. The in-game server
trusts tokens minted by the accounts crate (via shared signing key in
the simple case, or HTTP introspection later).

For single-machine dev: lobby and one or more in-game servers run as
separate processes on adjacent ports; the lobby reverse-proxies or
redirects clients to the right `:3001+n` URL per game. For multi-host
deploys: same model with the orchestration layer talking to a process
manager (systemd / podman / k8s) instead of `Command::spawn`.

---

## 4. Phased plan

Each phase ends in a working, mergeable state. None requires more than
one accounts PR + one lobby PR.

### Phase 1 — Visual completeness

Get the SPA pixel-perfect against the design before wiring anything to
the network. No persistence, no auth, no orchestration; just finishing
the JSX → Leptos translation so we can iterate UX without backend
plumbing.

- [ ] **Popup component** — Gwern-style hover preview with pin-on-click
      / esc-to-close / click-outside-to-dismiss / smart positioning.
      Replace every `<Trigger>` stub call site. Source:
      `docs/open4x-landing/project/hifi/popup.jsx` (165 LOC).
- [x] **`PopupBody`, `PopupActions`, `PopupList`** — body containers
      live at `open4x-lobby/src/components/popup_body.rs`. `PopupBody`
      / `PopupActions` are pure layout wrappers; `PopupList` takes a
      `Vec<PopupListItem>` (`Row { icon, label, desc? }` or
      `Separator`) so menu definitions read structurally rather than
      as untyped JSX. No interactivity yet — wired up when the Popup
      shell lands.
- [ ] **NewGame `StepCiv`** — civ picker grid (8 leaders), per-leader
      `CivSheet` popup with unique unit / unique building / leader
      ability / civ ability. Selection state stored on `RwSignal`.
- [ ] **NewGame `StepRules`** — difficulty Segmented (settler → deity),
      starting era, game speed, victory toggles ×6, world dynamics
      sliders (disasters, barbarians, city-states, AI aggression, AI
      personality). `Slider` component port.
- [ ] **NewGame `StepPlayers`** — player slot list with invite popup
      (paste email / OpenID / atproto / player ID), turn-mode params
      (timer, simultaneous, private, cross-play).
- [x] **`Slider` primitive** — wraps `<input type="range">` with an
      optional `format: Arc<dyn Fn(i32) -> String>` callback for the
      value display. `RwSignal<i32>`-driven; `min` / `max` default to
      `0` / `100`. Lives at
      `open4x-lobby/src/components/slider.rs`.
- [ ] **Tweaks panel port** — runtime density toggle (`compact` /
      `comfortable` / `spacious`) wired to the `data-density` attr on
      `.app`. Already on the JSX side; just port the React component.

**Done when**: every screen in `docs/open4x-landing/project/hifi/`
renders identically to the JSX prototype (ignoring purposeful Leptos
restructures). All popups work. Tweaks panel toggles density live.

### Phase 2 — `open4x-accounts` substrate

The crate today is types-only. This phase puts persistence and token
minting behind those types.

#### 2.1 Persistence

- [ ] Pick storage: **sqlx + sqlite by default**, postgres opt-in.
      Migrations live in `open4x-accounts/migrations/`.
- [ ] Schema:
  ```
  accounts(player_id PRIMARY KEY, preferred_name, pronouns, bio,
           prefs_json, created_at, updated_at)
  identities(id PRIMARY KEY, player_id FK, kind, primary_key,
             label, verified, created_at)
            -- primary_key: address (email) | issuer+sub (oidc) | did (atproto)
            -- UNIQUE(kind, primary_key)
  sessions(token_hash PRIMARY KEY, player_id FK, created_at,
           expires_at, revoked_at)
  ```
- [ ] `AccountStore` trait + sqlite impl + in-memory test impl.
      Methods: `lookup_by_identity`, `link_identity`, `unlink_identity`,
      `find_or_create_account_for_identity`, `update_profile`,
      `delete_account` (cascades).

#### 2.2 Magic-link tokens

- [ ] `MagicLinkToken` mint / verify via HMAC-SHA256 over
      `(email, expires_at, nonce)`. Server-side nonce-once table to
      prevent reuse. 15-minute expiry.
- [ ] Email transport: pluggable `Mailer` trait. Default `LogMailer`
      (writes the magic link to stderr — useful for dev). SMTP impl
      gated on a `mailer-smtp` feature.

#### 2.3 OIDC client

- [ ] `OidcProvider` config (issuer URL, client_id, client_secret,
      redirect_uri, scopes). Built-in pre-configs for Google · GitHub ·
      GitLab · Microsoft.
- [ ] OIDC discovery (`/.well-known/openid-configuration`) cached per
      issuer.
- [ ] Authorization-code flow with PKCE.
- [ ] ID-token verification (signature, iss, aud, exp, nonce).
- [ ] Claims → `Identity::OpenId{issuer, subject, label}` mapping.
      `label` derived from `preferred_username` or `email` claims when
      available.
- [ ] Custom-issuer flow — accept arbitrary issuer URL, run discovery
      live (cache aggressively after first success).

#### 2.4 atproto

- [ ] Handle resolution: try `_atproto.<handle>` DNS TXT, fall back to
      `https://<handle>/.well-known/atproto-did`.
- [ ] DID resolution: PLC directory for `did:plc:`, web for `did:web:`.
      Returns the user's PDS endpoint.
- [ ] OAuth flow against the PDS (atproto-oauth / DPoP signed
      requests).
- [ ] Map identity to `Identity::Atproto{did, handle}`.

#### 2.5 Session tokens

- [ ] Bearer-token shape: `lobby_<base64url(48 bytes)>`. Stored as
      SHA-256 hash in `sessions.token_hash` (so the DB compromise can't
      mint logins).
- [ ] `mint_session(player_id, ttl) -> RawToken`.
- [ ] `validate_session(raw) -> Option<PlayerId>` — constant-time
      compare against the hash table.
- [ ] `revoke_session(raw)`.

**Done when**: `cargo test -p open4x-accounts` covers the magic-link,
OIDC-claims-mapping, and session-token flows with a fake mailer + a
mocked OIDC discovery server. CLI tool to mint a session for a manually
inserted account, useful for the Phase 3 lobby work.

### Phase 3 — Lobby HTTP surface

Bring the SPA online. Each Leptos screen replaces its baked-in sample
data with a real REST call.

#### 3.1 Auth routes

| Method | Path                                       | Body / Query                          | Effect                                                |
|--------|--------------------------------------------|---------------------------------------|-------------------------------------------------------|
| POST   | `/api/v1/auth/email/start`                 | `{email}`                             | Mint magic-link, hand to `Mailer`                     |
| GET    | `/api/v1/auth/email/verify`                | `?token=…`                            | Validate, find-or-create account, set session cookie  |
| GET    | `/api/v1/auth/oidc/{provider}/start`       | —                                     | 302 to provider authorize URL (PKCE state in cookie)  |
| GET    | `/api/v1/auth/oidc/{provider}/callback`    | `?code&state`                         | Code exchange, verify ID token, link identity         |
| POST   | `/api/v1/auth/oidc/custom/start`           | `{issuer_url}`                        | Discovery + 302 like above                            |
| POST   | `/api/v1/auth/atproto/start`               | `{handle_or_did}`                     | Resolve PDS, 302 to PDS authorize                     |
| GET    | `/api/v1/auth/atproto/callback`            | `?code&state`                         | DPoP code exchange, link identity                     |
| POST   | `/api/v1/auth/signout`                     | —                                     | Revoke current session                                |

#### 3.2 Account routes

| Method | Path                                      | Returns / Effect                                    |
|--------|-------------------------------------------|-----------------------------------------------------|
| GET    | `/api/v1/me`                              | `Account` for the authenticated session             |
| PATCH  | `/api/v1/me`                              | Update `preferred_name`, `pronouns`, `bio`, `prefs` |
| POST   | `/api/v1/me/identities/email`             | Start linking-flow for a new email (magic-link)     |
| POST   | `/api/v1/me/identities/oidc/{provider}`   | Start linking-flow for an OIDC provider             |
| DELETE | `/api/v1/me/identities/{id}`              | Unlink — refuse if it would orphan the account      |
| POST   | `/api/v1/me/identities/{id}/primary`      | Mark email identity as primary (one per account)    |
| POST   | `/api/v1/me/avatar`                       | Multipart upload (PNG/JPG, ≤2MB), pipeline TODO     |
| DELETE | `/api/v1/me`                              | Delete account (cascades; see §6)                   |

#### 3.3 SPA wiring

- [ ] `components/api/{auth,me,identities}.rs` bindings.
- [ ] `Login` screen: email panel POSTs to `/auth/email/start`, OIDC
      buttons trigger `/auth/oidc/{provider}/start` redirects, custom
      OIDC popup posts to `/auth/oidc/custom/start`, atproto panel
      POSTs to `/auth/atproto/start`. Show "Magic link sent — check your
      inbox" feedback.
- [ ] Magic-link landing page (`/auth/email/verify`) — server-rendered
      redirect to `/menu` (or to the SPA's Menu route via hash).
- [ ] `Profile` screen: fetch `/me` on mount, populate fields, push
      changes via `PATCH /me`. Linked-identities list fed by `/me`.
      Identity unlink confirmations.
- [ ] Session-cookie middleware on the axum side: parse cookie, attach
      `PlayerId` to request extensions; auth-required endpoints reject
      with 401 + structured `{error: "no_session"}`.

**Done when**: the user can sign in with email or any of the four
pre-configured OIDC providers, see their account, link a second
identity, and sign out. Profile preferences round-trip via `/me`.

### Phase 4 — Games index + orchestration

The Menu currently shows hard-coded sample games. This phase puts a
games index behind `/api/v1/games` and wires the New-Game wizard to
actually create one.

#### 4.1 Schema

```
games(game_id PRIMARY KEY, owner_player_id FK,
      name, leader, civ_id, difficulty, players_human, players_ai,
      map_type, map_size, seed, turn, era, score, status,
      server_url, server_token, last_played_at, created_at, deleted_at)
game_members(game_id FK, player_id FK, role, invited_at, joined_at)
PRIMARY KEY (game_id, player_id)
```

#### 4.2 Routes

| Method | Path                              | Effect                                                 |
|--------|-----------------------------------|--------------------------------------------------------|
| GET    | `/api/v1/games`                   | List games visible to the user (own + invited)         |
| POST   | `/api/v1/games`                   | New-game wizard submit — bootstrap & return `game_id`  |
| GET    | `/api/v1/games/{id}`              | Single-game preview                                    |
| POST   | `/api/v1/games/{id}/notes`        | Update markdown notes                                  |
| POST   | `/api/v1/games/{id}/invite`       | Invite by email / OpenID / atproto / `PlayerId`        |
| DELETE | `/api/v1/games/{id}`              | Resign / archive / delete                              |
| POST   | `/api/v1/games/{id}/resume`       | Returns 302 to the in-game server URL with a token     |

#### 4.3 Orchestrator

- [ ] Pick a model: **shared-server-multi-room** as v1 (the existing
      `open4x-server` already keys `GameRoom` by id; just teach it to
      validate accounts-issued tokens). Process-per-game is a Phase 6
      enhancement.
- [ ] Server-side change: `open4x-server` accepts a session token
      issued by `open4x-accounts` (shared HMAC key in the simple
      single-machine setup; HTTP introspection later). The anonymous
      `POST /games/new` becomes opt-in via env var so guest play
      stays available in dev.
- [ ] Lobby `POST /api/v1/games` translates the wizard's params into a
      call to the in-game server's `POST /api/v1/games/new`,
      records the returned `game_id` in the lobby DB.
- [ ] `GET /api/v1/games/{id}/resume` mints a per-game scoped token,
      302s to the in-game server URL with the token in a query param
      (or short-lived session-bridge cookie).

#### 4.4 SPA wiring

- [ ] `OngoingGames` reads `/api/v1/games`, renders real tiles with
      MiniMap thumbnails seeded by the actual game seed (not the row
      index).
- [ ] Filter chips functional (`your_turn` / `waiting` / `completed` /
      `multiplayer`) — push filter to query params.
- [ ] Search box filters client-side over names + notes.
- [ ] Sort dropdown.
- [ ] "Resume" CTA → `GET /api/v1/games/{id}/resume` → follow redirect.
- [ ] `+ New game` wizard's "⌬ Generate world" → `POST /api/v1/games`
      → on success, redirect to in-game URL.
- [ ] Notes popup: markdown textarea, persisted via
      `POST /api/v1/games/{id}/notes`.
- [ ] Per-tile `···` menu: View summary / Copy game ID / Share invite
      link / Archive / Resign.

**Done when**: a logged-in user can run the wizard end-to-end and find
themselves in a freshly-created `open4x-server` `GameRoom` they own,
return to the lobby, see it in their list, and resume it on a different
device after signing in there.

### Phase 5 — Polish

User-visible quality once the platform basics are wired.

- [ ] **Avatar pipeline** — multipart upload, image-rs decode, downscale
      to 256×256 PNG, store under `accounts.avatars/`. Profile shows
      the uploaded image instead of the initial.
- [ ] **Show invite QR popup** — render the player ID as a QR code
      (`qrcode` crate, base64 PNG). Copy player-ID action.
- [ ] **Friends screen** — search-by-identity, friend requests, friends
      list. Schema additions.
- [ ] **Presets screen** — save / load / import-JSON wizard configs.
- [ ] **Docs screen** — embeds the rendered mdBook (this very book).
- [ ] **Email verification flow** — "Verify email" CTA on unverified
      identities, second magic-link to confirm.
- [ ] **Sign-in feedback states** — pending / success / failure for
      every entry point.
- [ ] **Real game tile thumbnails** — `MiniMap` re-seeded by the game's
      world seed so you actually recognise your saves.
- [ ] **Tile thumbnails reflect ownership** — show captured cities,
      explored vs unexplored, etc., when the snapshot is cheap to
      fetch.
- [ ] **i18n hooks** — gettext-style; pulled from the in-game roadmap.

### Phase 6 — Self-host + ops

Everything that's not a feature but is required to run this in
production.

- [ ] **Single-binary deploy** — `open4x-lobby` ships with embedded
      migrations, sqlite default DB path under `OPEN4X_DATA_DIR`.
- [ ] **Postgres opt-in** — connection string via env, sqlx feature
      flag.
- [ ] **SMTP config** — env-driven (`SMTP_HOST`, `SMTP_PORT`,
      `SMTP_USER`, `SMTP_PASS`, `SMTP_FROM`). Default no-op in dev.
- [ ] **Rate limiting** — magic-link mint per email + per IP, OIDC
      callback per state cookie.
- [ ] **Audit log** — append-only table of auth events (sign-in,
      identity link/unlink, account delete) for incident response.
- [ ] **Account deletion (GDPR)** — purge identities + sessions,
      anonymize game records (replace `owner_player_id` with a
      tombstone), document retention policy.
- [ ] **Process-per-game orchestrator** — for deployments that want
      isolation. `Command::spawn` an `open4x-server` per game, track
      pid + port + health.
- [ ] **Reverse-proxy story** — example nginx + caddy configs that
      route `/api/v1/games/{id}/play/*` to the right per-game backend.
- [ ] **Backup & restore** — `lobby db dump` / `lobby db restore`
      subcommands.

---

## 5. Cross-cutting decisions to lock in

These don't fit cleanly into one phase but need to be settled early.

- **Identity uniqueness.** `(kind, primary_key)` is the unique tuple.
  An email cannot be linked to two accounts. An OIDC `(issuer, sub)`
  cannot be linked to two accounts. atproto `did` cannot be linked to
  two accounts. **Implication**: signing in with an already-linked
  identity always lands you in the same account — no merging UI in v1.
- **Account merging — out of scope.** If a user has two accounts (one
  from email, one from GitHub) and wants them merged, that's a manual
  operator action in v1.
- **Primary email.** Exactly one `Identity::Email` per account may have
  `primary = true`. Used as the `from`/`to` for system mail and as the
  default Gravatar source. The "Set as primary" button enforces.
- **Cookies vs. bearer tokens.** Lobby uses **httpOnly secure
  cookies** for browser sessions. The in-game server uses **bearer
  tokens** for its REST surface. The Resume flow bridges by minting a
  short-lived bearer signed by the same key.
- **CSRF.** All state-changing routes require a double-submit cookie
  token. Magic-link verify is GET-with-query (intentional — links must
  work from any client) but consumes a single-use server-side nonce.
- **Tile coordinates / wire types.** Reuse `open4x-server`'s wire types
  where possible; do not duplicate `WorldSnapshot` etc. The lobby's
  game-creation flow round-trips the wizard's params straight into the
  server's existing `POST /api/v1/games/new`.

---

## 6. What this plan deliberately does NOT do

- **No multiplayer protocol changes.** The in-game `/ws` stays
  untouched. Real-time multiplayer is out of scope for accounts/login.
- **No payment / premium tier.** Open source, AGPL.
- **No mobile apps.** The SPA is desktop-first, mobile-okay; native
  apps are not on this roadmap.
- **No federation between Open4X instances.** Each lobby is its own
  trust boundary. Cross-instance identity (e.g. atproto-as-bridge) is
  a future thing.
- **No anti-abuse beyond rate limiting.** No CAPTCHA, no fraud
  scoring. Self-hosted instances pick their own posture.

---

## 7. Open questions

- **Tweaks panel scope.** Density is the obvious one to wire. Color
  scheme (paper / ink / auto) needs a CSS-vars dark variant before it's
  meaningful. Defer ink mode to Phase 5?
- **In-game server token format.** HMAC-shared-key (simple, requires
  config sync) vs. HTTP introspection against
  `open4x-accounts` (resilient, but adds a hop). v1 = HMAC; v2 may
  switch.
- **atproto OAuth maturity.** The spec is still moving. We may need to
  ship a "sign in via app password" fallback if OAuth churn breaks the
  flow during Phase 2.

---

## 8. Phasing summary

| Phase | Theme                              | Done when                                                                   | Blocks                       |
|-------|------------------------------------|-----------------------------------------------------------------------------|------------------------------|
| 0     | Scaffolding ✅                     | Workspace split + paper SPA renders all screens (some w/ placeholder data)  | —                            |
| 1     | Visual completeness                | Pixel-perfect against design; popups + slider work; remaining wizard steps  | none — pure UI               |
| 2     | `open4x-accounts` substrate        | Magic-link, OIDC client, atproto resolver, session tokens, sqlite store     | Phase 3                      |
| 3     | Lobby HTTP + auth wiring           | Sign in via 3 methods, link identities, profile round-trip via `/me`        | Phase 4                      |
| 4     | Games index + orchestration        | New-game wizard creates a real `GameRoom`, ongoing list reflects reality    | Phase 5                      |
| 5     | Polish                             | Avatar, QR, friends, presets, real thumbnails, email verify                 | Phase 6                      |
| 6     | Self-host + ops                    | Single-binary deploy, SMTP, rate limit, audit log, process-per-game         | —                            |

---

## 9. Changelog

Running record of work performed against this plan, newest at top.

### Phase 0 — Scaffolding (2026-05-10)

- Workspace `[workspace.members]` updated to add `open4x-accounts` and
  `open4x-lobby`. Both build clean alongside the existing `libciv` /
  `open4x-server` / `open4x-cli` triumvirate.
- `open4x-accounts/src/lib.rs` ships the type skeleton: `PlayerId(u64)`
  with the dot-grouped hex `Display`, `Identity` enum (Email · OpenId ·
  Atproto), `Account` (preferred_name / pronouns / bio / identities /
  prefs), `Preferences` (density · color_scheme · keyboard_nav ·
  turn_notifications · discoverable_by_id), `MagicLinkToken` newtype.
  Two unit tests cover `PlayerId::display` and identity labels.
- `open4x-lobby/Cargo.toml` mirrors `open4x-server`'s ssr/csr feature
  gate. `[lib]` is `cdylib + rlib`, `[[bin]]` is gated on `ssr`.
- `open4x-lobby/index.html` pulls in IBM Plex Mono / Sans / Serif via
  Google Fonts and links `styles.css` through Trunk's `data-trunk`
  asset pipeline (emits a `styles-<hash>.css` on build).
- `open4x-lobby/styles.css` is the design's CSS copied verbatim from
  `docs/open4x-landing/project/hifi/styles.css` (798 lines, no
  modifications).
- `open4x-lobby/src/components/` ships Leptos ports of the JSX
  primitives: `Btn` (primary / accent / ghost / bare; xs / sm / lg /
  block), `Tag`, `Toggle` (`RwSignal<bool>`-driven), `Segmented`
  (with a `Segment {value, label}` row type), `Panel` + `PanelHead`,
  `MiniMap` (deterministic LCG matching the JSX), and `Trigger`
  (placeholder for the not-yet-ported Gwern popup).
- `open4x-lobby/src/screens/` ships Landing (full port — ASCII banner,
  serif headline, two CTAs, EMAIL · OPENID · ATPROTO triggers,
  footer), Login (full port — three stacked auth panels), MenuShell
  (sidebar + secondary nav), OngoingGames (tile grid with the design's
  sample data baked in), NewGame (5-step strip; Map step + Review step
  ported, Civ / Rules / Players steps render placeholder bodies),
  Profile (full port — avatar / quick actions / fields / linked
  identities / preferences).
- `open4x-lobby/src/app.rs` ships the `App` component: `app-bar`
  (brand · HI-FI pill · Landing/Login/Menu nav · online dot · `?` kbd
  hint) plus a `RwSignal<Screen>` driving which screen mounts below.
  `MenuShell` nests a `RwSignal<MenuTab>` for the in-menu sub-nav.
- `open4x-lobby/src/main.rs` is a small Axum binary that serves
  `dist/` via tower-http `ServeDir` and exposes `/health`. Defaults to
  port 3002; static dir comes from `OPEN4X_LOBBY_STATIC_DIR`.
- Smoke test: `trunk build --release --features csr
  --no-default-features` produces 709 KB wasm + 24 KB hashed CSS.
  `target/release/open4x-lobby` boots, serves the SPA at `/`, the wasm
  + css at their fingerprinted URLs, and `/health` returns `ok`.
- Known limitations carried into Phase 1: `Trigger` stubs the popup;
  Wizard's middle three steps are placeholders; OngoingGames + Profile
  use baked-in sample data; no auth, no orchestration.
