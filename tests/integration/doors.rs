use bevy::ecs::message::MessageCursor;
use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::player::PLAYER_RADIUS;
use leave_it_behind::ship::doors::{DOOR_REACH, Door, DoorCue, OPEN_FRAME};
use leave_it_behind::ship::layout::{WALL_HALF, ship};
use leave_it_behind::ship::{CurrentRoom, RoomId};

use crate::common::{
    frame, key_toward, player_pos, press, put_player, release, run_frames, run_until, secs,
    test_app_with,
};

/// The door between the quarters and the cockpit, and the way through it
/// from the quarters.
fn quarters_door() -> (Vec2, Vec2) {
    for d in ship().doors().iter().filter(|d| !d.locked) {
        let across = if d.across_x { Vec2::Y } else { Vec2::X };
        let pair = (
            RoomId::at(d.center - across * 40.0),
            RoomId::at(d.center + across * 40.0),
        );
        if pair == (RoomId::Quarters, RoomId::Cockpit) {
            return (d.center, across);
        }
        if pair == (RoomId::Cockpit, RoomId::Quarters) {
            return (d.center, -across);
        }
    }
    panic!("no door from the quarters to the cockpit");
}

fn door_at(app: &mut App, center: Vec2) -> Door {
    let mut q = app.world_mut().query::<&Door>();
    *q.iter(app.world())
        .find(|d| d.center == center)
        .expect("a door there")
}

#[test]
fn a_door_opens_as_the_engineer_comes_and_closes_behind_them() {
    let mut app = test_app_with(Scenario::Quiet);
    let (door, way) = quarters_door();
    put_player(&mut app, door - way * (DOOR_REACH + 60.0));
    run_frames(&mut app, 2);
    assert_eq!(
        door_at(&mut app, door).frame,
        0,
        "closed while nobody is near"
    );

    press(&mut app, key_toward(way));
    run_until(&mut app, secs(2.0), |app| door_at(app, door).is_open());
    // Open before the engineer reaches it: they never stop.
    let gap = (door - player_pos(&mut app)).dot(way);
    assert!(
        gap > WALL_HALF + PLAYER_RADIUS,
        "open only {gap} units away"
    );

    run_until(&mut app, secs(3.0), |app| {
        (player_pos(app) - door).dot(way) > DOOR_REACH + 10.0
    });
    release(&mut app, key_toward(way));
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Cockpit);
    run_frames(&mut app, secs(0.5));
    assert_eq!(door_at(&mut app, door).frame, 0, "closed behind them");
}

#[test]
fn a_door_emits_one_opening_cue_and_one_clang_cue() {
    let mut app = test_app_with(Scenario::Quiet);
    let (door, way) = quarters_door();
    let mut cursor = MessageCursor::<DoorCue>::default();
    // Skip cues from the first flight frame near the player's spawn.
    cursor
        .read(app.world().resource::<Messages<DoorCue>>())
        .count();

    put_player(&mut app, door - way * (DOOR_REACH + 60.0));
    run_frames(&mut app, 2);
    cursor
        .read(app.world().resource::<Messages<DoorCue>>())
        .count();

    put_player(&mut app, door - way * (DOOR_REACH - 10.0));
    let mut cues = Vec::new();
    for _ in 0..secs(0.6) {
        frame(&mut app);
        cues.extend(
            cursor
                .read(app.world().resource::<Messages<DoorCue>>())
                .copied(),
        );
    }
    assert!(door_at(&mut app, door).is_open());

    put_player(&mut app, door - way * (DOOR_REACH + 60.0));
    for _ in 0..secs(0.6) {
        frame(&mut app);
        cues.extend(
            cursor
                .read(app.world().resource::<Messages<DoorCue>>())
                .copied(),
        );
    }
    assert_eq!(door_at(&mut app, door).frame, 0);
    assert_eq!(cues, [DoorCue::Opening, DoorCue::Closed]);
}

#[test]
fn a_closed_door_blocks_until_it_has_opened() {
    let mut app = test_app_with(Scenario::Quiet);
    let (door, way) = quarters_door();
    // Right in front of the closed door, walking straight into it.
    put_player(&mut app, door - way * (WALL_HALF + PLAYER_RADIUS + 2.0));
    press(&mut app, key_toward(way));
    run_frames(&mut app, 12);
    assert!(!door_at(&mut app, door).is_open());
    assert!(
        (player_pos(&mut app) - door).dot(way) < -WALL_HALF,
        "went through a closed door"
    );
    run_until(&mut app, secs(1.5), |app| {
        app.world().resource::<CurrentRoom>().0 == RoomId::Cockpit
    });
    assert_eq!(door_at(&mut app, door).frame, OPEN_FRAME);
}

#[test]
fn the_engineer_is_steered_into_the_doorway() {
    let mut app = test_app_with(Scenario::Quiet);
    let (door, way) = quarters_door();
    // 30 units to one side of the door's centre line: without help the
    // door frame would stop them (the gap is 36 units, they are 28 wide).
    put_player(&mut app, door - way * 120.0 + way.perp() * 30.0);
    frame(&mut app);
    press(&mut app, key_toward(way));
    run_until(&mut app, secs(1.5), |app| {
        app.world().resource::<CurrentRoom>().0 == RoomId::Cockpit
    });
}

#[test]
fn the_airlock_hatch_never_opens() {
    let mut app = test_app_with(Scenario::Quiet);
    let hatch = ship()
        .doors()
        .into_iter()
        .find(|d| d.locked)
        .expect("a hatch");
    let across = if hatch.across_x { Vec2::Y } else { Vec2::X };
    let inward = if RoomId::at(hatch.center + across * 40.0) == RoomId::Airlock {
        across
    } else {
        -across
    };
    put_player(&mut app, hatch.center + inward * 80.0);
    frame(&mut app);
    press(&mut app, key_toward(-inward));
    run_frames(&mut app, secs(2.0));
    let depth = (player_pos(&mut app) - hatch.center).dot(inward);
    assert!(
        (depth - (WALL_HALF + PLAYER_RADIUS)).abs() < 0.5,
        "stopped {depth} units from the hatch"
    );
    assert_eq!(app.world().resource::<CurrentRoom>().0, RoomId::Airlock);
}
