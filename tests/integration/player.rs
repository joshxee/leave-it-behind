use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::player::{Facing, Locked, Movement, PLAYER_RADIUS, PLAYER_SPEED, Player};
use leave_it_behind::ship::layout::{self, ship};
use leave_it_behind::ship::{CameraRig, CurrentRoom, RoomId};

use crate::common::{
    aim_at, boot, frame, key_toward, player_pos, press, put_player, release, run_frames, run_until,
    secs, test_app, test_app_with,
};

fn movement(app: &mut App) -> Movement {
    let mut q = app.world_mut().query_filtered::<&Movement, With<Player>>();
    *q.single(app.world()).unwrap()
}

/// The door between two rooms and which way leads from `from` to `to`.
fn door_between(from: RoomId, to: RoomId) -> (Vec2, Vec2) {
    ship()
        .doors()
        .iter()
        .filter(|d| !d.locked)
        .find_map(|d| {
            let across = if d.across_x { Vec2::Y } else { Vec2::X };
            let (a, b) = (
                RoomId::at(d.center + across * 40.0),
                RoomId::at(d.center - across * 40.0),
            );
            if (a, b) == (to, from) {
                Some((d.center, across))
            } else if (a, b) == (from, to) {
                Some((d.center, -across))
            } else {
                None
            }
        })
        .expect("a door between the rooms")
}

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
    run_frames(&mut app, 30);
    let moved = player_pos(&mut app) - start;
    assert!(moved.y.abs() < 1e-3);
    // Half a second of fixed ticks, give or take the tick on the press frame.
    assert!(
        (moved.x - PLAYER_SPEED / 2.0).abs() <= PLAYER_SPEED / 60.0 + 0.05,
        "moved {moved:?}"
    );
    assert!(movement(&mut app).walking);
    assert_eq!(movement(&mut app).heading, Vec2::X);
}

#[test]
fn no_input_no_movement() {
    let mut app = test_app_with(Scenario::Quiet);
    let start = player_pos(&mut app);
    run_frames(&mut app, 30);
    assert_eq!(player_pos(&mut app), start);
    assert!(!movement(&mut app).walking);
}

#[test]
fn walls_stop_the_player() {
    let mut app = test_app_with(Scenario::Quiet);
    // The quarters' only door is in the wall beside the corridor, so there
    // is wall ahead.
    let (door, way) = door_between(RoomId::Quarters, RoomId::ForeCorridor);
    assert_eq!(way.y, 0.0, "the door is to the side");
    assert!((layout::player_spawn().y - door.y).abs() < 1.0);
    press(&mut app, KeyCode::KeyW);
    run_frames(&mut app, secs(2.0));
    let p = player_pos(&mut app);
    let top = RoomId::Quarters.interior().max.y;
    assert!((p.y - (top - PLAYER_RADIUS)).abs() < 1e-3, "{p}");
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
    let (door, way) = door_between(RoomId::ForeCorridor, RoomId::Cockpit);
    put_player(&mut app, door - way * 150.0);
    press(&mut app, key_toward(way));
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
fn helm_to_airlock_takes_nine_to_ten_seconds() {
    let mut app = test_app_with(Scenario::Quiet);
    put_player(&mut app, layout::helm_seat());
    let tail = ship()
        .doors()
        .into_iter()
        .find(|d| d.locked)
        .expect("the airlock hatch");
    let way = (tail.center - layout::helm_seat()).normalize();
    press(&mut app, key_toward(way));
    // Until the engineer is up against the hatch.
    let stop = -(layout::WALL_HALF + PLAYER_RADIUS + 1.0);
    let frames = run_until(&mut app, secs(14.0), |app| {
        (player_pos(app) - tail.center).dot(way) >= stop
    });
    let seconds = frames as f32 / 60.0;
    assert!((9.0..=10.0).contains(&seconds), "took {seconds}s");
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Airlock);
}

#[test]
fn rooms_joined_by_a_door_are_one_to_two_and_a_half_seconds_apart() {
    // Along the spine about two seconds; the rooms beside a corridor are
    // closer.
    for d in ship().doors().iter().filter(|d| !d.locked) {
        let across = if d.across_x { Vec2::Y } else { Vec2::X };
        let (a, b) = (
            RoomId::at(d.center + across * 40.0),
            RoomId::at(d.center - across * 40.0),
        );
        let hop = a.center().distance(b.center()) / PLAYER_SPEED;
        assert!((1.0..=2.5).contains(&hop), "{a:?} -> {b:?}: {hop}s");
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
    assert!(!movement(&mut app).walking, "strapped in is not walking");
    release(&mut app, KeyCode::KeyD);
}
