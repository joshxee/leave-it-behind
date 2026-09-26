#!/usr/bin/env bash
# Serves wasm/ on a fixed port (default 4173). Build first with build-web.sh.
set -euo pipefail
cd "$(dirname "$0")/../wasm"
PORT="${PORT:-4173}"

if command -v npx >/dev/null 2>&1; then
  exec npx --yes http-server@14 . -p "$PORT" -a 127.0.0.1 -c-1 --silent
elif command -v python3 >/dev/null 2>&1; then
  exec python3 -m http.server "$PORT" --bind 127.0.0.1
else
  exec python -m http.server "$PORT" --bind 127.0.0.1
fi
