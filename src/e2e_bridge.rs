//! Browser test bridge. Compiled only for `wasm32` with the `e2e` feature;
//! `scripts/build-web.sh` fails a non-e2e build that contains any of this.
//!
//! - `window.__bevyReady = true` (and console `BEVY_READY`) after the first
//!   rendered frame in `AppState::Playing`.
//! - `window.__bevyState`: JSON-compatible snapshot, refreshed every
//!   [`SNAPSHOT_EVERY`] frames (every frame while frozen).
//! - `?freeze=1`: pause `Time<Virtual>` once ready.
//! - `window.__bevyStep(n)`: advance exactly `n` fixed ticks (one per frame,
//!   since each frame advances virtual time by one fixed step), then pause.

use std::cell::Cell;

use bevy::prelude::*;
use bevy::time::TimeSystems;
use wasm_bindgen::prelude::*;

use crate::determinism::FixedTick;
use crate::player::Player;
use crate::scoring::Score;
use crate::AppState;

const SNAPSHOT_EVERY: u32 = 5;

thread_local! {
    static PENDING_STEPS: Cell<u32> = const { Cell::new(0) };
}

#[derive(Resource, Default)]
struct Bridge {
    frozen_requested: bool,
    ready: bool,
    playing_frames: u32,
}

pub struct E2eBridgePlugin;

impl Plugin for E2eBridgePlugin {
    fn build(&self, app: &mut App) {
        let frozen_requested = query_param("freeze").as_deref() == Some("1");
        install_step_fn();
        app.insert_resource(Bridge {
            frozen_requested,
            ..default()
        })
        .add_systems(First, drive_virtual_time.before(TimeSystems))
        .add_systems(Last, (mark_ready, publish_state).chain());
    }
}

fn query_param(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search).ok()?.get(key)
}

fn set_global(key: &str, value: &JsValue) {
    if let Some(window) = web_sys::window() {
        let _ = js_sys::Reflect::set(&window, &JsValue::from_str(key), value);
    }
}

fn install_step_fn() {
    let step = Closure::<dyn Fn(u32)>::new(|n: u32| {
        PENDING_STEPS.with(|p| p.set(p.get() + n));
    });
    set_global("__bevyStep", step.as_ref());
    step.forget();
}

fn drive_virtual_time(bridge: Res<Bridge>, mut virt: ResMut<Time<Virtual>>) {
    if !(bridge.ready && bridge.frozen_requested) {
        return;
    }
    let pending = PENDING_STEPS.with(|p| p.get());
    if pending > 0 {
        PENDING_STEPS.with(|p| p.set(pending - 1));
        virt.unpause();
    } else {
        virt.pause();
    }
}

fn mark_ready(
    mut bridge: ResMut<Bridge>,
    state: Res<State<AppState>>,
    mut virt: ResMut<Time<Virtual>>,
) {
    if bridge.ready || *state.get() != AppState::Playing {
        return;
    }
    bridge.playing_frames += 1;
    // The frame counted here has been extracted and rendered once Last ran
    // twice in Playing.
    if bridge.playing_frames >= 2 {
        bridge.ready = true;
        if bridge.frozen_requested {
            virt.pause();
        }
        web_sys::console::log_1(&"BEVY_READY".into());
        set_global("__bevyReady", &JsValue::TRUE);
    }
}

fn publish_state(
    bridge: Res<Bridge>,
    mut frame: Local<u32>,
    state: Res<State<AppState>>,
    tick: Res<FixedTick>,
    score: Res<Score>,
    virt: Res<Time<Virtual>>,
    players: Query<&Transform, With<Player>>,
) {
    *frame += 1;
    if !virt.is_paused() && *frame % SNAPSHOT_EVERY != 0 {
        return;
    }
    let (px, py) = players
        .iter()
        .next()
        .map(|t| (t.translation.x, t.translation.y))
        .unwrap_or_default();
    let json = format!(
        r#"{{"state":"{}","tick":{},"score":{},"frozen":{},"ready":{},"player":{{"x":{px:.3},"y":{py:.3}}},"entities":{{"players":{}}}}}"#,
        state.get().as_str(),
        tick.0,
        score.0,
        virt.is_paused(),
        bridge.ready,
        players.iter().count(),
    );
    if let Ok(value) = js_sys::JSON::parse(&json) {
        set_global("__bevyState", &value);
    }
}
