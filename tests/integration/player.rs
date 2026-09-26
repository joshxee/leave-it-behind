use bevy::prelude::*;
use leave_it_behind::player::{PLAYER_SPEED, Player};

use crate::common::{boot, run_frames, test_app};

fn player_pos(app: &mut App) -> Vec2 {
    let mut q = app.world_mut().query_filtered::<&Transform, With<Player>>();
    q.single(app.world()).unwrap().translation.truncate()
}

#[test]
fn spawns_exactly_one_player() {
    let mut app = test_app();
    boot(&mut app);
    let mut q = app.world_mut().query::<&Player>();
    assert_eq!(q.iter(app.world()).count(), 1);
}

#[test]
fn holding_right_moves_right_at_player_speed() {
    let mut app = test_app();
    boot(&mut app);
    let start = player_pos(&mut app);

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowRight);
    run_frames(&mut app, 60);

    let moved = player_pos(&mut app) - start;
    assert!(moved.y.abs() < 1e-3);
    // One second of fixed ticks, give or take the tick on the press frame.
    assert!(
        (moved.x - PLAYER_SPEED).abs() <= PLAYER_SPEED / 60.0 + 1e-3,
        "moved {moved:?}"
    );
}

#[test]
fn no_input_no_movement() {
    let mut app = test_app();
    boot(&mut app);
    let start = player_pos(&mut app);
    run_frames(&mut app, 30);
    assert_eq!(player_pos(&mut app), start);
}
