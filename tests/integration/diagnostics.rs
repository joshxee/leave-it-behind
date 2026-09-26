use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::diagnostics::{DiagView, Diagnostics, SCAN_SECS};
use leave_it_behind::player::{Focus, InteractKind};

use crate::common::{frame, press, run_frames, run_until, secs, tap, test_app_with};

fn view(app: &App) -> DiagView {
    app.world().resource::<Diagnostics>().view
}

#[test]
fn the_console_is_in_reach_from_its_stand() {
    let mut app = test_app_with(Scenario::Diagnostics);
    frame(&mut app);
    assert_eq!(
        app.world().resource::<Focus>().0,
        Some(InteractKind::Diagnostics)
    );
}

#[test]
fn e_scans_then_shows_the_faults() {
    let mut app = test_app_with(Scenario::Diagnostics);
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert!(matches!(view(&app), DiagView::Scanning { .. }));
    assert_eq!(app.world().resource::<Diagnostics>().uses, 1);
    let frames = run_until(&mut app, secs(2.0), |app| view(app) == DiagView::Open);
    assert!((frames as f32 / 60.0 - SCAN_SECS).abs() < 0.05);
}

#[test]
fn e_again_closes_it() {
    let mut app = test_app_with(Scenario::Diagnostics);
    tap(&mut app, KeyCode::KeyE);
    run_frames(&mut app, secs(1.5));
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert_eq!(view(&app), DiagView::Closed);
}

#[test]
fn walking_away_closes_it() {
    let mut app = test_app_with(Scenario::Diagnostics);
    tap(&mut app, KeyCode::KeyE);
    run_frames(&mut app, secs(1.5));
    assert_eq!(view(&app), DiagView::Open);
    press(&mut app, KeyCode::KeyS);
    run_until(&mut app, secs(1.0), |app| view(app) == DiagView::Closed);
}

#[test]
fn e_away_from_the_console_does_nothing() {
    let mut app = test_app_with(Scenario::Quiet);
    tap(&mut app, KeyCode::KeyE);
    run_frames(&mut app, 3);
    assert_eq!(view(&app), DiagView::Closed);
    assert_eq!(app.world().resource::<Diagnostics>().uses, 0);
}
