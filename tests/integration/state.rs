use bevy::prelude::*;
use leave_it_behind::AppState;

use crate::common::{boot, frame, state, test_app};

#[test]
fn boots_into_playing() {
    let mut app = test_app();
    boot(&mut app);
    assert_eq!(state(&app), AppState::Playing);
}

#[test]
fn state_can_be_driven_by_tests() {
    let mut app = test_app();
    boot(&mut app);
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Lost);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Lost);
}

#[test]
fn gameplay_freezes_outside_playing() {
    let mut app = test_app();
    boot(&mut app);
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Landed);
    frame(&mut app);
    let journey = *app.world().resource::<leave_it_behind::level::Journey>();
    for _ in 0..30 {
        frame(&mut app);
    }
    assert_eq!(
        *app.world().resource::<leave_it_behind::level::Journey>(),
        journey
    );
}
