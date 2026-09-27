//! Each upgrade changes play through the real controls: walking, turning a
//! bolt, taping a breach.

use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::faults::bolts::Bolt;
use leave_it_behind::faults::{Fault, Site};
use leave_it_behind::player::PLAYER_SPEED;
use leave_it_behind::scenarios::breach_stand;
use leave_it_behind::tools::{TapeStrip, WRENCH_TURN_SECS};
use leave_it_behind::upgrades::{Upgrade, Upgrades};

use crate::common::{
    aim_at, click, frame, mouse_down, mouse_up, player_pos, press, put_player, run_frames,
    run_until, secs, test_app_with,
};

fn with_upgrade(scenario: Scenario, upgrade: Upgrade, picks: usize) -> App {
    let mut app = test_app_with(scenario);
    app.insert_resource(Upgrades::all_of(upgrade, picks));
    app
}

fn loose_bolts(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Bolt>();
    q.iter(app.world()).filter(|b| b.loose).count()
}

fn breach_repair(app: &mut App) -> Option<f32> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).next().map(|f| f.repair)
}

#[test]
fn running_faster_covers_more_ground() {
    let mut app = with_upgrade(Scenario::Quiet, Upgrade::RunFaster, 2);
    let start = player_pos(&mut app);
    press(&mut app, KeyCode::KeyD);
    run_frames(&mut app, 30);
    let moved = (player_pos(&mut app) - start).x;
    let speed = PLAYER_SPEED * 1.3;
    assert!(
        (moved - speed / 2.0).abs() <= speed / 60.0 + 0.05,
        "moved {moved}"
    );
}

#[test]
fn a_faster_wrench_turns_a_bolt_sooner() {
    let mut app = with_upgrade(Scenario::Bolts, Upgrade::FasterWrench, 2);
    let turn = WRENCH_TURN_SECS * 0.75 * 0.75;
    frame(&mut app);
    click(&mut app);
    run_frames(&mut app, secs(turn) - 3);
    assert_eq!(loose_bolts(&mut app), 3, "not before its time");
    run_frames(&mut app, 6);
    assert_eq!(loose_bolts(&mut app), 2);
    assert!(secs(turn) + 3 < secs(WRENCH_TURN_SECS));
}

#[test]
fn wider_tape_seals_faster_in_wider_strips() {
    let mut app = with_upgrade(Scenario::Breach, Upgrade::WiderTape, 2);
    mouse_down(&mut app);
    let frames = run_until(&mut app, secs(4.0), |app| breach_repair(app).is_none());
    mouse_up(&mut app);
    // Three seconds of tape at twice the rate.
    let seconds = frames as f32 / 60.0;
    assert!((1.4..=1.7).contains(&seconds), "took {seconds}s");
    let mut strips = app.world_mut().query::<&TapeStrip>();
    let n = strips.iter(app.world()).count();
    // Three strips side by side every 0.15 s.
    assert!(n >= 3 * 9, "{n} strips");
}

#[test]
fn wider_tape_reaches_a_hole_the_plain_roll_misses() {
    let site = Site::AirlockPortAft;
    // Along the wall, 32 units off the hole: past the plain roll's 26.
    let stand = breach_stand() + site.normal().perp() * 32.0;
    let taped = |picks: usize| {
        let mut app = with_upgrade(Scenario::Breach, Upgrade::WiderTape, picks);
        put_player(&mut app, stand);
        frame(&mut app);
        aim_at(&mut app, stand - site.normal() * 60.0);
        run_frames(&mut app, 2);
        mouse_down(&mut app);
        run_frames(&mut app, secs(0.5));
        breach_repair(&mut app).unwrap_or(1.0)
    };
    assert_eq!(taped(0), 0.0);
    assert!(taped(1) > 0.1);
}
