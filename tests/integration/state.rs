use bevy::prelude::*;
use leave_it_behind::AppState;

use crate::common::{boot, test_app};

#[test]
fn boots_into_playing() {
    let mut app = test_app();
    app.update();
    // Boot requests the transition; it applies on the next frame.
    boot(&mut app);
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Playing
    );
}

#[test]
fn state_can_be_driven_by_tests() {
    let mut app = test_app();
    boot(&mut app);
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Boot);
    app.update();
    assert_eq!(
        *app.world().resource::<State<AppState>>().get(),
        AppState::Boot
    );
}
