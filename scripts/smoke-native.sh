#!/usr/bin/env bash
# Native headless render smoke test. Runs the game with the `smoke` feature
# (bevy_ci_testing), which follows ci/smoke.ron: the title screen at frame 40,
# level one at frame 110, exit at frame 130. Screenshots land in
# test-reports/smoke/ (title.png, smoke.png).
#
# On headless Linux, wraps the run in xvfb-run with Mesa software rendering.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT_DIR="test-reports/smoke"
mkdir -p "$OUT_DIR"
rm -f screenshot-smoke.png screenshot-title.png

cargo build --features smoke

export CI_TESTING_CONFIG="ci/smoke.ron"
# Assets resolve relative to the executable unless told otherwise.
export BEVY_ASSET_ROOT="$PWD"
BIN="target/debug/leave-it-behind"
[ -f "$BIN.exe" ] && BIN="$BIN.exe"

run() {
  if [ "$(uname -s)" = "Linux" ] && [ -z "${DISPLAY:-}${WAYLAND_DISPLAY:-}" ]; then
    export LIBGL_ALWAYS_SOFTWARE=1 WGPU_BACKEND="${WGPU_BACKEND:-vulkan}"
    xvfb-run -a -s "-screen 0 1280x720x24" "$@"
  else
    "$@"
  fi
}

LOG="$OUT_DIR/smoke.log"
run timeout 300 "$BIN" 2>&1 | tee "$LOG"

if grep -E "Path not found|panicked" "$LOG" >/dev/null; then
  echo "smoke: errors in log ($LOG)" >&2
  exit 1
fi

for shot in title smoke; do
  if [ ! -f "screenshot-$shot.png" ]; then
    echo "smoke: no $shot screenshot produced" >&2
    exit 1
  fi
  mv "screenshot-$shot.png" "$OUT_DIR/$shot.png"
done
echo "smoke: ok -> $OUT_DIR/title.png, $OUT_DIR/smoke.png"
