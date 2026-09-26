# Testing

| Layer | Where | Runs in CI | Command |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` beside the code | yes | `cargo test --lib` |
| ECS integration | `tests/integration/` (one binary) | yes | `cargo test --test integration` |
| Native render smoke | `ci/smoke.ron`, `scripts/smoke-native.sh` | yes, `continue-on-error` | `scripts/smoke-native.sh` |
| Web e2e | `e2e/specs/` (Playwright) | yes | see below |
| Visual baseline | `e2e/specs/visual.spec.ts` (`@visual`) | skipped until a Linux baseline exists | see below |
| Release smoke | `e2e/specs/release-smoke.spec.ts` (`@release-smoke`) | on release builds | see below |

`cargo nextest run` works as a faster optional runner; plain `cargo test` must always work.

## Unit tests

Pure functions (`step_position`, `add_points`) are tested directly. For one
system, build a `World`, insert what it needs, and call
`world.run_system_once(system)` (see `src/scoring.rs`).

## ECS integration tests

**Never add a new top-level file under `tests/`.** Each would be linked
against Bevy as its own binary. Add `tests/integration/<feature>.rs` and a
`mod <feature>;` line in `tests/integration/main.rs`.

Helpers in `tests/integration/common.rs`:

- `test_app()`: `MinimalPlugins` + `StatesPlugin` + `ButtonInput` resources +
  a fake `Window` + `GamePlugin` + `TestDeterminismPlugin`. `InputPlugin` is
  deliberately absent, so tests control presses.
- `test_app_with(Scenario::X)`: same, starting from a scenario, booted into `Playing`.
- `boot`, `tap(app, key)`, `run_frames(app, n)`.

Patterns:

- Time is deterministic. `TimeUpdateStrategy::ManualDuration(1/60 s)` makes
  every `app.update()` run exactly one `FixedUpdate` tick. Never sleep.
- Input: `resource_mut::<ButtonInput<KeyCode>>().press(..)`, update, then
  `.release(..)` / `.clear()`.
- Messages: take `messages.get_cursor_current()` before acting, then
  `cursor.read(messages)` afterwards.
- States: set `NextState<AppState>`, `update()`, assert `State<AppState>`.
- Input is read in `Update` but applied in `FixedUpdate`, which runs *before*
  `Update` in a frame. So a press takes effect on the frame after it's read.
  Call `app.update()` once more after `tap`.

## Scenarios

`src/scenarios/mod.rs` holds fixtures: `fn(&mut World)` run after startup
spawning. The same scenario is used by:

- native tests: `test_app_with(Scenario::ScoreNine)`
- the e2e web build: `index.html?scenario=score_nine`
- natively with `--features e2e`: `SCENARIO=score_nine cargo run --features e2e`
- `node e2e/capture.mjs --scenario score_nine`

Current: `default`, `score_nine` (one point below `SCORE_THRESHOLD`).

Template for a new one:

```rust
// 1. variant
pub enum Scenario { /* ... */ BossFight }
// 2. name + ALL
Scenario::BossFight => "boss_fight",
// 3. apply
Scenario::BossFight => boss_fight(world),

fn boss_fight(world: &mut World) {
    world.insert_resource(Score(42));
    world.spawn((Boss, Transform::from_xyz(200.0, 0.0, 0.0)));
}
```

Then add a test in `tests/integration/scenarios.rs` and list it in the feature's `README.md`.

## Native render smoke test

`scripts/smoke-native.sh` builds with `--features smoke` (Bevy's
`bevy_ci_testing`) and runs the game with `CI_TESTING_CONFIG=ci/smoke.ron`.
It takes a named screenshot at frame 60, exits at frame 90, and moves the image
to `test-reports/smoke/smoke.png`. It fails if the log shows a panic or a
missing asset. On headless Linux it runs under `xvfb-run` with Mesa's software
renderers (llvmpipe/lavapipe; `LIBGL_ALWAYS_SOFTWARE=1`).

RON schema (bevy_dev_tools 0.19): `(setup: (fixed_frame_time: Some(f32)),
events: [(frame, Screenshot | NamedScreenshot("x") | ScreenshotAndExit | AppExit | Custom("x")), ...])`.

GPU emulation on hosted runners is not guaranteed, so the CI job is
`continue-on-error: true`. Treat a failure as a signal to run it locally, not as noise.

## Web e2e (Playwright)

```bash
E2E=1 scripts/build-web.sh                 # build with the test bridge
cd e2e && npm ci
npx playwright test --grep-invert @visual  # serve-web.sh is started automatically
node build-report.mjs                      # test-reports/e2e/summary.md
```

The bridge (`src/e2e_bridge.rs`, only with `--features e2e` on wasm):

- `window.__bevyReady` / console `BEVY_READY` after the first rendered frame in `Playing`.
- `window.__bevyState`: `{state, tick, score, frozen, ready, player: {x, y}, entities: {players}}`.
- `?freeze=1` pauses virtual time once ready. `window.__bevyStep(n)` advances
  exactly `n` fixed ticks, one per frame, then pauses again. Use the
  `step(page, n)` helper.
- Under `e2e` the canvas is fixed at 1280×720 (the Playwright viewport),
  the RNG seed is fixed, and the version text is hidden.

Input comes from real Playwright keyboard/mouse events. There is no JS-to-ECS input backdoor.

Every spec attaches at least one named screenshot, pass or fail.

CI runs with `retries: 1`. A test that passes only on retry is **flaky**.
`build-report.mjs` lists it under "Flaky" as a warning. Fix it; don't ignore it.

## Visual baselines

`visual.spec.ts` compares the canvas to
`e2e/specs/__screenshots__/visual.spec.ts/starter-scene.png`. Baselines
depend on the platform: Windows/macOS renders won't match Linux CI.
Generate or update them **only on Linux**:

- GitHub: run the **Update snapshots** workflow (`workflow_dispatch`), download
  the `snapshots` artifact, review, and commit.
- Locally: `docker run --rm -v "$PWD":/w -w /w/e2e mcr.microsoft.com/playwright:v1.56.1-noble npx playwright test --grep @visual --update-snapshots`
  (after `E2E=1 scripts/build-web.sh`).

Only a human or the primary model approves baseline changes. Until the first
Linux baseline is committed, CI skips `@visual` with `--grep-invert @visual`.

## Release smoke

`@release-smoke` needs no bridge. It loads the page, waits 20 s, and fails on
console errors, the crash overlay, or a single-color canvas. Run it against a player build:

```bash
scripts/build-web.sh --release
cd e2e && npx playwright test --grep @release-smoke
```
