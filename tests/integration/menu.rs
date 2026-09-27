use bevy::prelude::*;
use bevy::window::WindowFocused;
use leave_it_behind::faults::Fault;
use leave_it_behind::faults::drift::Nav;
use leave_it_behind::level::Journey;
use leave_it_behind::menu::{Confirm, Menu, Screen};
use leave_it_behind::player::Player;
use leave_it_behind::settings::Settings;
use leave_it_behind::ui::Hud;
use leave_it_behind::{AppState, Scenario};

use crate::common::{
    boot_to_menu, focus, frame, paused, press, release, run_frames, run_until, screen, secs, state,
    tap, test_app, test_app_with,
};

fn elapsed(app: &App) -> f32 {
    app.world().resource::<Journey>().elapsed
}

fn players(app: &mut App) -> usize {
    let mut q = app.world_mut().query::<&Player>();
    q.iter(app.world()).count()
}

/// Taps `key`, then runs one more frame so the menu action applies.
fn key(app: &mut App, key: KeyCode) {
    tap(app, key);
    frame(app);
}

/// Moves the highlight down `n` rows and presses Enter.
fn choose(app: &mut App, n: usize) {
    for _ in 0..n {
        key(app, KeyCode::ArrowDown);
    }
    key(app, KeyCode::Enter);
}

fn main_menu() -> App {
    let mut app = test_app();
    boot_to_menu(&mut app);
    key(&mut app, KeyCode::Space);
    assert_eq!(screen(&app), Some(Screen::Main));
    app
}

#[test]
fn a_player_boots_to_the_title_then_the_main_menu() {
    let mut app = test_app();
    boot_to_menu(&mut app);
    assert_eq!(screen(&app), Some(Screen::Title));
    assert_eq!(players(&mut app), 0);
    key(&mut app, KeyCode::KeyQ);
    assert_eq!(screen(&app), Some(Screen::Main));
    assert_eq!(focus(&app), 0);
}

#[test]
fn play_starts_level_one_and_shows_the_hud() {
    let mut app = main_menu();
    let mut huds = app.world_mut().query_filtered::<&Visibility, With<Hud>>();
    assert!(huds.iter(app.world()).all(|v| *v == Visibility::Hidden));
    key(&mut app, KeyCode::Enter);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    frame(&mut app);
    assert_eq!(screen(&app), None);
    assert_eq!(players(&mut app), 1);
    assert!(elapsed(&app) < 0.1);
    let mut huds = app.world_mut().query_filtered::<&Visibility, With<Hud>>();
    assert!(huds.iter(app.world()).all(|v| *v == Visibility::Inherited));
}

#[test]
fn the_keyboard_moves_the_highlight_and_wraps() {
    let mut app = main_menu();
    key(&mut app, KeyCode::ArrowDown);
    assert_eq!(focus(&app), 1);
    key(&mut app, KeyCode::KeyW);
    key(&mut app, KeyCode::KeyW);
    // PLAY, HOW TO PLAY, SETTINGS, QUIT: up from the top wraps to QUIT.
    assert_eq!(focus(&app), 3);
    key(&mut app, KeyCode::KeyS);
    assert_eq!(focus(&app), 0);
}

#[test]
fn keys_pressed_in_the_same_frame_all_count() {
    // Slow frames bunch keys up: Down then Enter still opens the second row.
    let mut app = main_menu();
    press(&mut app, KeyCode::ArrowDown);
    press(&mut app, KeyCode::Enter);
    frame(&mut app);
    assert_eq!(screen(&app), Some(Screen::HowToPlay));
}

#[test]
fn how_to_play_and_back_keep_the_highlight() {
    let mut app = main_menu();
    choose(&mut app, 1);
    assert_eq!(screen(&app), Some(Screen::HowToPlay));
    key(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), Some(Screen::Main));
    assert_eq!(focus(&app), 1);
}

#[test]
fn quit_on_the_main_menu_exits() {
    let mut app = main_menu();
    choose(&mut app, 3);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}

#[test]
fn esc_pauses_the_flight_and_everything_stops() {
    let mut app = test_app_with(Scenario::Bolts);
    run_frames(&mut app, 30);
    key(&mut app, KeyCode::Escape);
    assert!(paused(&app));
    assert_eq!(screen(&app), Some(Screen::Pause));
    let journey = elapsed(&app);
    let clock = |app: &mut App| {
        let mut q = app.world_mut().query::<&Fault>();
        q.single(app.world()).unwrap().elapsed
    };
    let fault = clock(&mut app);
    // Walking is ignored too.
    press(&mut app, KeyCode::KeyD);
    run_frames(&mut app, secs(2.0));
    assert_eq!(elapsed(&app), journey);
    assert_eq!(clock(&mut app), fault);

    key(&mut app, KeyCode::Escape);
    assert!(!paused(&app));
    assert_eq!(screen(&app), None);
    run_frames(&mut app, 30);
    assert!(elapsed(&app) > journey + 0.4);
}

#[test]
fn p_pauses_and_resumes_too() {
    let mut app = test_app_with(Scenario::Quiet);
    key(&mut app, KeyCode::KeyP);
    assert!(paused(&app));
    key(&mut app, KeyCode::KeyP);
    assert!(!paused(&app));
    assert_eq!(screen(&app), None);
}

fn lose_focus(app: &mut App) {
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    app.world_mut().write_message(WindowFocused {
        window,
        focused: false,
    });
    frame(app);
    frame(app);
}

#[test]
fn losing_focus_pauses_unless_turned_off() {
    let mut app = test_app_with(Scenario::Quiet);
    lose_focus(&mut app);
    assert!(paused(&app));

    let mut app = test_app_with(Scenario::Quiet);
    app.world_mut().resource_mut::<Settings>().pause_unfocused = false;
    lose_focus(&mut app);
    assert!(!paused(&app));
}

#[test]
fn restart_asks_first_then_starts_a_fresh_unpaused_flight() {
    let mut app = test_app_with(Scenario::Default);
    run_frames(&mut app, secs(3.0));
    key(&mut app, KeyCode::Escape);
    choose(&mut app, 1);
    assert_eq!(screen(&app), Some(Screen::Confirm(Confirm::Restart)));
    // Enter on a confirm screen goes back: the safe choice comes first.
    key(&mut app, KeyCode::Enter);
    assert_eq!(screen(&app), Some(Screen::Pause));
    assert_eq!(focus(&app), 1);
    key(&mut app, KeyCode::Enter);
    choose(&mut app, 1);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Playing);
    assert!(!paused(&app));
    assert_eq!(screen(&app), None);
    assert!(elapsed(&app) < 0.1);
    assert_eq!(players(&mut app), 1);
}

#[test]
fn main_menu_from_pause_leaves_the_flight() {
    let mut app = test_app_with(Scenario::Scramble);
    key(&mut app, KeyCode::Escape);
    choose(&mut app, 4);
    assert_eq!(screen(&app), Some(Screen::Confirm(Confirm::MainMenu)));
    choose(&mut app, 1);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Menu);
    // The title card is once per launch, before the first flight.
    assert_eq!(screen(&app), Some(Screen::Main));
    assert_eq!(players(&mut app), 0);
    let mut faults = app.world_mut().query::<&Fault>();
    assert_eq!(faults.iter(app.world()).count(), 0);
}

#[test]
fn quitting_mid_flight_asks_first() {
    let mut app = test_app_with(Scenario::Quiet);
    key(&mut app, KeyCode::Escape);
    choose(&mut app, 5);
    assert_eq!(screen(&app), Some(Screen::Confirm(Confirm::Quit)));
    assert_eq!(app.should_exit(), None);
    choose(&mut app, 1);
    assert_eq!(app.should_exit(), Some(AppExit::Success));
}

#[test]
fn resuming_drops_input_from_before_the_pause() {
    // E and Esc in the same frame: the E must not take the helm on resume.
    let mut app = test_app_with(Scenario::Drift);
    press(&mut app, KeyCode::KeyE);
    key(&mut app, KeyCode::Escape);
    assert!(paused(&app));
    key(&mut app, KeyCode::Escape);
    run_frames(&mut app, 5);
    assert!(!app.world().resource::<Nav>().engaged);
    // A fresh E still works.
    release(&mut app, KeyCode::KeyE);
    tap(&mut app, KeyCode::KeyE);
    frame(&mut app);
    assert!(app.world().resource::<Nav>().engaged);
}

#[test]
fn the_end_screen_flies_again_or_goes_to_the_main_menu() {
    let mut app = test_app_with(Scenario::Landing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    assert_eq!(screen(&app), Some(Screen::End));
    // CONTINUE is highlighted (see `campaign.rs`); FLY AGAIN is next, like R.
    key(&mut app, KeyCode::ArrowDown);
    key(&mut app, KeyCode::Enter);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    key(&mut app, KeyCode::Escape);
    frame(&mut app);
    assert_eq!(state(&app), AppState::Menu);
    assert_eq!(screen(&app), Some(Screen::Main));
}

#[test]
fn settings_change_with_the_arrow_keys() {
    let mut app = main_menu();
    choose(&mut app, 2);
    assert_eq!(screen(&app), Some(Screen::Settings));
    let shake = |app: &App| app.world().resource::<Settings>().shake;
    key(&mut app, KeyCode::ArrowLeft);
    assert_eq!(shake(&app), 75);
    key(&mut app, KeyCode::KeyA);
    assert_eq!(shake(&app), 50);
    key(&mut app, KeyCode::ArrowRight);
    assert_eq!(shake(&app), 75);
    // Enter cycles and wraps: 75 -> 100 -> 0.
    key(&mut app, KeyCode::Enter);
    key(&mut app, KeyCode::Enter);
    assert_eq!(shake(&app), 0);
    // Controls hint is the third row: a toggle.
    key(&mut app, KeyCode::ArrowDown);
    key(&mut app, KeyCode::ArrowDown);
    key(&mut app, KeyCode::Enter);
    assert!(!app.world().resource::<Settings>().controls_hint);
    // RESET SETTINGS (below TIPS, PAUSE WHEN UNFOCUSED, FULLSCREEN and
    // VSYNC) puts everything back.
    for _ in 0..5 {
        key(&mut app, KeyCode::ArrowDown);
    }
    key(&mut app, KeyCode::Enter);
    assert_eq!(*app.world().resource::<Settings>(), Settings::default());
    key(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), Some(Screen::Main));
}

#[test]
fn the_settings_scenario_stacks_settings_over_the_pause_menu() {
    let mut app = test_app_with(Scenario::Settings);
    frame(&mut app);
    assert!(paused(&app));
    assert_eq!(screen(&app), Some(Screen::Settings));
    assert_eq!(app.world().resource::<Menu>().stack.len(), 2);
    key(&mut app, KeyCode::Escape);
    assert_eq!(screen(&app), Some(Screen::Pause));
    key(&mut app, KeyCode::Escape);
    assert!(!paused(&app));
}
