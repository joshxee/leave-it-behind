//! Shared helpers for headless ECS tests.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::window::PrimaryWindow;
use leave_it_behind::player::Player;
use leave_it_behind::ship::CameraRig;
use leave_it_behind::{
    ActiveScenario, AppState, FIXED_HZ, GamePlugin, Scenario, TestDeterminismPlugin,
};

/// Headless app with the full `GamePlugin`, deterministic time (one fixed
/// tick per `update()`), and a fake 1280×720 primary window. Startup has not
/// run yet.
///
/// `InputPlugin` is left out on purpose: tests press and clear keys themselves.
pub fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<AccumulatedMouseScroll>()
        .add_plugins((GamePlugin, TestDeterminismPlugin));
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app
}

/// Like [`test_app`], starting from a scenario fixture. Runs `Startup` and
/// the Boot -> Playing transition (where the scenario applies) before returning.
pub fn test_app_with(scenario: Scenario) -> App {
    let mut app = test_app();
    app.insert_resource(ActiveScenario(scenario));
    boot(&mut app);
    app
}

/// Runs frames until the game is in `AppState::Playing`.
pub fn boot(app: &mut App) {
    for _ in 0..3 {
        frame(app);
        if state(app) == AppState::Playing {
            return;
        }
    }
    panic!("game never reached AppState::Playing");
}

pub fn state(app: &App) -> AppState {
    *app.world().resource::<State<AppState>>().get()
}

/// Runs one frame (one fixed tick), then clears just-pressed / just-released
/// like `InputPlugin` would, so held keys stay held but fire once.
pub fn frame(app: &mut App) {
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .clear();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    app.world_mut()
        .resource_mut::<AccumulatedMouseScroll>()
        .delta = Vec2::ZERO;
}

/// Runs `n` frames (= `n` fixed ticks under `TestDeterminismPlugin`).
pub fn run_frames(app: &mut App, n: usize) {
    for _ in 0..n {
        frame(app);
    }
}

/// Frames in `secs` seconds of game time.
pub fn secs(secs: f32) -> usize {
    (secs as f64 * FIXED_HZ).round() as usize
}

pub fn press(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(key);
}

pub fn release(app: &mut App, key: KeyCode) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(key);
}

/// Holds `key` for exactly one frame. Gameplay sees it on the next frame.
pub fn tap(app: &mut App, key: KeyCode) {
    press(app, key);
    frame(app);
    release(app, key);
}

pub fn mouse_down(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
}

pub fn mouse_up(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
}

/// One left click (down for one frame).
pub fn click(app: &mut App) {
    mouse_down(app);
    frame(app);
    mouse_up(app);
}

/// One frame of scrolling by `lines` wheel notches (negative = down).
pub fn scroll(app: &mut App, lines: f32) {
    *app.world_mut().resource_mut::<AccumulatedMouseScroll>() = AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, lines),
    };
    frame(app);
}

/// Puts the mouse cursor over world point `p` (the fake window is 1280×720,
/// one world unit per pixel, centered on the current room).
pub fn aim_at(app: &mut App, p: Vec2) {
    let mut rigs = app.world_mut().query::<&CameraRig>();
    let anchor = rigs.single(app.world()).unwrap().anchor;
    let offset = p - anchor;
    let cursor = Vec2::new(640.0 + offset.x, 360.0 - offset.y);
    let mut windows = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    windows
        .single_mut(app.world_mut())
        .unwrap()
        .set_cursor_position(Some(cursor));
}

pub fn player_pos(app: &mut App) -> Vec2 {
    let mut q = app.world_mut().query_filtered::<&Transform, With<Player>>();
    q.single(app.world()).unwrap().translation.truncate()
}

/// Moves the player (test setup only; gameplay never teleports).
pub fn put_player(app: &mut App, p: Vec2) {
    let mut q = app
        .world_mut()
        .query_filtered::<&mut Transform, With<Player>>();
    for mut t in q.iter_mut(app.world_mut()) {
        t.translation = p.extend(t.translation.z);
    }
}

/// Runs frames until `done` holds, up to `max` frames. Returns frames run.
pub fn run_until(app: &mut App, max: usize, mut done: impl FnMut(&mut App) -> bool) -> usize {
    for n in 0..max {
        if done(app) {
            return n;
        }
        frame(app);
    }
    panic!("condition not reached within {max} frames");
}

/// The WASD key that walks toward `dir` along its dominant axis.
pub fn key_toward(dir: Vec2) -> KeyCode {
    if dir.x.abs() >= dir.y.abs() {
        if dir.x > 0.0 {
            KeyCode::KeyD
        } else {
            KeyCode::KeyA
        }
    } else if dir.y > 0.0 {
        KeyCode::KeyW
    } else {
        KeyCode::KeyS
    }
}
