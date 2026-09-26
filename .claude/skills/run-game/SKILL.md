---
name: run-game
description: How to build and launch this Bevy game natively and in the browser (e2e web build with the test bridge), drive it with keyboard input, capture screenshots and game state, and tear it down. Followed by /run, /verify, and any agent that needs the running game.
allowed-tools: Bash(cargo run --features dev) Bash(E2E=1 scripts/build-web.sh) Bash(scripts/serve-web.sh) Bash(node e2e/tools/capture.mjs *) Bash(scripts/smoke-native.sh) Bash(curl -sf http://localhost:4173/index.html) Bash(pkill -f "[h]ttp-server") Bash(pkill -f "[h]ttp.server 4173")
---

# Run the game

## Native

```bash
cargo run --features dev
```

Opens a 1280×720 window. Arrow keys move, Space scores. Needs a display. For a
headless check, use `scripts/smoke-native.sh` (screenshot in `test-reports/smoke/smoke.png`).
Start from a scenario natively with `SCENARIO=score_nine cargo run --features dev,e2e`.

## Web (with the test bridge)

1. Build. The first build takes about 10 minutes; later ones take 1–2:
   ```bash
   E2E=1 scripts/build-web.sh
   ```
2. Serve in the background on port 4173:
   ```bash
   scripts/serve-web.sh   # run_in_background
   ```
3. Wait until `curl -sf http://localhost:4173/index.html` succeeds. The URL is `http://localhost:4173`.
   Query params: `?scenario=<name>` (see `src/scenarios/`), `?freeze=1`.

## Drive it

```bash
node e2e/tools/capture.mjs --out test-reports/verify-<ts> [--scenario score_nine] [--keys "Space,ArrowRight:500"] [--wait-state 's.score === 10'] [--freeze --ticks 30]
```

- Waits for `__bevyReady` (the game has rendered in `Playing`), then presses
  each key (`KEY:ms` holds it) and saves `NN-<step>.png` after each step.
- `--wait-state` takes a JS predicate over `s` (`window.__bevyState`:
  `state, tick, score, frozen, player {x, y}, entities {players}`).
- Writes `states.json`, `console.json`, and `result.json`. Exits 1 on console
  errors, page errors, the crash overlay, or no `__bevyReady`.
- It starts the server itself if nothing is listening on 4173.

## Teardown

```bash
pkill -f "[h]ttp-server"; pkill -f "[h]ttp.server 4173"
```

The `[h]` keeps `pkill` from matching (and killing) the shell running it.
Or stop the background task that runs `scripts/serve-web.sh`.
