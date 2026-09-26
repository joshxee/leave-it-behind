//! Shared helpers for headless ECS tests.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use leave_it_behind::{ActiveScenario, AppState, GamePlugin, Scenario, TestDeterminismPlugin};

/// Headless app with the full `GamePlugin`, deterministic time (one fixed
/// tick per `update()`), and a fake primary window. Startup has not run yet.
///
/// `InputPlugin` is left out on purpose: tests press and clear keys themselves.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .add_plugins((GamePlugin, TestDeterminismPlugin));
    app.world_mut().spawn(Window::default());
    app
}

/// Like [`test_app`], starting from a scenario fixture. Runs `Startup` and
/// the Boot -> Playing transition before returning.
pub fn test_app_with(scenario: Scenario) -> App {
    let mut app = test_app();
    app.insert_resource(ActiveScenario(scenario));
    boot(&mut app);
    app
}

/// Runs frames until the game is in `AppState::Playing`.
pub fn boot(app: &mut App) {
    for _ in 0..3 {
        app.update();
        if *app.world().resource::<State<AppState>>().get() == AppState::Playing {
            return;
        }
    }
    panic!("game never reached AppState::Playing");
}

/// Holds `key` for exactly one frame.
pub fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
    app.update();
    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.release(key);
    input.clear();
}

/// Runs `n` frames (= `n` fixed ticks under `TestDeterminismPlugin`).
pub fn run_frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}
