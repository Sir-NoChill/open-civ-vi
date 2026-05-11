#!/bin/sh
# Forward every argument to the `open4x` binary. The `--server` flag is
# implicit via the OPEN4X_SERVER_URL env var (clap reads it through the
# `env = "OPEN4X_SERVER_URL"` attribute on the global flag).
#
# `exec` keeps signal handling sane — the binary becomes PID 1 in the
# container so Ctrl-C and `docker stop` propagate as expected.
exec /usr/local/bin/open4x "$@"
