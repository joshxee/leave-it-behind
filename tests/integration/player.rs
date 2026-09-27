use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::player::{Facing, Locked, PLAYER_RADIUS, PLAYER_SPEED, Player};
use leave_it_behind::ship::layout::{self, HALF_HEIGHT};
use leave_it_behind::ship::{CameraRig, CurrentRoom, RoomId};

use crate::common::{
    aim_at, boot, frame, player_pos, press, put_player, run_frames, run_until, secs, test_app,
    test_app_with,
};

#[test]
fn spawns_one_player_in_the_quarters() {
    let mut app = test_app();
    boot(&mut app);
    let mut q = app.world_mut().query::<&Player>();
    assert_eq!(q.iter(app.world()).count(), 1);
    assert_eq!(player_pos(&mut app), layout::player_spawn());
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Quarters);
}

#[test]
fn holding_d_moves_right_at_player_speed() {
    let mut app = test_app_with(Scenario::Quiet);
    let start = player_pos(&mut app);
    press(&mut app, KeyCode::KeyD);
    run_frames(&mut app, 60);
    let moved = player_pos(&mut app) - start;
    assert!(moved.y.abs() < 1e-3);
    // One second of fixed ticks, give or take the tick on the press frame.
    assert!(
        (moved.x - PLAYER_SPEED).abs() <= PLAYER_SPEED / 60.0 + 0.05,
        "moved {moved:?}"
    );
}

#[test]
fn no_input_no_movement() {
    let mut app = test_app_with(Scenario::Quiet);
    let start = player_pos(&mut app);
    run_frames(&mut app, 30);
    assert_eq!(player_pos(&mut app), start);
}

#[test]
fn walls_stop_the_player() {
    let mut app = test_app_with(Scenario::Quiet);
    press(&mut app, KeyCode::KeyW);
    run_frames(&mut app, secs(2.0));
    let p = player_pos(&mut app);
    assert!((p.y - (HALF_HEIGHT - PLAYER_RADIUS)).abs() < 1e-3, "{p}");
}

#[test]
fn facing_follows_the_mouse() {
    let mut app = test_app_with(Scenario::Quiet);
    let p = player_pos(&mut app);
    aim_at(&mut app, p + Vec2::new(0.0, 120.0));
    run_frames(&mut app, 2);
    let mut q = app.world_mut().query::<&Facing>();
    let facing = q.single(app.world()).unwrap().0;
    assert!((facing - Vec2::Y).length() < 1e-3, "{facing}");
}

#[test]
fn walking_through_a_door_cuts_the_camera_to_the_next_room() {
    let mut app = test_app_with(Scenario::Quiet);
    press(&mut app, KeyCode::KeyD);
    run_until(&mut app, secs(3.0), |app| {
        app.world().resource::<CurrentRoom>().0 == RoomId::Cockpit
    });
    frame(&mut app);
    let mut q = app.world_mut().query::<(&CameraRig, &Transform)>();
    let (rig, transform) = q.single(app.world()).unwrap();
    assert_eq!(rig.anchor, RoomId::Cockpit.center());
    assert_eq!(transform.translation.truncate(), RoomId::Cockpit.center());
}

#[test]
fn helm_to_airlock_takes_about_fifteen_seconds() {
    let mut app = test_app_with(Scenario::Quiet);
    put_player(&mut app, layout::helm_seat());
    press(&mut app, KeyCode::KeyA);
    let tail = RoomId::Airlock.interior().min.x + PLAYER_RADIUS + 1.0;
    let frames = run_until(&mut app, secs(18.0), |app| player_pos(app).x <= tail);
    let seconds = frames as f32 / 60.0;
    assert!((13.5..=16.0).contains(&seconds), "took {seconds}s");
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Airlock);
}

#[test]
fn room_hops_take_two_and_a_half_to_four_seconds() {
    for pair in RoomId::ALL.windows(2) {
        let hop = pair[0].center().distance(pair[1].center()) / PLAYER_SPEED;
        assert!(
            (2.5..=4.0).contains(&hop),
            "{:?} -> {:?}: {hop}s",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn a_locked_player_does_not_walk() {
    let mut app = test_app_with(Scenario::Quiet);
    let start = player_pos(&mut app);
    let mut q = app.world_mut().query_filtered::<Entity, With<Player>>();
    let player = q.single(app.world()).unwrap();
    app.world_mut().entity_mut(player).insert(Locked);
    press(&mut app, KeyCode::KeyD);
    run_frames(&mut app, 30);
    assert_eq!(player_pos(&mut app), start);
}
