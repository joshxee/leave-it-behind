//! Browser test bridge. Compiled only for `wasm32` with the `e2e` feature;
//! `scripts/build-web.sh` fails a non-e2e build that contains any of this.
//!
//! - `window.__bevyReady = true` (and console `BEVY_READY`) after the first
//!   rendered frame in `AppState::Playing`.
//! - `window.__bevyState`: JSON-compatible snapshot, refreshed every
//!   [`SNAPSHOT_EVERY`] frames (every frame while frozen). Its shape is the
//!   `BevyState` type in `e2e/specs/helpers.ts`.
//! - `?freeze=1`: pause `Time<Virtual>` once ready.
//! - `window.__bevyStep(n)`: advance exactly `n` fixed ticks (one per frame,
//!   since each frame advances virtual time by one fixed step), then pause.

use std::cell::Cell;
use std::fmt::Write;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::time::TimeSystems;
use wasm_bindgen::prelude::*;

use crate::AppState;
use crate::alarm::Alarm;
use crate::determinism::FixedTick;
use crate::diagnostics::Diagnostics;
use crate::faults::Fault;
use crate::faults::bolts::Bolt;
use crate::faults::drift::{Nav, in_band};
use crate::level::{Journey, RunStats};
use crate::player::{Facing, Focus, InteractKind, Locked, Player};
use crate::ship::{CameraRig, CurrentRoom};
use crate::tools::{TapeStrip, ToolBelt, ToolState};

const SNAPSHOT_EVERY: u32 = 2;

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
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get(key)
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

/// Everything the snapshot reads, bundled to stay under the parameter limit.
#[derive(SystemParam)]
struct Snapshot<'w, 's> {
    state: Res<'w, State<AppState>>,
    tick: Res<'w, FixedTick>,
    virt: Res<'w, Time<Virtual>>,
    room: Res<'w, CurrentRoom>,
    focus: Res<'w, Focus>,
    belt: Res<'w, ToolBelt>,
    tools: Res<'w, ToolState>,
    journey: Res<'w, Journey>,
    nav: Res<'w, Nav>,
    diag: Res<'w, Diagnostics>,
    alarm: Res<'w, Alarm>,
    stats: Res<'w, RunStats>,
    players: Query<'w, 's, (&'static Transform, &'static Facing, Has<Locked>), With<Player>>,
    rigs: Query<'w, 's, &'static CameraRig>,
    faults: Query<'w, 's, &'static Fault>,
    bolts: Query<'w, 's, &'static Bolt>,
    strips: Query<'w, 's, (), With<TapeStrip>>,
}

fn publish_state(bridge: Res<Bridge>, mut frame: Local<u32>, s: Snapshot) {
    *frame += 1;
    if !s.virt.is_paused() && !frame.is_multiple_of(SNAPSHOT_EVERY) {
        return;
    }
    let json = snapshot_json(&bridge, &s);
    if let Ok(value) = js_sys::JSON::parse(&json) {
        set_global("__bevyState", &value);
    }
}

fn snapshot_json(bridge: &Bridge, s: &Snapshot) -> String {
    let (pos, facing, locked) = s
        .players
        .iter()
        .next()
        .map(|(t, f, l)| (t.translation.truncate(), f.0, l))
        .unwrap_or_default();
    let camera = s.rigs.iter().next().map(|r| r.anchor).unwrap_or_default();
    let focus = match s.focus.0 {
        Some(InteractKind::Helm) => r#""Helm""#,
        Some(InteractKind::Diagnostics) => r#""Diagnostics""#,
        None => "null",
    };
    let mut faults = String::new();
    for (i, f) in s.faults.iter().enumerate() {
        let p = f.site.pos();
        let _ = write!(
            faults,
            r#"{}{{"kind":"{}","site":"{}","room":"{}","remaining":{:.3},"repair":{:.3},"x":{:.1},"y":{:.1}}}"#,
            if i > 0 { "," } else { "" },
            f.kind().as_str(),
            f.site.as_str(),
            f.site.room().as_str(),
            f.remaining(),
            f.repair,
            p.x,
            p.y,
        );
    }
    let mut bolts = String::new();
    for (i, b) in s.bolts.iter().filter(|b| b.loose).enumerate() {
        let _ = write!(
            bolts,
            r#"{}{{"x":{:.1},"y":{:.1}}}"#,
            if i > 0 { "," } else { "" },
            b.pos.x,
            b.pos.y
        );
    }
    let failure = s.stats.failure.map_or("null".to_string(), |site| {
        format!(r#""{}""#, site.kind().as_str())
    });
    format!(
        concat!(
            r#"{{"state":"{}","tick":{},"frozen":{},"ready":{},"#,
            r#""room":"{}","camera":{{"x":{:.1},"y":{:.1}}},"#,
            r#""player":{{"x":{:.3},"y":{:.3},"fx":{:.3},"fy":{:.3},"locked":{}}},"focus":{},"#,
            r#""tool":"{}","tape":{:.3},"snap":{},"turning":{},"taping":{},"strips":{},"#,
            r#""journey":{{"elapsed":{:.3},"remaining":{:.3},"duration":{:.1}}},"#,
            r#""faults":[{}],"looseBolts":[{}],"#,
            r#""nav":{{"engaged":{},"x":{:.3},"y":{:.3},"inBand":{}}},"#,
            r#""diag":"{}","diagUses":{},"#,
            r#""alarm":{{"level":{:.3},"active":{},"jolt":{:.3}}},"#,
            r#""stats":{{"started":{},"fixed":{},"failure":{}}},"#,
            r#""entities":{{"players":{},"faults":{},"tapeStrips":{}}}}}"#,
        ),
        s.state.get().as_str(),
        s.tick.0,
        s.virt.is_paused(),
        bridge.ready,
        s.room.0.as_str(),
        camera.x,
        camera.y,
        pos.x,
        pos.y,
        facing.x,
        facing.y,
        locked,
        focus,
        s.belt.held.as_str(),
        s.belt.tape_left,
        s.tools.snap.is_some(),
        s.tools.turn.is_some(),
        s.tools.taping.is_some(),
        s.strips.iter().count(),
        s.journey.elapsed,
        s.journey.remaining(),
        s.journey.duration,
        faults,
        bolts,
        s.nav.engaged,
        s.nav.marker.x,
        s.nav.marker.y,
        in_band(s.nav.marker),
        s.diag.view.as_str(),
        s.diag.uses,
        s.alarm.level,
        s.alarm.active,
        s.alarm.jolt,
        s.stats.started,
        s.stats.fixed,
        failure,
        s.players.iter().count(),
        s.faults.iter().count(),
        s.strips.iter().count(),
    )
}
