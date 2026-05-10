# Reverse Proxy

Open4X is two pieces: the **lobby** (`open4x-lobby`, the public-facing
pre-game surface — login, ongoing games, wizard) and the **game
server** (`open4x-server`, an instance-per-room runtime). In a real
deploy you put a reverse proxy in front so the public origin terminates
TLS, sets `X-Forwarded-For` correctly, and hands off to the right
backend.

This page covers two deployment topologies — **shared mode** (one
game-server process serving every game) and **per-game mode** (the
lobby spawns a fresh `open4x-server` per game on a port from a
configured range) — with worked Caddy and nginx configs for each.

---

## Trusted proxies + `X-Forwarded-For`

Whatever you pick, the lobby needs to know which TCP peers are
*your* reverse proxy so it can read the real client IP out of the
forwarded header. Without this its rate-limiter and audit log
attribute every request to the proxy itself, which makes per-IP
throttles useless.

```env
OPEN4X_LOBBY_TRUSTED_PROXIES=127.0.0.0/8,::1/128,10.0.0.0/8
```

CIDR list, comma-separated. When the direct peer matches one of the
listed networks, the lobby walks `X-Forwarded-For` right-to-left and
returns the first hop that *isn't* in the trusted list. Empty
(default) = always ignore the header.

The proxy must:

1. Set `X-Forwarded-For` to the client (most do this by default).
2. Forward `Host` so magic-link emails build correct URLs (set
   `OPEN4X_LOBBY_PUBLIC_URL` if your client doesn't see `Host`
   right).

---

## Topology 1 — Shared game server

```
                ┌─ open4x-lobby :3001 ─┐
   Internet ──► proxy ──┤                   │
                ├─ open4x-server :3001 (one) │
                └────────────────────────────┘
```

Lobby's `OPEN4X_GAME_SERVER_URL` points at the single backend; every
game lives in the same process as a separate `GameRoom`. Resume
hands the browser the bearer token; the browser hits the same origin
the lobby is on, switching to the in-game UI by URL.

This is the simplest setup, and it's what the lobby defaults to.

### Caddy (shared)

```caddyfile
lobby.example.com {
    encode zstd gzip
    reverse_proxy 127.0.0.1:3001
}

play.example.com {
    encode zstd gzip
    reverse_proxy 127.0.0.1:3002
}
```

```env
OPEN4X_GAME_SERVER_URL=http://127.0.0.1:3002
OPEN4X_LOBBY_PUBLIC_URL=https://lobby.example.com
OPEN4X_LOBBY_TRUSTED_PROXIES=127.0.0.0/8
```

### nginx (shared)

```nginx
server {
    listen 443 ssl http2;
    server_name lobby.example.com;
    # ssl_certificate / ssl_certificate_key handled elsewhere

    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host              $host;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 443 ssl http2;
    server_name play.example.com;

    location / {
        proxy_pass http://127.0.0.1:3002;
        proxy_set_header Host              $host;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        # The game server's /ws endpoint speaks WebSocket.
        proxy_http_version 1.1;
        proxy_set_header Upgrade    $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
    }
}
```

(The standard `map $http_upgrade $connection_upgrade { …; }` block
goes in `nginx.conf`.)

---

## Topology 2 — Process-per-game

```
                ┌─ open4x-lobby :3001 ──────────┐
   Internet ──► proxy ──┤                                 │
                │       └──────────► spawns:              │
                │                    open4x-server :4501  │
                │                    open4x-server :4502  │
                │                    …                    │
                └─────────────────────────────────────────┘
```

The lobby spawns one `open4x-server` per game on a port allocated
from `OPEN4X_LOBBY_PORT_RANGE`. Every spawned child is bound to
`127.0.0.1` only; it's the proxy's job to expose the right child to
the public.

The lobby writes a **public** URL into the `games.server_url` row at
create time. By default that's the loopback URL (works for localhost
dev), but in any real deployment you point it at your proxy via:

```env
OPEN4X_LOBBY_PUBLIC_GAME_URL_TEMPLATE=https://g-{port}.example.com
# or, for path-prefix routing:
OPEN4X_LOBBY_PUBLIC_GAME_URL_TEMPLATE=https://example.com/play/{port}
```

`{port}` is replaced with the per-game allocated port at spawn time.
Resume returns the templated URL + token; the browser navigates
directly to the public URL and the proxy bridges to the right
loopback port.

You also need a wildcard / catch-all on the proxy that maps the
public form back to the loopback port.

### Caddy (per-game subdomain)

```caddyfile
lobby.example.com {
    reverse_proxy 127.0.0.1:3001
}

g-{port}.example.com {
    @port path *
    reverse_proxy 127.0.0.1:{re.port.port}
}

# Caddy doesn't natively support "extract a number out of a host
# label" without a matcher plugin. The portable form is to enumerate
# the range explicitly (matches OPEN4X_LOBBY_PORT_RANGE):
g-4501.example.com { reverse_proxy 127.0.0.1:4501 }
g-4502.example.com { reverse_proxy 127.0.0.1:4502 }
# … (one per port in the configured range)
```

```env
OPEN4X_LOBBY_PER_GAME=1
OPEN4X_LOBBY_GAME_BINARY=/usr/local/bin/open4x-server
OPEN4X_LOBBY_PORT_RANGE=4501-4600
OPEN4X_LOBBY_PUBLIC_GAME_URL_TEMPLATE=https://g-{port}.example.com
OPEN4X_LOBBY_TRUSTED_PROXIES=127.0.0.0/8
```

> **TLS for wildcard subdomains** — point a `*.example.com` DNS
> record at the proxy and use a wildcard TLS cert (Let's Encrypt
> via DNS-01). Without that, every per-game subdomain needs its own
> cert which doesn't scale.

### nginx (per-game path-prefix)

Path-prefix is easier than wildcard subdomains because nginx
extracts the port via a `location` regex:

```nginx
server {
    listen 443 ssl http2;
    server_name example.com;

    # Lobby on the root.
    location / {
        proxy_pass http://127.0.0.1:3001;
        proxy_set_header Host              $host;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Per-game routing: /play/<port>/...
    # Capture the port and proxy to 127.0.0.1:<port>. The trailing
    # path is preserved so SPA + REST + WS all reach the right
    # backend.
    location ~ ^/play/(?<game_port>4[5-9][0-9]{2})(?<rest>/.*)?$ {
        proxy_pass http://127.0.0.1:$game_port$rest;
        proxy_set_header Host              $host;
        proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        # WebSocket support for /play/<port>/ws.
        proxy_http_version 1.1;
        proxy_set_header Upgrade    $http_upgrade;
        proxy_set_header Connection $connection_upgrade;
    }
}
```

Match the regex range to your `OPEN4X_LOBBY_PORT_RANGE`. The example
above accepts ports `4500-4999` (`4[5-9][0-9]{2}`); tighten as
appropriate. **Important**: do not let users reach arbitrary
loopback ports — pin the regex to the configured range.

```env
OPEN4X_LOBBY_PER_GAME=1
OPEN4X_LOBBY_PORT_RANGE=4501-4600
OPEN4X_LOBBY_PUBLIC_GAME_URL_TEMPLATE=https://example.com/play/{port}
OPEN4X_LOBBY_TRUSTED_PROXIES=127.0.0.0/8
```

---

## Health-checks

Both the lobby and every spawned game server expose a `GET /health`
endpoint. The lobby's `/health` pings its sqlite pool and returns 503
on DB failure — wire it to your load-balancer's health check. Game
server `/health` is a static `ok`.

---

## Future extensions (not yet implemented)

- **Lobby-fronted proxy**: the lobby itself proxies
  `/api/v1/games/{id}/play/*` to the right per-game backend. This
  would remove the wildcard-subdomain / regex-route requirement on
  the external proxy. Tracked as a follow-up to Phase 6.
- **Public URL lookup over DNS-SD / k8s services** for orchestrators
  that hand out non-loopback addresses (different host, different
  zone). Today the orchestrator assumes "child runs on the same host
  as the lobby."
