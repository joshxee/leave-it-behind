use bevy::prelude::*;
use leave_it_behind::faults::drift::{HOLD_SECS, Nav};
use leave_it_behind::faults::{Fault, FaultKind, Site};
use leave_it_behind::level::RunStats;
use leave_it_behind::player::{Locked, Player};
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    frame, press, release, run_frames, run_until, secs, state, tap, test_app_with,
};

fn faults(app: &mut App) -> Vec<Fault> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).copied().collect()
}

fn player_locked(app: &mut App) -> bool {
    let mut q = app
        .world_mut()
        .query_filtered::<Has<Locked>, With<Player>>();
    q.single(app.world()).unwrap()
}

/// Holds WASD toward the centre of the nav display, like a player would.
fn steer_to_centre(app: &mut App) {
    let marker = app.world().resource::<Nav>().marker;
    let keys = [
        (KeyCode::KeyA, marker.x > 0.05),
        (KeyCode::KeyD, marker.x < -0.05),
        (KeyCode::KeyS, marker.y > 0.05),
        (KeyCode::KeyW, marker.y < -0.05),
    ];
    for (key, down) in keys {
        if down {
            press(app, key);
        } else {
            release(app, key);
        }
    }
}

#[test]
fn an_unfixed_fault_ends_the_run_with_its_cause() {
    let mut app = test_app_with(Scenario::BreachCritical);
    run_until(&mut app, secs(3.0), |app| state(app) == AppState::Lost);
    let failure = app.world().resource::<RunStats>().failure;
    assert_eq!(failure, Some(Site::AirlockPortAft));
    assert_eq!(failure.unwrap().kind(), FaultKind::HullBreach);
}

#[test]
fn a_drifting_course_pulls_the_marker_off_centre() {
    let mut app = test_app_with(Scenario::Drift);
    run_frames(&mut app, secs(1.0));
    let marker = app.world().resource::<Nav>().marker;
    assert!(marker.length() > 0.4, "{marker}");
}

#[test]
fn e_at_the_helm_locks_the_engineer_and_e_again_releases() {
    let mut app = test_app_with(Scenario::Drift);
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert!(app.world().resource::<Nav>().engaged);
    assert!(player_locked(&mut app));
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert!(!app.world().resource::<Nav>().engaged);
    assert!(!player_locked(&mut app));
}

#[test]
fn steering_into_the_band_and_holding_fixes_the_course() {
    let mut app = test_app_with(Scenario::Drift);
    tap(&mut app, KeyCode::KeyE);
    let frames = run_until(&mut app, secs(12.0), |app| {
        steer_to_centre(app);
        faults(app).is_empty()
    });
    let seconds = frames as f32 / 60.0;
    assert!(seconds >= HOLD_SECS, "{seconds}s");
    assert!(seconds <= HOLD_SECS + 2.5, "took {seconds}s");
    assert_eq!(app.world().resource::<RunStats>().fixed, 1);
    frame(&mut app);
    assert!(
        !app.world().resource::<Nav>().engaged,
        "a fix releases the helm"
    );
    assert!(!player_locked(&mut app));
}

#[test]
fn wasd_steers_the_marker_not_the_engineer() {
    let mut app = test_app_with(Scenario::Drift);
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    let before = app.world().resource::<Nav>().marker;
    press(&mut app, KeyCode::KeyW);
    run_frames(&mut app, 20);
    let after = app.world().resource::<Nav>().marker;
    assert!(
        after.y > before.y + 0.1 || after.y == 1.0,
        "{before} -> {after}"
    );
}

#[test]
fn clocks_are_independent() {
    let mut app = test_app_with(Scenario::Scramble);
    run_frames(&mut app, secs(5.0));
    let all = faults(&mut app);
    assert_eq!(all.len(), 3);
    for f in &all {
        assert!((f.elapsed - 5.0).abs() < 0.05, "{f:?}");
    }
    let mut kinds: Vec<_> = all.iter().map(|f| f.kind()).collect();
    kinds.sort_by_key(|k| k.as_str());
    assert_eq!(
        kinds,
        vec![
            FaultKind::HullBreach,
            FaultKind::LooseBolts,
            FaultKind::TrajectoryDrift
        ]
    );
}
