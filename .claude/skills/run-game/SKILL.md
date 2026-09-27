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

Opens a 1280×720 window on the title screen: any key opens the main menu,
Enter on PLAY starts level one. WASD moves, the mouse aims, left click uses the
held tool, 1 / 2 or the wheel switch tools, E interacts, Esc or P pauses, R
flies again from the end screen. Menus: arrows or WASD, Enter, Esc, or the
mouse. Settings and progress save to the OS app-data folder (`src/save/README.md`).
Needs a display. For a headless check, use `scripts/smoke-native.sh`
(screenshot in `test-reports/smoke/smoke.png`).
Start from a scenario natively with `SCENARIO=bolts cargo run --features dev,e2e`.

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
   Query params: `?scenario=<name>` (see `src/scenarios/`; any scenario skips the
   title screen), `?freeze=1`. Without a scenario the game starts on the title.
   A flight started from there with fresh saves (every capture) is a first
   flight: the countdown holds until E at the diagnostic console finishes a
   scan (`src/coach/README.md`). Scenarios skip that, except `first_flight`.

## Drive it

```bash
node e2e/tools/capture.mjs --out test-reports/verify-<ts> [--scenario breach] [--keys "Move:480:100,Click:3000"] [--wait-state 's.faults.length === 0'] [--freeze --ticks 30]
```

- Waits for `__bevyReady` (the game has rendered its first screen: the title,
  or a flight with `--scenario`), then runs each
  step and saves `NN-<step>.png` after it: `KEY` presses a key (`KeyD`, `e`,
  `Digit2`), `KEY:ms` holds it, `Move:X:Y` moves the mouse to canvas pixel
  (X, Y), `Click` clicks, `Click:ms` holds the left button, `Wheel:DY` scrolls.
  Move the mouse before clicking: a click also re-aims the engineer.
  With `--freeze`, a held key or button lasts `--ticks` ticks instead of `ms`.
- The canvas is 1280×720, one world unit per pixel, centered on the current
  room: world (x, y) is canvas (640 + x − camera.x, 360 − (y − camera.y)).
- `--wait-state` takes a JS predicate over `s` (`window.__bevyState`; the shape
  is `BevyState` in `e2e/specs/helpers.ts`: `state, paused, menu, settings,
  progress, lastRun, tick, room, camera, player, tool, tape, faults, looseBolts,
  nav, diag, alarm, stats, ...`). To reach the main menu from the title, press
  any key (`--keys "Enter"`); `s.menu.screen` names the open screen.
- At about 15 fps under software rendering, each frame is one game tick: keep
  `--ticks` per step under ~600 (the tool waits 60 s per step).
- Writes `states.json`, `console.json`, and `result.json`. Exits 1 on console
  errors, page errors, the crash overlay, or no `__bevyReady`.
- It starts the server itself if nothing is listening on 4173.

## Teardown

```bash
pkill -f "[h]ttp-server"; pkill -f "[h]ttp.server 4173"
```

The `[h]` keeps `pkill` from matching (and killing) the shell running it.
Or stop the background task that runs `scripts/serve-web.sh`.
