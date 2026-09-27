//! The ship's vitals (oxygen and engine heat pools) and the HUD panel that
//! shows them.

use bevy::prelude::*;
use leave_it_behind::faults::vitals::RECOVER_SECS;
use leave_it_behind::faults::{Fault, FaultKind, Site, Vitals, fault_bundle};
use leave_it_behind::level::RunStats;
use leave_it_behind::palette;
use leave_it_behind::scenarios::BREACH_CLOCK;
use leave_it_behind::ui::{Gauge, GaugeLabel, GaugeValue};
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    frame, mouse_down, mouse_up, run_frames, run_until, secs, state, test_app_with,
};

fn vitals(app: &App) -> Vitals {
    *app.world().resource::<Vitals>()
}

fn set_repair(app: &mut App, repair: f32) {
    let mut q = app.world_mut().query::<&mut Fault>();
    for mut fault in q.iter_mut(app.world_mut()) {
        fault.repair = repair;
    }
}

/// What the panel's row for `gauge` says: (label, value, label colour).
fn row(app: &mut App, gauge: Gauge) -> (String, String, Color) {
    let mut labels = app.world_mut().query::<(&GaugeLabel, &Text, &TextColor)>();
    let (label, color) = labels
        .iter(app.world())
        .find(|(l, _, _)| l.0 == gauge)
        .map(|(_, t, c)| (t.0.clone(), c.0))
        .unwrap();
    let mut values = app.world_mut().query::<(&GaugeValue, &Text)>();
    let value = values
        .iter(app.world())
        .find(|(v, _)| v.0 == gauge)
        .map(|(_, t)| t.0.clone())
        .unwrap();
    (label, value, color)
}

fn is_warm(c: Color) -> bool {
    let s = c.to_srgba();
    s.red > s.blue + 0.3
}

#[test]
fn a_breach_drains_the_air_at_its_clock() {
    let mut app = test_app_with(Scenario::Breach);
    run_frames(&mut app, secs(10.0));
    let left = vitals(&app).oxygen_left();
    let expected = 1.0 - 10.0 / BREACH_CLOCK;
    assert!((left - expected).abs() < 0.02, "{left} vs {expected}");
    assert_eq!(vitals(&app).heat.spent, 0.0);
}

#[test]
fn a_breach_on_low_air_fails_long_before_its_clock() {
    let mut app = test_app_with(Scenario::SecondBreach);
    let frames = run_until(&mut app, secs(BREACH_CLOCK), |app| {
        state(app) == AppState::Lost
    });
    let took = frames as f32 / 60.0;
    // 40% of the air left, drained at a full tank per clock.
    assert!(
        (took - 0.4 * BREACH_CLOCK).abs() < 1.0,
        "lost after {took}s"
    );
    let failure = app.world().resource::<RunStats>().failure;
    assert_eq!(failure.map(|s| s.kind()), Some(FaultKind::HullBreach));
}

#[test]
fn two_breaches_drain_twice_as_fast() {
    let mut app = test_app_with(Scenario::Breach);
    app.world_mut()
        .spawn(fault_bundle(Site::HullStarboardMid, BREACH_CLOCK));
    run_frames(&mut app, secs(10.0));
    let left = vitals(&app).oxygen_left();
    let expected = 1.0 - 20.0 / BREACH_CLOCK;
    assert!((left - expected).abs() < 0.02, "{left} vs {expected}");
}

#[test]
fn sealing_the_breach_lets_the_air_refill() {
    let mut app = test_app_with(Scenario::Breach);
    run_frames(&mut app, secs(2.0));
    mouse_down(&mut app);
    run_until(&mut app, secs(6.0), |app| {
        app.world().resource::<RunStats>().fixed == 1
    });
    mouse_up(&mut app);
    let sealed = vitals(&app).oxygen_left();
    assert!(sealed < 0.95, "{sealed}");
    run_frames(&mut app, secs(RECOVER_SECS));
    assert_eq!(vitals(&app).oxygen_left(), 1.0);
    assert_eq!(state(&app), AppState::Playing);
}

#[test]
fn each_tight_bolt_slows_the_heat() {
    let mut app = test_app_with(Scenario::Bolts);
    let rate = |app: &mut App| {
        let before = vitals(app).heat.spent;
        run_frames(app, secs(5.0));
        (vitals(app).heat.spent - before) / 5.0
    };
    let all_loose = rate(&mut app);
    set_repair(&mut app, 2.0 / 3.0);
    let one_loose = rate(&mut app);
    assert!(all_loose > 0.0);
    assert!(
        (one_loose * 3.0 - all_loose).abs() < 1e-3,
        "{all_loose} -> {one_loose}"
    );
}

#[test]
fn the_panel_names_the_failing_system_in_warm_colours() {
    let mut app = test_app_with(Scenario::SecondBreach);
    frame(&mut app);
    let (label, value, color) = row(&mut app, Gauge::Oxygen);
    assert_eq!((label.as_str(), value.as_str()), ("O2", "40%"));
    assert!(is_warm(color), "{color:?}");
    let (label, value, color) = row(&mut app, Gauge::Heat);
    assert_eq!((label.as_str(), value.as_str()), ("HEAT", "0%"));
    assert_eq!(color, palette::UI_DIM);
    let (label, value, _) = row(&mut app, Gauge::Course);
    assert_eq!((label.as_str(), value.as_str()), ("COURSE", "OK"));
}

#[test]
fn the_panel_counts_down_to_impact() {
    let mut app = test_app_with(Scenario::Drift);
    run_frames(&mut app, secs(10.5));
    let (label, value, color) = row(&mut app, Gauge::Course);
    assert_eq!((label.as_str(), value.as_str()), ("IMPACT IN", "0:50"));
    assert!(is_warm(color), "{color:?}");
}
