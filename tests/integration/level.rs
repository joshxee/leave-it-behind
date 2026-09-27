use bevy::prelude::*;
use leave_it_behind::faults::drift::Nav;
use leave_it_behind::faults::{Fault, FaultKind, Site};
use leave_it_behind::level::{CurrentLevel, FaultPlan, Journey, RunStats};
use leave_it_behind::player::Player;
use leave_it_behind::ship::layout;
use leave_it_behind::tools::{TAPE_CAPACITY, ToolBelt};
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    boot, frame, player_pos, put_player, run_frames, run_until, secs, state, tap, test_app,
    test_app_with,
};

fn active_sites(app: &mut App) -> Vec<Site> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).map(|f| f.site).collect()
}

fn elapsed(app: &App) -> f32 {
    app.world().resource::<Journey>().elapsed
}

#[test]
fn the_countdown_starts_at_four_and_a_half_minutes() {
    let mut app = test_app();
    boot(&mut app);
    let journey = *app.world().resource::<Journey>();
    assert_eq!(journey.duration, 270.0);
    assert!(journey.remaining() > 269.9);
}

#[test]
fn level_one_starts_its_first_fault_on_schedule() {
    let mut app = test_app();
    boot(&mut app);
    run_until(&mut app, secs(11.0), |app| elapsed(app) >= 9.9);
    assert!(active_sites(&mut app).is_empty());
    run_until(&mut app, secs(1.0), |app| elapsed(app) >= 10.1);
    assert_eq!(active_sites(&mut app), vec![Site::PortEngineInner]);
    assert_eq!(app.world().resource::<RunStats>().started, 1);
}

#[test]
fn level_one_plays_out_identically_every_run() {
    // Through the first drift (it starts at 1:06 with a seeded heading).
    let snapshot = |app: &mut App| {
        run_until(app, secs(70.0), |app| {
            // Fix the earlier faults so the run survives to the drift.
            let mut q = app.world_mut().query::<&mut Fault>();
            for mut fault in q.iter_mut(app.world_mut()) {
                if fault.elapsed >= 20.0 {
                    fault.repair = 1.0;
                }
            }
            elapsed(app) >= 68.0
        });
        let nav = *app.world().resource::<Nav>();
        (active_sites(app), nav.marker, nav.heading)
    };
    let mut first = test_app();
    boot(&mut first);
    let a = snapshot(&mut first);
    let mut second = test_app();
    // A different session seed must not matter: level one reseeds.
    second.insert_resource(leave_it_behind::GameRng::from_seed(12345));
    boot(&mut second);
    let b = snapshot(&mut second);
    assert_eq!(a, b);
    assert!(a.0.contains(&Site::Helm));
}

#[test]
fn reaching_zero_lands_the_ship() {
    let mut app = test_app_with(Scenario::Landing);
    assert_eq!(state(&app), AppState::Playing);
    let frames = run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    assert!((170..=190).contains(&frames), "{frames} frames");
}

#[test]
fn ignoring_every_fault_loses_to_the_first() {
    let mut app = test_app();
    boot(&mut app);
    run_until(&mut app, secs(95.0), |app| state(app) == AppState::Lost);
    // Loose bolts at 0:10 with an 80 s clock.
    assert!((elapsed(&app) - 90.0).abs() < 0.1, "{}", elapsed(&app));
    let failure = app.world().resource::<RunStats>().failure.unwrap();
    assert_eq!(failure.kind(), FaultKind::LooseBolts);
}

#[test]
fn an_engineer_who_fixes_everything_within_twenty_seconds_lands() {
    // Level-design check: with every fault fixed 20 s after it starts (the
    // envelope's response time), the flight lands. Repairs are applied
    // directly; the controls have their own tests.
    let mut app = test_app();
    boot(&mut app);
    run_until(&mut app, secs(280.0), |app| {
        let mut q = app.world_mut().query::<&mut Fault>();
        for mut fault in q.iter_mut(app.world_mut()) {
            if fault.elapsed >= 20.0 {
                fault.repair = 1.0;
            }
        }
        state(app) != AppState::Playing
    });
    assert_eq!(state(&app), AppState::Landed);
    let stats = app.world().resource::<RunStats>().clone();
    let slots = app.world().resource::<CurrentLevel>().0.slots.len() as u32;
    assert_eq!(
        (stats.started, stats.fixed, stats.failure),
        (slots, slots, None)
    );
    // The 4 breaches and 3 loose panels each ran their 20 s. A drift can
    // wander back into the centre band and settle sooner, so its 3 ran at
    // most 20 s each.
    let d = stats.damage;
    let near = |value: f32, expected: f32| (value - expected).abs() < 1.0;
    assert!(near(d.oxygen, 80.0) && near(d.engine, 60.0), "{d:?}");
    assert!(d.course > 0.0 && d.course < 60.5, "{d:?}");
}

#[test]
fn r_starts_a_fresh_run() {
    let mut app = test_app();
    boot(&mut app);
    // Mess up the run: move, use tape, let a fault start, then land.
    put_player(&mut app, layout::helm_seat());
    app.world_mut().resource_mut::<ToolBelt>().tape_left = 3.0;
    run_until(&mut app, secs(12.0), |app| elapsed(app) >= 11.0);
    app.world_mut().resource_mut::<Journey>().elapsed = 270.0;
    run_until(&mut app, 5, |app| state(app) == AppState::Landed);

    tap(&mut app, KeyCode::KeyR);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    frame(&mut app);
    assert!(elapsed(&app) < 0.1);
    assert_eq!(player_pos(&mut app), layout::player_spawn());
    assert_eq!(app.world().resource::<ToolBelt>().tape_left, TAPE_CAPACITY);
    assert!(active_sites(&mut app).is_empty());
    let pending = app.world().resource::<FaultPlan>().pending.len();
    assert_eq!(
        pending,
        app.world().resource::<CurrentLevel>().0.slots.len()
    );
    let mut players = app.world_mut().query::<&Player>();
    assert_eq!(players.iter(app.world()).count(), 1);
    assert_eq!(*app.world().resource::<RunStats>(), RunStats::default());
}

#[test]
fn r_does_nothing_mid_flight() {
    let mut app = test_app();
    boot(&mut app);
    run_frames(&mut app, secs(2.0));
    tap(&mut app, KeyCode::KeyR);
    frame(&mut app);
    assert!(elapsed(&app) > 1.9);
}
