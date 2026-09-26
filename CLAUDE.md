# Leave It Behind: notes for AI agents

Bevy **0.19.1** game (see `Cargo.toml`). Bevy's API changes every minor
version (0.17 renamed events to messages: `add_message`, `MessageReader`,
`MessageWriter`, `Messages<T>`). Check APIs against the pinned version's
source in `~/.cargo/registry/src/*/bevy_*-0.19.1/` or its docs, never memory.

## Layout

- `src/main.rs`: only `DefaultPlugins` + `GamePlugin`. Keep it that way.
- `src/lib.rs`: `GamePlugin`, made of one plugin per feature module
  (`player`, `scoring`, `ui`, `audio`, `state`, `rng`, `scenarios`, `version`).
- `src/state.rs`: `AppState` (Boot → Playing) and `GameSet`
  (Input → Simulate → Resolve → Present), chained in `Startup`, `Update`, and `FixedUpdate`.
- `src/determinism.rs`: `TestDeterminismPlugin` (fixed seed, one fixed tick per frame).
- `src/e2e_bridge.rs`: browser test bridge, compiled only for wasm32 with `--features e2e`.
- `src/scenarios/`: scenario fixtures shared by native tests, the web build and `e2e/capture.mjs`.
- `tests/integration/`: the single integration-test binary (see Testing).
- `scripts/`: `build-web.sh` (the only way to build the web version), `serve-web.sh`, `smoke-native.sh`.
- `e2e/`: Playwright specs, `build-report.mjs`, `capture.mjs`.

## Rules

- **Gameplay runs in `FixedUpdate`** and reads input through `PlayerIntent`
  (written in `Update`), never `just_pressed` inside `FixedUpdate`.
- **Randomness only via `ResMut<GameRng>`.** Never call `rand::rng()`,
  `thread_rng()` or `getrandom` from gameplay code. Tests and e2e fix the seed.
- **Never add a top-level file under `tests/`.** Each one links Bevy as a
  separate binary. Add a module to `tests/integration/main.rs` instead.
- **Every new game situation gets a scenario** in `src/scenarios/`, and that
  scenario is listed in the feature's `README.md`. Use scenarios in tests and
  in `capture.mjs --scenario` rather than playing through to a state.
- **Size budget:** `ci/budgets.env` `WASM_BUDGET_KB` is a reviewed limit.
  Never raise it to make a check pass. Find what grew, or trim Bevy features
  (`default-features = false` plus an explicit list) and record the dropped
  features below.
- **Screenshot baselines** (`e2e/specs/__screenshots__`) are generated on
  Linux only and approved by a human or the primary model, never by a subagent.
- The `dev` feature (dynamic linking) is for local native builds only. Never
  enable it in CI, release, or wasm builds.
- The test bridge must never ship: `build-web.sh` fails a non-e2e build that
  contains `__bevyReady`, `__bevyState`, `__bevyStep`, `BEVY_READY`, or `scenario=`.
- Hide non-deterministic UI (FPS, timestamps, version text) under `e2e`.

## Commands

```bash
cargo test                                   # unit + integration (headless)
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features e2e -- -D warnings
cargo fmt --check
scripts/smoke-native.sh                      # native render smoke → test-reports/smoke/
E2E=1 scripts/build-web.sh                   # web build with test bridge
(cd e2e && npm ci && npx playwright test --grep-invert @visual)
node e2e/build-report.mjs                    # → test-reports/e2e/summary.md
node e2e/capture.mjs --scenario score_nine --ticks 30 --keys Space
scripts/build-web.sh --release               # player build, budget + leak check
cargo run --features dev                     # fast local iteration
```

## Bevy features

Default features (2d, 3d, ui, audio) plus `wav` (for `Clank.wav`) and, on
wasm, `web`. Nothing trimmed yet. If the size budget is exceeded, 3d
(pbr, gltf) and picking are the first candidates to drop; list any dropped
feature here.
