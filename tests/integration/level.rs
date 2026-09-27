use bevy::prelude::*;
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
fn a_flight_starts_with_every_fault_still_to_fix() {
    let mut app = test_app();
    boot(&mut app);
    let journey = *app.world().resource::<Journey>();
    assert!(journey.elapsed < 0.1 && !journey.cleared());
    let plan = app.world().resource::<FaultPlan>().clone();
    let stats = app.world().resource::<RunStats>().clone();
    assert_eq!(plan.total(&stats), 4);
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
fn level_one_teaches_each_fix_in_the_same_order_every_run() {
    // The first three faults are pinned whatever the session seed; the
    // fourth is random.
    let firsts = |seed: u64| {
        let mut app = test_app();
        app.insert_resource(leave_it_behind::GameRng::from_seed(seed));
        boot(&mut app);
        let mut started = Vec::new();
        run_until(&mut app, secs(70.0), |app| {
            let mut q = app.world_mut().query::<&mut Fault>();
            for mut fault in q.iter_mut(app.world_mut()) {
                if !started.contains(&fault.site) {
                    started.push(fault.site);
                }
                if fault.elapsed >= 20.0 {
                    fault.repair = 1.0;
                }
            }
            elapsed(app) >= 60.0
        });
        started
    };
    let expected = vec![Site::PortEngineInner, Site::EngineRoomPort, Site::Helm];
    assert_eq!(firsts(1), expected);
    assert_eq!(firsts(12345), expected);
}

#[test]
fn fixing_every_fault_lands_the_ship_after_the_final_approach() {
    let mut app = test_app_with(Scenario::Landing);
    assert_eq!(state(&app), AppState::Playing);
    let frames = run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    assert!((170..=190).contains(&frames), "{frames} frames");
}

#[test]
fn a_flight_never_lands_with_a_fault_left() {
    // Level one's schedule is done by 1:30; with the last fault left
    // unfixed (and its clock stretched) the ship flies on well past that.
    let mut app = test_app();
    boot(&mut app);
    let mut last = None;
    run_until(&mut app, secs(150.0), |app| {
        let pending = app.world().resource::<FaultPlan>().pending.len();
        let mut q = app.world_mut().query::<&mut Fault>();
        for mut fault in q.iter_mut(app.world_mut()) {
            if pending > 0 && fault.elapsed >= 20.0 {
                fault.repair = 1.0;
            } else if pending == 0 {
                last.get_or_insert(fault.site);
                // Keep it from failing: this test is about landing.
                fault.elapsed = 0.0;
                fault.clock = 1e6;
            }
        }
        state(app) != AppState::Playing
    });
    assert!(last.is_some());
    assert_eq!(state(&app), AppState::Playing);
    assert!(!app.world().resource::<Journey>().cleared());
}

#[test]
fn ignoring_every_fault_loses_to_the_first() {
    let mut app = test_app();
    boot(&mut app);
    run_until(&mut app, secs(85.0), |app| state(app) == AppState::Lost);
    // Loose bolts at 0:10 with a 70 s clock.
    assert!((elapsed(&app) - 80.0).abs() < 0.1, "{}", elapsed(&app));
    let failure = app.world().resource::<RunStats>().failure.unwrap();
    assert_eq!(failure.kind(), FaultKind::LooseBolts);
}

#[test]
fn an_engineer_who_fixes_everything_within_twenty_seconds_lands() {
    // Level-design check: with every fault fixed 20 s after it starts (the
    // envelope's response time), the flight lands. Repairs are applied
    // directly; the controls have their own tests. (Every level: `campaign.rs`.)
    let mut app = test_app();
    boot(&mut app);
    run_until(&mut app, secs(160.0), |app| {
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
    // Each of the four faults ran its 20 s: one of each kind, and one more.
    let d = stats.damage;
    assert!((d.total() - 80.0).abs() < 1.0, "{d:?}");
    assert!(
        d.oxygen >= 19.0 && d.course >= 19.0 && d.engine >= 19.0,
        "{d:?}"
    );
}

#[test]
fn r_starts_a_fresh_run() {
    let mut app = test_app();
    boot(&mut app);
    // Mess up the run: move, use tape, let a fault start, then land.
    put_player(&mut app, layout::helm_seat());
    app.world_mut().resource_mut::<ToolBelt>().tape_left = 3.0;
    run_until(&mut app, secs(12.0), |app| elapsed(app) >= 11.0);
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Landed);
    run_until(&mut app, 5, |app| state(app) == AppState::Landed);

    tap(&mut app, KeyCode::KeyR);
    tap(&mut app, KeyCode::Enter);
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
