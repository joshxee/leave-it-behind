use bevy::prelude::*;
use leave_it_behind::faults::bolts::Bolt;
use leave_it_behind::faults::{Fault, FaultKind};
use leave_it_behind::level::FaultPlan;
use leave_it_behind::player::{Focus, InteractKind};
use leave_it_behind::ship::{CurrentRoom, RoomId};
use leave_it_behind::tools::{Tool, ToolBelt, ToolState};
use leave_it_behind::{AppState, Scenario};

use crate::common::{frame, run_frames, run_until, secs, state, test_app_with};

fn kinds(app: &mut App) -> Vec<FaultKind> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).map(|f| f.kind()).collect()
}

#[test]
fn every_scenario_boots_and_runs() {
    for &scenario in Scenario::ALL {
        let mut app = test_app_with(scenario);
        run_frames(&mut app, 10);
        assert_eq!(state(&app), AppState::Playing, "{}", scenario.name());
    }
}

#[test]
fn quiet_never_starts_a_fault() {
    let mut app = test_app_with(Scenario::Quiet);
    assert!(app.world().resource::<FaultPlan>().pending.is_empty());
    run_frames(&mut app, secs(15.0));
    assert!(kinds(&mut app).is_empty());
}

#[test]
fn bolts_puts_a_snapped_wrench_on_a_loose_panel() {
    let mut app = test_app_with(Scenario::Bolts);
    frame(&mut app);
    assert_eq!(kinds(&mut app), vec![FaultKind::LooseBolts]);
    let mut bolts = app.world_mut().query::<&Bolt>();
    assert_eq!(bolts.iter(app.world()).filter(|b| b.loose).count(), 3);
    assert_eq!(app.world().resource::<ToolBelt>().held, Tool::Wrench);
    assert!(app.world().resource::<ToolState>().snap.is_some());
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::PortEngine);
}

#[test]
fn breach_and_drift_put_the_engineer_on_the_spot() {
    let mut app = test_app_with(Scenario::Breach);
    frame(&mut app);
    assert_eq!(kinds(&mut app), vec![FaultKind::HullBreach]);
    assert_eq!(app.world().resource::<ToolBelt>().held, Tool::Tape);
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Airlock);

    let mut app = test_app_with(Scenario::Drift);
    frame(&mut app);
    assert_eq!(kinds(&mut app), vec![FaultKind::TrajectoryDrift]);
    assert_eq!(app.world().resource::<Focus>().0, Some(InteractKind::Helm));
}

#[test]
fn landing_lands_and_breach_critical_loses() {
    let mut app = test_app_with(Scenario::Landing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    let mut app = test_app_with(Scenario::BreachCritical);
    run_until(&mut app, secs(3.0), |app| state(app) == AppState::Lost);
}
