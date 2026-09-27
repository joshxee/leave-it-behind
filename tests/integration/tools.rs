use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::faults::bolts::Bolt;
use leave_it_behind::faults::{Fault, Site};
use leave_it_behind::level::{Journey, RunStats};
use leave_it_behind::ship::RoomId;
use leave_it_behind::ship::layout::HALF_HEIGHT;
use leave_it_behind::tools::{
    TAPE_CAPACITY, TapeStrip, Tool, ToolBelt, ToolState, WRENCH_TURN_SECS,
};

use crate::common::{
    aim_at, click, frame, mouse_down, mouse_up, player_pos, press, put_player, release, run_frames,
    run_until, scroll, secs, tap, test_app_with,
};

fn held(app: &App) -> Tool {
    app.world().resource::<ToolBelt>().held
}

fn tape_left(app: &App) -> f32 {
    app.world().resource::<ToolBelt>().tape_left
}

fn loose_bolts(app: &mut App) -> Vec<Vec2> {
    let mut q = app.world_mut().query::<&Bolt>();
    q.iter(app.world())
        .filter(|b| b.loose)
        .map(|b| b.pos)
        .collect()
}

fn faults(app: &mut App) -> Vec<Fault> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).copied().collect()
}

#[test]
fn number_keys_and_the_wheel_switch_tools() {
    let mut app = test_app_with(Scenario::Quiet);
    assert_eq!(held(&app), Tool::Wrench);
    tap(&mut app, KeyCode::Digit2);
    frame(&mut app);
    assert_eq!(held(&app), Tool::Tape);
    tap(&mut app, KeyCode::Digit1);
    frame(&mut app);
    assert_eq!(held(&app), Tool::Wrench);
    scroll(&mut app, -1.0);
    frame(&mut app);
    assert_eq!(held(&app), Tool::Tape);
    scroll(&mut app, 1.0);
    frame(&mut app);
    assert_eq!(held(&app), Tool::Wrench);
}

#[test]
fn the_wrench_head_snaps_to_a_loose_bolt_in_reach() {
    let mut app = test_app_with(Scenario::Bolts);
    frame(&mut app);
    let state = app.world().resource::<ToolState>().clone();
    let bolt = Site::PortEngineInner.bolts().unwrap()[0];
    assert!(state.snap.is_some());
    assert_eq!(state.head, bolt);
    assert_ne!(state.tip, bolt, "the head is pulled off its rest position");
}

#[test]
fn a_click_turns_the_snapped_bolt_tight() {
    let mut app = test_app_with(Scenario::Bolts);
    frame(&mut app);
    assert_eq!(loose_bolts(&mut app).len(), 3);
    click(&mut app);
    run_frames(&mut app, secs(WRENCH_TURN_SECS) + 3);
    assert_eq!(loose_bolts(&mut app).len(), 2);
    let fault = faults(&mut app)[0];
    assert!((fault.repair - 1.0 / 3.0).abs() < 1e-4);
}

#[test]
fn pulling_the_wrench_away_cancels_the_turn() {
    let mut app = test_app_with(Scenario::Bolts);
    frame(&mut app);
    click(&mut app);
    press(&mut app, KeyCode::KeyS);
    run_frames(&mut app, secs(1.0));
    release(&mut app, KeyCode::KeyS);
    run_frames(&mut app, secs(1.0));
    assert_eq!(loose_bolts(&mut app).len(), 3);
    assert!(app.world().resource::<ToolState>().turn.is_none());
}

#[test]
fn walking_the_panel_fixes_loose_bolts_in_about_three_seconds() {
    let mut app = test_app_with(Scenario::Bolts);
    let bolts = Site::PortEngineInner.bolts().unwrap();
    let start = app.world().resource::<Journey>().elapsed;
    for bolt in bolts {
        // Walk along the panel until level with the bolt, aiming at it.
        let dir = if bolt.x > player_pos(&mut app).x + 2.0 {
            KeyCode::KeyD
        } else {
            KeyCode::KeyA
        };
        if (bolt.x - player_pos(&mut app).x).abs() > 2.0 {
            press(&mut app, dir);
            run_until(&mut app, secs(2.0), |app| {
                (bolt.x - player_pos(app).x).abs() <= 4.0
            });
            release(&mut app, dir);
        }
        aim_at(&mut app, bolt);
        frame(&mut app);
        click(&mut app);
        run_until(&mut app, secs(2.0), |app| !loose_bolts(app).contains(&bolt));
    }
    run_until(&mut app, 10, |app| faults(app).is_empty());
    let took = app.world().resource::<Journey>().elapsed - start;
    assert!(took <= 4.5, "took {took}s");
    assert_eq!(app.world().resource::<RunStats>().fixed, 1);
}

#[test]
fn holding_tape_on_a_breach_seals_it_in_about_three_seconds() {
    let mut app = test_app_with(Scenario::Breach);
    mouse_down(&mut app);
    let frames = run_until(&mut app, secs(5.0), |app| faults(app).is_empty());
    mouse_up(&mut app);
    let seconds = frames as f32 / 60.0;
    assert!((2.9..=3.3).contains(&seconds), "took {seconds}s");
    assert_eq!(app.world().resource::<RunStats>().fixed, 1);
    assert!((TAPE_CAPACITY - tape_left(&app) - 3.0).abs() < 0.1);
}

#[test]
fn tape_on_a_bare_wall_is_wasted() {
    let mut app = test_app_with(Scenario::Quiet);
    tap(&mut app, KeyCode::Digit2);
    let x = RoomId::Hull.center().x;
    put_player(&mut app, Vec2::new(x, HALF_HEIGHT - 22.0));
    // Let the camera cut to the hull first: aiming is relative to the room.
    run_frames(&mut app, 2);
    aim_at(&mut app, Vec2::new(x, HALF_HEIGHT + 10.0));
    mouse_down(&mut app);
    run_frames(&mut app, secs(1.0));
    mouse_up(&mut app);
    frame(&mut app);
    assert!(
        (TAPE_CAPACITY - tape_left(&app) - 1.0).abs() < 0.05,
        "{}",
        tape_left(&app)
    );
    let mut strips = app.world_mut().query::<&TapeStrip>();
    assert!(strips.iter(app.world()).count() >= 5);
}

#[test]
fn tape_needs_a_wall() {
    let mut app = test_app_with(Scenario::Quiet);
    tap(&mut app, KeyCode::Digit2);
    mouse_down(&mut app);
    run_frames(&mut app, secs(1.0));
    assert_eq!(tape_left(&app), TAPE_CAPACITY);
}

#[test]
fn an_empty_roll_cannot_finish_a_breach() {
    let mut app = test_app_with(Scenario::TapeLow);
    mouse_down(&mut app);
    run_frames(&mut app, secs(3.0));
    assert_eq!(tape_left(&app), 0.0);
    let fault = faults(&mut app)[0];
    assert!((fault.repair - 1.0 / 3.0).abs() < 0.02, "{}", fault.repair);
}
