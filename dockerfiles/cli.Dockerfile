# open4x-cli image — paired with the server image to drive a parity
# harness over HTTP. The entrypoint forwards every arg to the
# `open4x` binary and reads `OPEN4X_SERVER_URL` so callers don't have
# to repeat `--server <URL>` on every invocation.
#
# Build context: workspace root.

# ── builder ────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config build-essential ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY . .

RUN cargo build --release -p open4x-cli

# ── runtime ────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /work
COPY --from=builder /src/target/release/open4x /usr/local/bin/open4x
COPY dockerfiles/cli-entrypoint.sh /usr/local/bin/cli-entrypoint.sh
RUN chmod +x /usr/local/bin/cli-entrypoint.sh

# Defaults wired up by docker-compose.yml; safe overrides for one-off
# `docker run` invocations against a server on the host network.
ENV OPEN4X_TOKEN_FILE=/work/.open4x-session.json

ENTRYPOINT ["/usr/local/bin/cli-entrypoint.sh"]
