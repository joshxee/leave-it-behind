# Testing

Run everything: `scripts/test-all.sh` (or `/test-report`). It writes
`test-reports/<ts>/REPORT.md` (copied to `test-reports/latest/`).

| Layer | Use it for | Where | Command |
|---|---|---|---|
| Unit | pure functions, one system | `#[cfg(test)] mod tests` beside the code | `cargo test --lib` |
| ECS integration | plugins together: input → systems → state | `tests/integration/<feature>.rs` | `cargo test --test integration` |
| Native smoke | "does it render natively" | `ci/smoke.ron`, `scripts/smoke-native.sh` | `scripts/smoke-native.sh` |
| Web e2e | visible/interactive behaviour in the real browser build | `e2e/specs/*.spec.ts` | `cd e2e && npx playwright test` |
| Visual | pixel regressions | `e2e/specs/visual.spec.ts` (`@visual`) | Linux only (below) |
| Release smoke | a player build loads and runs | `e2e/specs/release-smoke.spec.ts` (`@release-smoke`) | below |

Choose the lowest layer that can catch the bug. Logic belongs in unit tests.
Anything that crosses plugins goes in an integration test. Use e2e only for
what the player sees or does in the browser. `cargo nextest run` is an
optional faster runner; plain `cargo test` must always work.

## Unit tests

Template (beside the code, e.g. `src/<feature>/mod.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn pure_rule() {
        assert_eq!(add_points(9, 1), (10, true));
    }

    #[test]
    fn one_system() {
        let mut world = World::new();
        world.init_resource::<Score>();
        world.init_resource::<Messages<ScoreChanged>>();
        world.init_resource::<Messages<ThresholdReached>>();
        world.insert_resource(PlayerIntent { score_presses: 2, ..default() });
        world.run_system_once(apply_score_presses).unwrap();
        assert_eq!(world.resource::<Score>().0, 2);
    }
}
```

## ECS integration tests

**Never add a new top-level file under `tests/`.** Each would be linked
against Bevy as its own binary (`autotests = false`; one `[[test]]` in
`Cargo.toml`). Add `tests/integration/<feature>.rs` and `mod <feature>;` in
`tests/integration/main.rs`.

Helpers (`tests/integration/common.rs`):

- `test_app()`: `MinimalPlugins` + `StatesPlugin` + `ButtonInput` resources + a
  fake `Window` + `GamePlugin` + `TestDeterminismPlugin`. `InputPlugin` is
  deliberately absent so tests control presses. Startup has not run yet.
- `test_app_with(Scenario::X)`: starts from a scenario, already booted into `Playing`.
- `boot(&mut app)`, `tap(&mut app, key)`, `run_frames(&mut app, n)`.

Rules:

- Every `app.update()` is exactly one `FixedUpdate` tick (`TimeUpdateStrategy::ManualDuration`). Never sleep.
- Input read in `Update` is applied by `FixedUpdate` next frame, so after `tap`, call `app.update()` once more.
- Messages: take a cursor with `get_cursor_current()` before acting, then `cursor.read(messages)`.
- States: set `NextState<AppState>`, `update()`, then assert `State<AppState>`.

Template (`tests/integration/<feature>.rs`):

```rust
use bevy::prelude::*;
use leave_it_behind::scoring::{Score, ScoreChanged};
use leave_it_behind::Scenario;

use crate::common::{tap, test_app_with};

#[test]
fn space_scores_from_scenario() {
    let mut app = test_app_with(Scenario::ScoreNine);
    let mut cursor = app.world().resource::<Messages<ScoreChanged>>().get_cursor_current();

    tap(&mut app, KeyCode::Space);
    app.update();

    assert_eq!(*app.world().resource::<Score>(), Score(10));
    let messages = app.world().resource::<Messages<ScoreChanged>>();
    assert_eq!(cursor.read(messages).count(), 1);
}
```

## Scenarios

Fixtures in `src/scenarios/mod.rs`: `fn(&mut World)` run after startup
spawning. The same scenario is used by native tests (`test_app_with`), the web
build (`?scenario=<name>`), native e2e runs (`SCENARIO=<name> cargo run --features e2e`),
and `node e2e/tools/capture.mjs --scenario <name>`. Current ones: `default`, `score_nine`.

Template:

```rust
pub enum Scenario { /* ... */ BossFight }                // 1. variant (+ add to ALL)
Scenario::BossFight => "boss_fight",                     // 2. name()
Scenario::BossFight => boss_fight(world),                // 3. apply()

fn boss_fight(world: &mut World) {
    world.insert_resource(Score(42));
    world.spawn((Boss, Transform::from_xyz(200.0, 0.0, 0.0)));
}
```

Then add a test in `tests/integration/scenarios.rs` and list it in the feature's `README.md`.

## Native render smoke

`scripts/smoke-native.sh` builds with `--features smoke` (Bevy's
`bevy_ci_testing`) and runs `CI_TESTING_CONFIG=ci/smoke.ron`: a screenshot at
frame 60, exit at frame 90, saved to `test-reports/smoke/smoke.png`. It fails on
a panic or a missing asset. Headless Linux runs under `xvfb-run` with Mesa
software rendering. The CI job is `continue-on-error: true` because GPU
emulation on hosted runners is not guaranteed. A red run is a prompt to run it locally.

To add a check, edit `ci/smoke.ron`. Schema for bevy_dev_tools 0.19:

```ron
(
    setup: (fixed_frame_time: Some(0.016666668)),
    events: [(60, NamedScreenshot("smoke")), (90, AppExit)],
)
```

## Web e2e (Playwright)

```bash
E2E=1 scripts/build-web.sh
cd e2e && npm ci && npx playwright test --grep-invert @visual   # serve-web.sh starts automatically
```

Test bridge (`src/e2e_bridge.rs`, wasm + `--features e2e` only):
`window.__bevyReady` after the first rendered frame in `Playing`, and
`window.__bevyState` = `{state, tick, score, frozen, ready, player: {x, y}, entities: {players}}`.
With `?freeze=1`, `window.__bevyStep(n)` advances exactly `n` fixed ticks
(helper: `step(page, n)`). Under `e2e`, the canvas is fixed at 1280×720 and the seed is fixed.

Rules: use real Playwright keyboard/mouse input (no JS-to-ECS backdoor). Every
test attaches at least one screenshot, pass or fail, named
`"<label>: <what it should show>"`. The report's screenshot index uses that text verbatim.

Template (`e2e/specs/<feature>.spec.ts`):

```ts
import { expect, test } from '@playwright/test';
import { attachShot, collectErrors, gameState, openGame } from './helpers';

test('space from score_nine reaches the threshold', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'score_nine' });
    await page.locator('#bevy-canvas').click();
    await page.keyboard.press('Space');
    await page.waitForFunction(() => window.__bevyState!.score === 10);
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'after-space: score text reads "SCORE: 10"', true);
  }
});
```

CI runs with `retries: 1`. A test that passes only on retry is **flaky**, and
`REPORT.md` lists it under "Flaky" as a warning. Fix it; don't ignore it.

## Visual baselines

`visual.spec.ts` compares the frozen canvas to
`e2e/specs/__screenshots__/visual.spec.ts/starter-scene.png`. Baselines depend
on the platform, so generate them **only on Linux**:

- GitHub: run the **Update snapshots** workflow, download the `snapshots` artifact, review it, and commit.
- Locally: `docker run --rm -v "$PWD":/w -w /w/e2e mcr.microsoft.com/playwright:v1.56.1-noble npx playwright test --grep @visual --update-snapshots`

Only a human or the primary model approves baseline changes. Until the first
baseline is committed, CI and `test-all.sh` skip `@visual`.

## Release smoke

Needs no bridge. It loads the page, waits 20 s, and fails on console errors,
the crash overlay, or a single-color canvas. The release workflow runs it
against the exact `html5` artifact. Locally:

```bash
scripts/build-web.sh --release && (cd e2e && npx playwright test --grep @release-smoke)
```

## Reports

`scripts/test-all.sh` writes `test-reports/<ts>/` with `summary.json`,
`logs/`, `screenshots/`, and `REPORT.md`, built deterministically by
`scripts/build-report.mjs`. The report has a summary table, failures (name,
file:line, ≤30-line excerpt, artifact paths), flaky tests, a screenshot index,
an empty `## Triage` section (filled only by the test-runner agent on failure),
and `STATUS: PASS|FAIL`.
