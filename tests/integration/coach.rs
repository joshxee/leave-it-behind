use bevy::prelude::*;
use leave_it_behind::coach::{Coach, CoachPanel, CoachText, LAUNCH_NOTE, Tip, TipsSeen};
use leave_it_behind::diagnostics::{DiagView, Diagnostics};
use leave_it_behind::faults::bolts::Bolt;
use leave_it_behind::faults::{Fault, FaultKind, Site};
use leave_it_behind::level::Journey;
use leave_it_behind::save::Storage;
use leave_it_behind::scenarios::console_stand;
use leave_it_behind::settings::Settings;
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    aim_at, boot_to_menu, click, focus, frame, key_toward, player_pos, press, put_player, release,
    run_frames, run_until, secs, state, tap, test_app, test_app_with, test_app_with_storage,
};

/// The coaching panel's lines, or none while it is hidden.
fn panel(app: &mut App) -> Vec<String> {
    let mut panels = app
        .world_mut()
        .query_filtered::<&Visibility, With<CoachPanel>>();
    if *panels.single(app.world()).unwrap() == Visibility::Hidden {
        return Vec::new();
    }
    let mut texts = app.world_mut().query_filtered::<&Text, With<CoachText>>();
    let text = &texts.single(app.world()).unwrap().0;
    text.lines().map(str::to_string).collect()
}

fn launched(app: &App) -> bool {
    app.world().resource::<Journey>().launched
}

fn elapsed(app: &App) -> f32 {
    app.world().resource::<Journey>().elapsed
}

fn seen(app: &App) -> TipsSeen {
    app.world().resource::<Settings>().tips_seen
}

fn view(app: &App) -> DiagView {
    app.world().resource::<Diagnostics>().view
}

fn kinds(app: &mut App) -> Vec<FaultKind> {
    let mut q = app.world_mut().query::<&Fault>();
    q.iter(app.world()).map(|f| f.kind()).collect()
}

fn loose_bolts(app: &mut App) -> Vec<Vec2> {
    let mut q = app.world_mut().query::<&Bolt>();
    q.iter(app.world())
        .filter(|b| b.loose)
        .map(|b| b.pos)
        .collect()
}

/// Taps `key`, then runs one more frame so a menu action applies.
fn key(app: &mut App, key: KeyCode) {
    tap(app, key);
    frame(app);
}

/// Highlights `row` of the top menu screen with the arrow keys, then Enter.
fn pick(app: &mut App, row: usize) {
    while focus(app) < row {
        key(app, KeyCode::ArrowDown);
    }
    while focus(app) > row {
        key(app, KeyCode::ArrowUp);
    }
    key(app, KeyCode::Enter);
}

/// From the title, as a player boots: the main menu, then PLAY.
fn play_from_the_title(app: &mut App) {
    boot_to_menu(app);
    key(app, KeyCode::Space);
    pick(app, 0);
    run_until(app, 3, |app| state(app) == AppState::Playing);
    frame(app);
}

/// Steps up to the console (setup), then looks at the diagnostic screen.
fn do_the_preflight_check(app: &mut App) {
    put_player(app, console_stand());
    frame(app);
    tap(app, KeyCode::KeyE);
    run_until(app, secs(2.0), |app| launched(app));
}

#[test]
fn a_first_flight_waits_for_the_preflight_check() {
    let mut app = test_app_with(Scenario::FirstFlight);
    assert!(app.world().resource::<Coach>().active);
    // Level one's first fault is due at 0:10: nothing starts while it waits.
    run_frames(&mut app, secs(12.0));
    assert!(!launched(&app));
    assert_eq!(elapsed(&app), 0.0);
    assert!(kinds(&mut app).is_empty());
    assert_eq!(panel(&mut app), [Tip::Preflight.text()]);
}

#[test]
fn looking_at_the_diagnostic_screen_launches_the_flight() {
    let mut app = test_app_with(Scenario::FirstFlight);
    put_player(&mut app, console_stand());
    frame(&mut app);
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert!(matches!(view(&app), DiagView::Scanning { .. }));
    assert!(!launched(&app), "starting a scan is not looking at it");
    run_until(&mut app, secs(2.0), |app| launched(app));
    assert_eq!(view(&app), DiagView::Open);
    assert!(seen(&app).has(Tip::Preflight));
    frame(&mut app);
    assert_eq!(panel(&mut app), [LAUNCH_NOTE]);
    // A normal flight from here: the countdown runs, and the first fault
    // comes ten seconds after launch with its tip.
    run_until(&mut app, secs(11.0), |app| !kinds(app).is_empty());
    assert!((elapsed(&app) - 10.0).abs() < 0.1, "{}", elapsed(&app));
    assert_eq!(kinds(&mut app), [FaultKind::LooseBolts]);
    assert_eq!(panel(&mut app), [Tip::LooseBolts.text()]);
}

#[test]
fn walking_away_mid_scan_keeps_the_flight_waiting() {
    let mut app = test_app_with(Scenario::FirstFlight);
    put_player(&mut app, console_stand());
    frame(&mut app);
    tap(&mut app, KeyCode::KeyE);
    press(&mut app, KeyCode::KeyS);
    run_frames(&mut app, secs(2.0));
    release(&mut app, KeyCode::KeyS);
    assert_eq!(view(&app), DiagView::Closed);
    assert!(!launched(&app));
    assert!(!seen(&app).has(Tip::Preflight));
    assert_eq!(panel(&mut app), [Tip::Preflight.text()]);
}

#[test]
fn each_kind_of_fault_gets_a_tip_the_first_time_only() {
    // Level-design check over level one: every fault is fixed 15 s after it
    // starts (directly, as in level.rs). Records the panel as each starts.
    let mut app = test_app_with(Scenario::FirstFlight);
    do_the_preflight_check(&mut app);
    let mut known: Vec<Entity> = Vec::new();
    let mut starts: Vec<(f32, FaultKind, Vec<String>)> = Vec::new();
    run_until(&mut app, secs(200.0), |app| {
        let mut new = Vec::new();
        let mut q = app.world_mut().query::<(Entity, &mut Fault)>();
        for (entity, mut fault) in q.iter_mut(app.world_mut()) {
            if !known.contains(&entity) {
                known.push(entity);
                new.push(fault.kind());
            }
            if fault.elapsed >= 15.0 {
                fault.repair = 1.0;
            }
        }
        for kind in new {
            let lines = panel(app);
            starts.push((elapsed(app), kind, lines));
        }
        elapsed(app) >= 190.0
    });
    assert_eq!(state(&app), AppState::Playing);
    assert_eq!(starts.len(), 10, "{starts:?}");
    let tip = |kind| vec![Tip::for_fault(kind).text().to_string()];
    for (at, kind, lines) in &starts[..3] {
        assert_eq!(*lines, tip(*kind), "first {kind:?} at {at}");
    }
    for (at, kind, lines) in &starts[3..] {
        assert!(lines.is_empty(), "{kind:?} at {at}: {lines:?}");
    }
    assert_eq!(seen(&app), TipsSeen::EVERY);
    assert!(!app.world().resource::<Settings>().tips_on());
}

#[test]
fn the_coaching_is_never_repeated() {
    let mut app = test_app();
    play_from_the_title(&mut app);
    assert!(
        !launched(&app),
        "a player's first flight waits for the check"
    );
    do_the_preflight_check(&mut app);
    run_until(&mut app, secs(12.0), |app| !kinds(app).is_empty());
    frame(&mut app);
    assert_eq!(panel(&mut app), [Tip::LooseBolts.text()]);

    // Restart with the tip up: it was shown, so it is done too.
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 1);
    pick(&mut app, 1);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Playing);
    assert!(launched(&app), "no pre-flight check the second time");
    let done = TipsSeen {
        preflight: true,
        loose_bolts: true,
        ..default()
    };
    assert_eq!(seen(&app), done);
    run_until(&mut app, secs(12.0), |app| !kinds(app).is_empty());
    frame(&mut app);
    assert!(panel(&mut app).is_empty());

    // What was seen is saved: the next launch of the game remembers it.
    let storage = app.world_mut().remove_resource::<Storage>().unwrap();
    let mut app = test_app_with_storage(storage);
    play_from_the_title(&mut app);
    assert!(launched(&app));
    assert_eq!(seen(&app), done);
    assert!(panel(&mut app).is_empty());
}

#[test]
fn tips_off_skips_the_coaching_and_on_brings_it_back() {
    let mut app = test_app();
    boot_to_menu(&mut app);
    key(&mut app, KeyCode::Space);
    pick(&mut app, 2);
    // SHAKE, FLASH, CONTROLS HINT, TIPS.
    pick(&mut app, 3);
    assert_eq!(seen(&app), TipsSeen::EVERY);
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 0);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    frame(&mut app);
    assert!(launched(&app));
    run_until(&mut app, secs(11.0), |app| !kinds(app).is_empty());
    frame(&mut app);
    assert!(panel(&mut app).is_empty());

    // Back on from the pause menu: the tip for the fault on board shows...
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 3);
    pick(&mut app, 3);
    assert_eq!(seen(&app), TipsSeen::default());
    key(&mut app, KeyCode::Escape);
    key(&mut app, KeyCode::Escape);
    frame(&mut app);
    assert_eq!(panel(&mut app), [Tip::LooseBolts.text()]);
    assert!(launched(&app), "a flight under way never waits again");
    // ...and the next flight starts with the pre-flight check.
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 1);
    pick(&mut app, 1);
    frame(&mut app);
    assert!(!launched(&app));
    assert_eq!(panel(&mut app), [Tip::Preflight.text()]);
}

#[test]
fn switching_tips_off_during_the_preflight_check_launches_the_flight() {
    let mut app = test_app_with(Scenario::FirstFlight);
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 3);
    pick(&mut app, 3);
    key(&mut app, KeyCode::Escape);
    key(&mut app, KeyCode::Escape);
    frame(&mut app);
    assert!(launched(&app));
    assert!(panel(&mut app).is_empty());
    let before = elapsed(&app);
    run_frames(&mut app, secs(1.0));
    assert!(elapsed(&app) > before + 0.9, "the countdown runs");
}

#[test]
fn the_first_bolts_tip_stays_until_they_are_fixed() {
    let mut app = test_app_with(Scenario::FirstBolts);
    frame(&mut app);
    assert!(launched(&app), "the pre-flight check is done");
    assert_eq!(panel(&mut app), [Tip::LooseBolts.text()]);
    // Walk the panel turning each bolt, as a player does.
    let along = Site::PortEngineInner.normal().perp();
    for bolt in Site::PortEngineInner.bolts().unwrap() {
        let off = |app: &mut App| (bolt - player_pos(app)).dot(along);
        if off(&mut app).abs() > 2.0 {
            let key = key_toward(along * off(&mut app));
            press(&mut app, key);
            run_until(&mut app, secs(2.0), |app| off(app).abs() <= 4.0);
            release(&mut app, key);
        }
        aim_at(&mut app, bolt);
        frame(&mut app);
        click(&mut app);
        run_until(&mut app, secs(2.0), |app| !loose_bolts(app).contains(&bolt));
        if !loose_bolts(&mut app).is_empty() {
            assert_eq!(panel(&mut app), [Tip::LooseBolts.text()]);
        }
    }
    run_until(&mut app, 10, |app| kinds(app).is_empty());
    assert!(panel(&mut app).is_empty());
    assert!(seen(&app).has(Tip::LooseBolts));
}

#[test]
fn leaving_a_flight_with_a_tip_up_counts_it_as_seen() {
    let mut app = test_app_with(Scenario::FirstBolts);
    frame(&mut app);
    assert!(!seen(&app).has(Tip::LooseBolts));
    key(&mut app, KeyCode::Escape);
    pick(&mut app, 4);
    pick(&mut app, 1);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Menu);
    assert!(seen(&app).has(Tip::LooseBolts));
    assert!(panel(&mut app).is_empty());
}

#[test]
fn other_scenarios_skip_the_coaching() {
    for scenario in [Scenario::Default, Scenario::Bolts] {
        let mut app = test_app_with(scenario);
        run_frames(&mut app, secs(11.0));
        let name = scenario.name();
        assert!(!app.world().resource::<Coach>().active, "{name}");
        assert!(launched(&app), "{name}");
        assert!(!kinds(&mut app).is_empty(), "{name}: a fault is up");
        assert!(panel(&mut app).is_empty(), "{name}");
        assert_eq!(seen(&app), TipsSeen::default(), "{name}");
    }
}
