#!/usr/bin/env bash
# Single source of truth for the web build. Used by CI, the release workflow,
# the e2e tests, and the verify skill.
#
#   scripts/build-web.sh [--release]
#     E2E=1               build with the `e2e` feature (test bridge + scenarios)
#     SKIP_BUDGET=1       skip the size budget check (never in CI)
#
# Both modes build the wasm-release profile; --release also runs wasm-opt.
set -euo pipefail
cd "$(dirname "$0")/.."

RELEASE=0
for arg in "$@"; do
  case "$arg" in
    --release) RELEASE=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

PROFILE="wasm-release"
BIN="leave-it-behind"
OUT="wasm"
FEATURES=()
if [ "${E2E:-}" = "1" ]; then FEATURES=(--features e2e); fi

cargo build --profile "$PROFILE" --target wasm32-unknown-unknown "${FEATURES[@]}"

wasm-bindgen --no-typescript --out-name bevy_game --out-dir "$OUT" --target web \
  "target/wasm32-unknown-unknown/$PROFILE/$BIN.wasm"

# Real copy (symlinks don't work on Windows).
rm -rf "$OUT/assets"
cp -R assets "$OUT/assets"

# Version shown by the crash overlay in index.html.
VERSION="${GAME_VERSION:-$(git describe --tags --exact-match 2>/dev/null || git rev-parse --short HEAD 2>/dev/null || echo unknown)}"
printf 'window.__gameVersion = "%s";\n' "$VERSION" > "$OUT/version.js"

WASM="$OUT/bevy_game_bg.wasm"
if [ "$RELEASE" = "1" ]; then
  if command -v wasm-opt >/dev/null 2>&1; then
    before=$(wc -c < "$WASM")
    wasm-opt -Oz --all-features -o "$WASM.opt" "$WASM"
    after=$(wc -c < "$WASM.opt")
    if [ "$after" -ge "$before" ]; then
      echo "wasm-opt did not shrink the output ($before -> $after bytes)" >&2
      exit 1
    fi
    mv "$WASM.opt" "$WASM"
    echo "wasm-opt: $before -> $after bytes"
  else
    echo "wasm-opt not installed; skipping (install binaryen for smaller builds)"
  fi
fi

# Size budget (gzipped).
gz_kb=$(( $(gzip -9 -c "$WASM" | wc -c) / 1024 ))
echo "wasm size: ${gz_kb} KiB gzipped"
if [ -z "${SKIP_BUDGET:-}" ]; then
  # shellcheck source=../ci/budgets.env
  . ci/budgets.env
  if [ "$gz_kb" -gt "$WASM_BUDGET_KB" ]; then
    echo "wasm size ${gz_kb} KiB exceeds budget ${WASM_BUDGET_KB} KiB (ci/budgets.env)" >&2
    exit 1
  fi
fi

# The test bridge and scenario loader must never reach players.
if [ "${E2E:-}" != "1" ]; then
  for needle in __bevyReady __bevyState __bevyStep BEVY_READY 'scenario='; do
    if grep -a -q -F "$needle" "$WASM" "$OUT/bevy_game.js"; then
      echo "leak check: '$needle' found in a non-e2e web build" >&2
      exit 1
    fi
  done
  echo "leak check: ok"
fi
