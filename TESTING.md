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
        assert_eq!(cycle_tool(Tool::Wrench, 1), Tool::Tape);
    }

    #[test]
    fn one_system() {
        let mut world = World::new();
        world.init_resource::<Alarm>();
        world.spawn(Fault::new(Site::Helm, 50.0));
        world.run_system_once(some_system).unwrap();
        assert_eq!(world.resource::<Alarm>().active, 1);
    }
}
```

## ECS integration tests

**Never add a new top-level file under `tests/`.** Each would be linked
against Bevy as its own binary (`autotests = false`; one `[[test]]` in
`Cargo.toml`). Add `tests/integration/<feature>.rs` and `mod <feature>;` in
`tests/integration/main.rs`.

Helpers (`tests/integration/common.rs`):

- `test_app()`: `MinimalPlugins` + `StatesPlugin` + `ButtonInput` / scroll resources + a
  fake 1280×720 `PrimaryWindow` + `GamePlugin` + `TestDeterminismPlugin`. `InputPlugin` is
  deliberately absent so tests control presses. Startup has not run yet.
- `test_app_with(Scenario::X)`: starts from a scenario, already booted into `Playing`.
- `boot`, `frame` (one tick, then clears just-pressed like `InputPlugin`), `run_frames`,
  `run_until(app, max, pred)`, `secs(s)` (frames in `s` seconds), `state`.
- Input: `press` / `release` / `tap` keys, `mouse_down` / `mouse_up` / `click`, `scroll`,
  `aim_at(world_point)` (puts the cursor over a world point in the current room).
- Setup only: `put_player`, `player_pos`.

Rules:

- Every `app.update()` is exactly one `FixedUpdate` tick (`TimeUpdateStrategy::ManualDuration`). Never sleep.
- Input read in `Update` is applied by `FixedUpdate` next frame, so after `tap`, call `frame` once more.
- Use `tap` for E, R and number keys (they act on just-pressed); hold WASD with `press`.
- Aiming is relative to the current room's camera: after `put_player` into another room, run a frame before `aim_at`.
- Messages: take a cursor with `get_cursor_current()` before acting, then `cursor.read(messages)`.
- States: set `NextState<AppState>`, `frame`, then assert `State<AppState>`.
- Level-design checks may set `Fault::repair` directly (see `tests/integration/level.rs`); control tests must use real input.

Template (`tests/integration/<feature>.rs`):

```rust
use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::level::RunStats;

use crate::common::{mouse_down, run_until, secs, test_app_with};

#[test]
fn holding_tape_on_a_breach_seals_it() {
    let mut app = test_app_with(Scenario::Breach);
    mouse_down(&mut app);
    run_until(&mut app, secs(5.0), |app| app.world().resource::<RunStats>().fixed == 1);
}
```

## Scenarios

Fixtures in `src/scenarios/mod.rs`: `fn(&mut World)` run on top of every fresh
run (`OnEnter(AppState::Playing)`, after the player, tools and fault plan
exist). The same scenario is used by native tests (`test_app_with`), the web
build (`?scenario=<name>`), native e2e runs (`SCENARIO=<name> cargo run --features e2e`),
and `node e2e/tools/capture.mjs --scenario <name>`. Current ones: `default`, `quiet`,
`bolts`, `breach`, `drift`, `diagnostics`, `scramble`, `landing`, `breach_critical`,
`tape_low` (see `src/scenarios/README.md`).

Template:

```rust
pub enum Scenario { /* ... */ TwoBreaches }               // 1. variant (+ add to ALL)
Scenario::TwoBreaches => "two_breaches",                  // 2. name()
Scenario::TwoBreaches => two_breaches(world),             // 3. apply()

fn two_breaches(world: &mut World) {
    quiet(world);                                         // no scheduled faults
    start(world, Site::AirlockPortAft, 50.0);
    start(world, Site::HullPortFore, 45.0);
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
`window.__bevyState`: state, tick, room, camera, player (position, facing,
locked), focus, tool, tape, faults (kind, site, room, remaining, repair,
position), loose bolts, nav marker, diagnostics view, alarm, run stats. The
full shape is `BevyState` in `e2e/specs/helpers.ts`. With `?freeze=1`,
`window.__bevyStep(n)` advances exactly `n` fixed ticks (helper: `step(page, n)`).
Under `e2e`, the canvas is fixed at 1280×720, the seed is fixed, and each frame
advances game time by exactly one tick (slow software rendering slows the game,
it never skips ticks). Helpers: `waitForState(page, pred)`, `toScreen` /
`aimAt(page, worldPoint)` (the canvas is at the page origin, one world unit per
pixel, centered on the room), `averageColor` (tint checks without baselines).

Rules: use real Playwright keyboard/mouse input (no JS-to-ECS backdoor). Every
test attaches at least one screenshot, pass or fail, named
`"<label>: <what it should show>"`. The report's screenshot index uses that text verbatim.

Template (`e2e/specs/<feature>.spec.ts`):

```ts
import { expect, test } from '@playwright/test';
import { aimAt, attachShot, collectErrors, openGame, waitForState } from './helpers';

test('tape seals the breach', async ({ page }, testInfo) => {
  const errors = collectErrors(page);
  try {
    await openGame(page, { scenario: 'breach' });   // also focuses the canvas
    const s = await waitForState(page, (t) => t.faults.length === 1);
    await aimAt(page, s.faults[0]);
    await page.mouse.down();
    await waitForState(page, (t) => t.faults.length === 0, 40_000);
    await page.mouse.up();
    expect(errors).toEqual([]);
  } finally {
    await attachShot(page, testInfo, 'sealed: tape strips over the spot, no hole left');
  }
});
```

Focus the canvas (`openGame` does) rather than clicking it: a click also uses the held tool.

CI runs with `retries: 1`. A test that passes only on retry is **flaky**, and
`REPORT.md` lists it under "Flaky" as a warning. Fix it; don't ignore it.

## Visual baselines

`visual.spec.ts` compares the frozen canvas (`quiet` scenario, 30 ticks) to
`e2e/specs/__screenshots__/visual.spec.ts/level-start.png`. Baselines depend
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
