# Docker harness — CLI ⇄ server parity testing

This stack pairs `open4x-server` (API-only, no SPA) with `open4x-cli`
(running in `--server` mode) so you can drive both sides of the
client/server projection from one shell. It's the runtime piece of
the plan in [`book/src/roadmap/cli-server-mode.md`](../book/src/roadmap/cli-server-mode.md).

## Files

| File | Purpose |
|---|---|
| `server.Dockerfile` | Multi-stage build of `open4x-server` with `--features ssr --no-default-features` (no Leptos/WASM). Final image is `debian:bookworm-slim` + the binary + `ca-certificates`. |
| `cli.Dockerfile` | Multi-stage build of `open4x-cli`. Includes `cli-entrypoint.sh` so positional args after `docker compose run cli` go straight to the binary. |
| `cli-entrypoint.sh` | `exec /usr/local/bin/open4x "$@"` — keeps signals sane (binary is PID 1). |
| `docker-compose.yml` | `server` + `cli` services on a private network; CLI reads `OPEN4X_SERVER_URL=http://server:3001`. |

## Usage

```bash
cd dockerfiles

# One-time (or after edits to libciv / open4x-cli / open4x-server)
docker compose build

# Bring up the server in the background
docker compose up -d server

# Bootstrap a game — writes /work/.open4x-session.json inside the
# `cli` container, persisted in the named volume.
docker compose run --rm cli new-game \
  --width 30 --height 18 --seed 1 \
  --player Rome --ai Babylon

# Read endpoints
docker compose run --rm cli status yields
docker compose run --rm cli status pending
docker compose run --rm cli list   units
docker compose run --rm cli list   cities
docker compose run --rm cli view

# Mutations
docker compose run --rm cli action move --unit <ULID> --to-q 5 --to-r 3
docker compose run --rm cli action research --tech Pottery
docker compose run --rm cli end-turn

# Tear it all down (drops the session volume — pass `-v` to also wipe it)
docker compose down
```

Because `OPEN4X_SERVER_URL` is set in compose, the CLI never needs an
explicit `--server` flag. Every command writes JSON to stdout.

## Hitting the same server from the host

The `server` service publishes `3001:3001`, so you can also drive it
without compose:

```bash
cargo run -p open4x-cli -- --server http://localhost:3001 new-game \
  --width 30 --height 18 --seed 1 --player Rome --ai Babylon
cargo run -p open4x-cli -- --server http://localhost:3001 status yields
```

This is the easiest path while iterating on the CLI itself — only
the server needs the slow Docker rebuild.

## Parity diff (manual)

Today this is a copy-paste loop; an automated diff lands in Phase 3
of the plan. The shape is:

```bash
# Local mode
cargo run -p open4x-cli -- new-game   --game-file /tmp/g.json \
                                      --player Rome --ai Babylon
cargo run -p open4x-cli -- status     --game-file /tmp/g.json \
                                      --player Rome yields > /tmp/local.json

# Remote mode
cargo run -p open4x-cli -- --server http://localhost:3001 \
                            new-game --player Rome --ai Babylon
cargo run -p open4x-cli -- --server http://localhost:3001 \
                            status yields > /tmp/remote.json

diff /tmp/local.json /tmp/remote.json
```

Expected divergences (yield bucket shape, ID rendering, etc.) are
listed at the bottom of [`cli-server-mode.md`](../book/src/roadmap/cli-server-mode.md);
anything outside that list is a regression.
