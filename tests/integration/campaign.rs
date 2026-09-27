//! The campaign: landing leads to the upgrade screen, a pick flies the next
//! level, the last level ends it, and PLAY starts over.

use bevy::prelude::*;
use leave_it_behind::faults::{Fault, Site, fault_bundle};
use leave_it_behind::level::campaign::level;
use leave_it_behind::level::{CurrentLevel, FaultPlan, MAX_FLIGHT_SECS};
use leave_it_behind::menu::Screen;
use leave_it_behind::upgrades::{Upgrade, Upgrades};
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    boot_to_menu, focus, frame, run_until, screen, secs, state, tap, test_app, test_app_with,
};

fn level_id(app: &App) -> String {
    app.world().resource::<CurrentLevel>().0.id.clone()
}

fn upgrades(app: &App) -> Upgrades {
    app.world().resource::<Upgrades>().clone()
}

fn key(app: &mut App, key: KeyCode) {
    tap(app, key);
    frame(app);
}

fn continue_log(app: &mut App) {
    assert!(matches!(screen(app), Some(Screen::Story(_))));
    key(app, KeyCode::Enter);
}

/// The `landing` scenario (every fault fixed, landing 3 s after launch)
/// flown to the end screen.
fn landed() -> App {
    let mut app = test_app_with(Scenario::Landing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    assert_eq!(screen(&app), Some(Screen::End));
    app
}

#[test]
fn landing_continues_to_the_upgrades_then_the_next_level() {
    let mut app = landed();
    // CONTINUE is highlighted.
    assert_eq!(focus(&app), 0);
    key(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), Some(Screen::Upgrade));
    assert_eq!(state(&app), AppState::Landed);
    // RUN FASTER is highlighted: Enter takes it and flies level two.
    key(&mut app, KeyCode::Enter);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    frame(&mut app);
    assert_eq!(level_id(&app), "two");
    assert_eq!(upgrades(&app).picks(), &[Upgrade::RunFaster]);
    let plan = app.world().resource::<FaultPlan>().pending.len();
    assert_eq!(plan, 7, "level two's faults, none started yet");
    assert_eq!(screen(&app), None, "the menu closes for the flight");
}

#[test]
fn number_keys_pick_an_upgrade() {
    let mut app = landed();
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Digit3);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(upgrades(&app).picks(), &[Upgrade::WiderTape]);
    assert_eq!(level_id(&app), "two");
}

#[test]
fn esc_on_the_upgrades_goes_back_to_the_score() {
    let mut app = landed();
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), Some(Screen::End));
    assert_eq!(state(&app), AppState::Landed);
    assert!(upgrades(&app).picks().is_empty());
}

#[test]
fn flying_a_level_again_and_continuing_never_stacks_extra_picks() {
    let mut app = landed();
    // Land level one, fly it again (R), land, and continue twice over.
    key(&mut app, KeyCode::KeyR);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Digit2);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(upgrades(&app).picks(), &[Upgrade::FasterWrench]);
    // Level two lands too (the scenario applies to every run): on to three.
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    key(&mut app, KeyCode::KeyR);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Digit1);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(level_id(&app), "three");
    assert_eq!(
        upgrades(&app).picks(),
        &[Upgrade::FasterWrench, Upgrade::RunFaster]
    );
}

#[test]
fn losing_flies_the_same_level_again_with_the_same_upgrades() {
    let mut app = landed();
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Digit1);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    frame(&mut app);
    // A drift about to hit a rock (drift keeps its own clock).
    let fault = app.world_mut().spawn(fault_bundle(Site::Helm, 55.0)).id();
    app.world_mut().get_mut::<Fault>(fault).unwrap().elapsed = 54.9;
    run_until(&mut app, secs(1.0), |app| state(app) == AppState::Lost);
    frame(&mut app);
    assert_eq!(screen(&app), Some(Screen::End));
    key(&mut app, KeyCode::KeyR);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(level_id(&app), "two");
    assert_eq!(upgrades(&app).picks(), &[Upgrade::RunFaster]);
}

#[test]
fn landing_the_last_level_ends_the_campaign() {
    let mut app = test_app_with(Scenario::FinalLanding);
    assert_eq!(level_id(&app), "five");
    assert_eq!(upgrades(&app).picks().len(), 4);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    assert_eq!(screen(&app), Some(Screen::End));
    // No CONTINUE: FLY AGAIN is first.
    key(&mut app, KeyCode::Enter);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(level_id(&app), "five");
}

#[test]
fn play_on_the_main_menu_starts_a_new_game() {
    let mut app = test_app();
    boot_to_menu(&mut app);
    key(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), Some(Screen::Main));
    // Left over from an earlier campaign.
    app.world_mut()
        .insert_resource(CurrentLevel(level(4).unwrap()));
    app.world_mut()
        .insert_resource(Upgrades::all_of(Upgrade::WiderTape, 3));
    key(&mut app, KeyCode::Enter);
    continue_log(&mut app);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    assert_eq!(level_id(&app), "one");
    assert!(upgrades(&app).picks().is_empty());
}

#[test]
fn every_level_lands_within_three_minutes_when_each_fault_is_fixed_within_twenty_seconds() {
    // Level-design check over the whole campaign, as for level one in
    // `level.rs`: repairs are applied directly. The flight lands the final
    // approach after the last fix.
    for number in 1..=5 {
        let def = level(number).unwrap();
        let mut app = test_app();
        app.insert_resource(CurrentLevel(def.clone()));
        crate::common::boot(&mut app);
        assert_eq!(level_id(&app), def.id);
        run_until(&mut app, secs(MAX_FLIGHT_SECS + 5.0), |app| {
            let mut q = app.world_mut().query::<&mut Fault>();
            for mut fault in q.iter_mut(app.world_mut()) {
                if fault.elapsed >= 20.0 {
                    fault.repair = 1.0;
                }
            }
            state(app) != AppState::Playing
        });
        assert_eq!(state(&app), AppState::Landed, "{}", def.id);
        let flown = app
            .world()
            .resource::<leave_it_behind::level::Journey>()
            .elapsed;
        assert!(flown <= MAX_FLIGHT_SECS + 3.5, "{} took {flown}s", def.id);
        let stats = app
            .world()
            .resource::<leave_it_behind::level::RunStats>()
            .clone();
        let slots = def.slots.len() as u32;
        assert_eq!(
            (stats.started, stats.fixed, stats.failure),
            (slots, slots, None),
            "{}",
            def.id
        );
    }
}
