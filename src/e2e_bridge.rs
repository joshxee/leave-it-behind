//! Browser test bridge. Compiled only for `wasm32` with the `e2e` feature;
//! `scripts/build-web.sh` fails a non-e2e build that contains any of this.
//!
//! - `window.__bevyReady = true` (and console `BEVY_READY`) after the first
//!   rendered frame of the first screen: the title (plain URL) or a flight
//!   (`?scenario=`).
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

use crate::alarm::Alarm;
use crate::coach::{Coach, lines};
use crate::determinism::FixedTick;
use crate::diagnostics::Diagnostics;
use crate::faults::bolts::Bolt;
use crate::faults::drift::{Nav, in_band};
use crate::faults::{Fault, Vitals};
use crate::level::{
    CurrentLevel, Damage, FaultPlan, Journey, LastRun, Progress, RunRecord, RunStats,
};
use crate::menu::{Menu, MenuCtx, MenuRow, content};
use crate::player::sprite::EngineerSprite;
use crate::player::{Facing, Focus, InteractKind, Locked, Movement, Player};
use crate::ship::doors::Door;
use crate::ship::layout::console_point;
use crate::ship::{CameraRig, CurrentRoom};
use crate::tools::{TapeStrip, ToolBelt, ToolState};
use crate::ui::{Gauge, Notices, gauge};
use crate::upgrades::{Upgrade, Upgrades};
use crate::{AppState, Pause};

const SNAPSHOT_EVERY: u32 = 2;

thread_local! {
    static PENDING_STEPS: Cell<u32> = const { Cell::new(0) };
}

#[derive(Resource, Default)]
struct Bridge {
    frozen_requested: bool,
    ready: bool,
    shown_frames: u32,
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
    if bridge.ready || *state.get() == AppState::Boot {
        return;
    }
    bridge.shown_frames += 1;
    // The frame counted here has been extracted and rendered once Last ran
    // twice past Boot.
    if bridge.shown_frames >= 2 {
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
    pause: Option<Res<'w, State<Pause>>>,
    menu: Res<'w, Menu>,
    menu_ctx: MenuCtx<'w>,
    level: Res<'w, CurrentLevel>,
    upgrades: Res<'w, Upgrades>,
    progress: Res<'w, Progress>,
    last_run: Res<'w, LastRun>,
    notices: Res<'w, Notices>,
    rows: Query<
        'w,
        's,
        (
            &'static MenuRow,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
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
    vitals: Res<'w, Vitals>,
    stats: Res<'w, RunStats>,
    plan: Res<'w, FaultPlan>,
    coach: Res<'w, Coach>,
    players: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Facing,
            &'static Movement,
            Has<Locked>,
        ),
        With<Player>,
    >,
    sprites: Query<'w, 's, &'static EngineerSprite>,
    doors: Query<'w, 's, &'static Door>,
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

fn damage_json(d: &Damage) -> String {
    format!(
        r#"{{"oxygen":{:.3},"course":{:.3},"engine":{:.3},"total":{:.3}}}"#,
        d.oxygen,
        d.course,
        d.engine,
        d.total()
    )
}

fn record_json(r: Option<&RunRecord>) -> String {
    r.map_or("null".to_string(), |r| {
        format!(
            r#"{{"landed":{},"survived":{:.3},"damage":{}}}"#,
            r.landed,
            r.survived,
            damage_json(&r.damage)
        )
    })
}

/// JSON string literal (menu text is ASCII without quotes or backslashes,
/// but escape them anyway).
fn js_str(text: &str) -> String {
    format!(r#""{}""#, text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The open screen, its rows and each row's rectangle in canvas pixels
/// (so Playwright can click a row by its label).
fn menu_json(s: &Snapshot) -> String {
    let Some(top) = s.menu.top() else {
        return "null".to_string();
    };
    let ctx = s.menu_ctx.get();
    let items = content(top.screen, &ctx).items;
    let mut rows = String::new();
    for (i, item) in items.iter().enumerate() {
        let rect = s
            .rows
            .iter()
            .find(|(row, _, _)| row.0 == i)
            .map(|(_, node, transform)| {
                let size = node.size * node.inverse_scale_factor;
                let center = transform.translation * node.inverse_scale_factor;
                format!(
                    r#","x":{:.1},"y":{:.1},"w":{:.1},"h":{:.1}"#,
                    center.x, center.y, size.x, size.y
                )
            })
            .unwrap_or_default();
        let _ = write!(
            rows,
            r#"{}{{"label":{}{}}}"#,
            if i > 0 { "," } else { "" },
            js_str(&item.label(s.menu_ctx.settings())),
            rect
        );
    }
    format!(
        r#"{{"screen":"{}","depth":{},"focus":{},"items":[{}]}}"#,
        top.screen.as_str(),
        s.menu.stack.len(),
        top.focus,
        rows
    )
}

fn progress_json(s: &Snapshot) -> String {
    let level = s.progress.level(&s.level.0.id);
    format!(
        r#"{{"flights":{},"landings":{},"best":{}}}"#,
        level.map_or(0, |l| l.flights),
        level.map_or(0, |l| l.landings),
        record_json(level.and_then(|l| l.best.as_ref()))
    )
}

/// The level being flown: its id, place in the campaign and name.
fn level_json(s: &Snapshot) -> String {
    format!(
        r#"{{"id":{},"number":{},"name":{}}}"#,
        js_str(&s.level.0.id),
        s.level.number(),
        js_str(&s.level.0.name)
    )
}

/// Times each upgrade has been picked this campaign.
fn upgrades_json(s: &Snapshot) -> String {
    format!(
        r#"{{"runFaster":{},"fasterWrench":{},"widerTape":{}}}"#,
        s.upgrades.count(Upgrade::RunFaster),
        s.upgrades.count(Upgrade::FasterWrench),
        s.upgrades.count(Upgrade::WiderTape)
    )
}

fn settings_json(s: &Snapshot) -> String {
    let settings = s.menu_ctx.settings();
    format!(
        r#"{{"shake":{},"flash":{},"controlsHint":{},"tips":{},"pauseUnfocused":{},"fullscreen":{},"vsync":{}}}"#,
        settings.shake,
        settings.flash,
        settings.controls_hint,
        settings.tips_on(),
        settings.pause_unfocused,
        settings.fullscreen,
        settings.vsync
    )
}

/// What the coaching panel says (empty outside a flight, as on screen).
fn coach_json(s: &Snapshot) -> String {
    let shown = if *s.state.get() == AppState::Playing {
        lines(
            &s.coach,
            &s.journey,
            &s.menu_ctx.settings().tips_seen,
            &s.faults,
        )
    } else {
        Vec::new()
    };
    let shown: Vec<String> = shown.into_iter().map(js_str).collect();
    format!(
        r#"{{"active":{},"lines":[{}]}}"#,
        s.coach.active,
        shown.join(",")
    )
}

/// What the vitals panel's rows say, top to bottom.
fn gauges_json(s: &Snapshot) -> String {
    let rows: Vec<String> = Gauge::ALL
        .map(|g| gauge(g, &s.vitals, s.faults.iter()))
        .iter()
        .map(|r| {
            format!(
                r#"{{"label":{},"value":{},"alert":{}}}"#,
                js_str(r.label),
                js_str(&r.value),
                r.alert
            )
        })
        .collect();
    format!("[{}]", rows.join(","))
}

fn snapshot_json(bridge: &Bridge, s: &Snapshot) -> String {
    let (pos, facing, walking, locked) = s
        .players
        .iter()
        .next()
        .map(|(t, f, m, l)| (t.translation.truncate(), f.0, m.walking, l))
        .unwrap_or_default();
    let (pose, dir) = s
        .sprites
        .iter()
        .next()
        .map_or(("none", 0), |e| (e.action.as_str(), e.dir16));
    let mut doors = String::new();
    for (i, d) in s.doors.iter().enumerate() {
        let _ = write!(
            doors,
            r#"{}{{"x":{:.1},"y":{:.1},"frame":{},"open":{}}}"#,
            if i > 0 { "," } else { "" },
            d.center.x,
            d.center.y,
            d.frame,
            d.is_open(),
        );
    }
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
            s.vitals.time_left(f),
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
    let tape_contact = s.tools.taping.map_or("null".to_string(), |c| {
        format!(
            r#"{{"x":{:.3},"y":{:.3},"nx":{:.3},"ny":{:.3}}}"#,
            c.point.x, c.point.y, c.normal.x, c.normal.y
        )
    });
    let paused = s.pause.as_ref().is_some_and(|p| *p.get() == Pause::Paused);
    let last_run = s.last_run.record.as_ref().map_or("null".to_string(), |r| {
        format!(
            r#"{{"record":{},"newBest":{}}}"#,
            record_json(Some(r)),
            s.last_run.new_best
        )
    });
    let notice = s.notices.current().map_or("null".to_string(), js_str);
    let console = console_point();
    format!(
        concat!(
            r#"{{"state":"{}","paused":{},"menu":{},"settings":{},"progress":{},"level":{},"upgrades":{},"lastRun":{},"notice":{},"#,
            r#""tick":{},"frozen":{},"ready":{},"#,
            r#""room":"{}","camera":{{"x":{:.1},"y":{:.1}}},"#,
            r#""player":{{"x":{:.3},"y":{:.3},"fx":{:.3},"fy":{:.3},"locked":{},"walking":{},"pose":"{}","dir":{}}},"focus":{},"#,
            r#""tool":"{}","tape":{:.3},"snap":{},"turning":{},"taping":{},"tapeContact":{},"strips":{},"#,
            r#""journey":{{"elapsed":{:.3},"launched":{},"cleared":{}}},"#,
            r#""coach":{},"console":{{"x":{:.1},"y":{:.1}}},"#,
            r#""faults":[{}],"looseBolts":[{}],"doors":[{}],"#,
            r#""nav":{{"engaged":{},"x":{:.3},"y":{:.3},"inBand":{}}},"#,
            r#""diag":"{}","diagUses":{},"#,
            r#""alarm":{{"level":{:.3},"active":{},"jolt":{:.3}}},"#,
            r#""vitals":{{"oxygen":{:.3},"heat":{:.3},"gauges":{}}},"#,
            r#""stats":{{"started":{},"fixed":{},"total":{},"failure":{},"damage":{}}},"#,
            r#""entities":{{"players":{},"faults":{},"tapeStrips":{}}}}}"#,
        ),
        s.state.get().as_str(),
        paused,
        menu_json(s),
        settings_json(s),
        progress_json(s),
        level_json(s),
        upgrades_json(s),
        last_run,
        notice,
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
        walking,
        pose,
        dir,
        focus,
        s.belt.held.as_str(),
        s.belt.tape_left,
        s.tools.snap.is_some(),
        s.tools.turn.is_some(),
        s.tools.taping.is_some(),
        tape_contact,
        s.strips.iter().count(),
        s.journey.elapsed,
        s.journey.launched,
        s.journey.cleared(),
        coach_json(s),
        console.x,
        console.y,
        faults,
        bolts,
        doors,
        s.nav.engaged,
        s.nav.marker.x,
        s.nav.marker.y,
        in_band(s.nav.marker),
        s.diag.view.as_str(),
        s.diag.uses,
        s.alarm.level,
        s.alarm.active,
        s.alarm.jolt,
        s.vitals.oxygen_left(),
        s.vitals.heat.spent,
        gauges_json(s),
        s.stats.started,
        s.stats.fixed,
        s.plan.total(&s.stats),
        failure,
        damage_json(&s.stats.damage),
        s.players.iter().count(),
        s.faults.iter().count(),
        s.strips.iter().count(),
    )
}
