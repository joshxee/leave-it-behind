#!/usr/bin/env bash
# Runs every test layer in order and never stops early. Writes
# test-reports/<UTC timestamp>/ (logs, screenshots, summary.json, REPORT.md)
# and copies it to test-reports/latest/ (a copy: symlinks are unreliable on
# Windows). Exits non-zero if any step failed.
#
#   scripts/test-all.sh [--scope all|rust|native|web]
#     rust:   fmt, clippy, cargo test
#     native: rust + native smoke
#     web:    web build + Playwright
#     all:    everything (default)
set -uo pipefail
cd "$(dirname "$0")/.."

SCOPE="all"
while [ $# -gt 0 ]; do
  case "$1" in
    --scope) SCOPE="${2:-all}"; shift 2 ;;
    --scope=*) SCOPE="${1#--scope=}"; shift ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
case "$SCOPE" in all|rust|native|web) ;; *) echo "bad --scope: $SCOPE" >&2; exit 2 ;; esac

TS=$(date -u +%Y%m%dT%H%M%SZ)
DIR="test-reports/$TS"
mkdir -p "$DIR/logs" "$DIR/screenshots"
STEPS_JSON=""
FAILED=0

now() { date +%s; }

# record <step> <status> <exit> <seconds> <log>
record() {
  [ -n "$STEPS_JSON" ] && STEPS_JSON="$STEPS_JSON,"
  STEPS_JSON="$STEPS_JSON
    {\"step\": \"$1\", \"status\": \"$2\", \"exit_code\": $3, \"duration_s\": $4, \"log\": \"$5\"}"
  [ "$2" = "fail" ] && FAILED=1
  printf '%-10s %-5s (%ss) %s\n' "$1" "$2" "$4" "$5"
}

# run <step> <command...>: runs the command, logging to logs/<step>.log.
run() {
  local step="$1"; shift
  local log="logs/$step.log" start code
  start=$(now)
  echo "== $step: $*" >&2
  "$@" > "$DIR/$log" 2>&1
  code=$?
  tail -n 5 "$DIR/$log" >&2
  record "$step" "$([ $code -eq 0 ] && echo pass || echo fail)" "$code" "$(( $(now) - start ))" "$log"
  return $code
}

skip() { record "$1" skip 0 0 ""; echo "== $1: skipped ($2)" >&2; }

fmt_check() {
  cargo fmt --all -- --check || return 1
  local lines; lines=$(wc -l < CLAUDE.md)
  if [ "$lines" -gt 150 ]; then
    echo "CLAUDE.md is $lines lines (limit 150). Move detail to TESTING.md or src/<feature>/README.md."
    return 1
  fi
}

e2e_run() {
  local grep_args=()
  # @visual needs a committed Linux baseline (TESTING.md).
  [ -d e2e/specs/__screenshots__ ] || grep_args=(--grep-invert @visual)
  (cd e2e && { [ -d node_modules ] || npm ci; } && npx playwright test "${grep_args[@]}")
}

in_scope() { case "$SCOPE" in all) return 0 ;; *) [[ " $* " == *" $SCOPE "* ]] ;; esac; }

if in_scope rust native; then
  run fmt fmt_check
  # All features except `dev` (dynamic linking would rebuild Bevy as a dylib).
  run clippy cargo clippy --all-targets --features e2e,smoke -- -D warnings
  run test cargo test --workspace --no-fail-fast
fi

if in_scope native; then
  rm -f test-reports/smoke/smoke.png
  run smoke scripts/smoke-native.sh
  [ -f test-reports/smoke/smoke.png ] && cp test-reports/smoke/smoke.png "$DIR/screenshots/smoke-native.png"
fi

if in_scope web; then
  rm -rf e2e/test-results e2e/results.json
  if run web-build env E2E=1 scripts/build-web.sh; then
    run e2e e2e_run
  else
    skip e2e "web build failed"
  fi
  [ -f e2e/results.json ] && cp e2e/results.json "$DIR/playwright-results.json"
  [ -d e2e/test-results ] && mkdir -p "$DIR/screenshots/e2e" && cp -R e2e/test-results/. "$DIR/screenshots/e2e/"
fi

cat > "$DIR/summary.json" <<JSON
{
  "timestamp": "$TS",
  "scope": "$SCOPE",
  "status": "$([ $FAILED -eq 0 ] && echo pass || echo fail)",
  "steps": [$STEPS_JSON
  ]
}
JSON

node scripts/build-report.mjs "$DIR" || FAILED=1

rm -rf test-reports/latest
cp -R "$DIR" test-reports/latest

echo "report: $DIR/REPORT.md (copy in test-reports/latest/)"
tail -n 1 "$DIR/REPORT.md"
exit $FAILED
