#!/usr/bin/env bash
# Native headless render smoke test. Runs the game with the `smoke` feature
# (bevy_ci_testing), which follows ci/smoke.ron: screenshot at frame 60,
# exit at frame 90. The screenshot lands in test-reports/smoke/.
#
# On headless Linux, wraps the run in xvfb-run with Mesa software rendering.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT_DIR="test-reports/smoke"
mkdir -p "$OUT_DIR"
rm -f screenshot-smoke.png

cargo build --features smoke

export CI_TESTING_CONFIG="ci/smoke.ron"
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

run timeout 300 "$BIN"

if [ ! -f screenshot-smoke.png ]; then
  echo "smoke: no screenshot produced" >&2
  exit 1
fi
mv screenshot-smoke.png "$OUT_DIR/smoke.png"
echo "smoke: ok -> $OUT_DIR/smoke.png"
