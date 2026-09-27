use bevy::prelude::*;
use leave_it_behind::level::{LastRun, Progress, RunStats};
use leave_it_behind::save::{SAVE_KEY, SETTINGS_KEY, Storage};
use leave_it_behind::settings::Settings;
use leave_it_behind::ui::Notices;
use leave_it_behind::{ActiveScenario, AppState, Scenario};

use crate::common::{
    boot, frame, mouse_down, run_frames, run_until, secs, state, tap, test_app_with_storage,
};

/// Ends the app, keeping what it saved.
fn saved(mut app: App) -> Storage {
    app.world_mut().remove_resource::<Storage>().unwrap()
}

fn flown(scenario: Scenario, storage: Storage) -> App {
    let mut app = test_app_with_storage(storage);
    app.insert_resource(ActiveScenario(scenario));
    boot(&mut app);
    app
}

#[test]
fn settings_survive_a_restart_of_the_game() {
    let mut app = flown(Scenario::Quiet, Storage::memory());
    {
        let mut settings = app.world_mut().resource_mut::<Settings>();
        settings.shake = 25;
        settings.pause_unfocused = false;
    }
    frame(&mut app);
    let storage = saved(app);
    assert!(storage.read(SETTINGS_KEY).unwrap().is_some());

    let app = flown(Scenario::Quiet, storage);
    let settings = app.world().resource::<Settings>();
    assert_eq!(settings.shake, 25);
    assert!(!settings.pause_unfocused);
}

#[test]
fn nothing_is_written_until_something_changes() {
    let mut app = flown(Scenario::Quiet, Storage::memory());
    run_frames(&mut app, 10);
    let storage = saved(app);
    assert_eq!(storage.read(SETTINGS_KEY), Ok(None));
    assert_eq!(storage.read(SAVE_KEY), Ok(None));
}

#[test]
fn landings_are_recorded_and_the_best_is_kept() {
    let mut app = flown(Scenario::Landing, Storage::memory());
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    let last = *app.world().resource::<LastRun>();
    assert!(last.new_best);
    assert!(last.record.unwrap().landed);

    // The same flight again only ties: not a new best.
    tap(&mut app, KeyCode::KeyR);
    tap(&mut app, KeyCode::Enter);
    run_until(&mut app, 3, |app| state(app) == AppState::Playing);
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    frame(&mut app);
    assert!(!app.world().resource::<LastRun>().new_best);

    let storage = saved(app);
    let app = flown(Scenario::Quiet, storage);
    let progress = app.world().resource::<Progress>();
    let one = progress.level("one").expect("level one flown");
    assert_eq!((one.flights, one.landings), (2, 2));
    assert!(one.best.unwrap().landed);
}

#[test]
fn a_loss_is_recorded_but_never_beats_a_landing() {
    let mut app = flown(Scenario::Landing, Storage::memory());
    run_until(&mut app, secs(4.0), |app| state(app) == AppState::Landed);
    let storage = saved(app);

    let mut app = flown(Scenario::BreachCritical, storage);
    run_until(&mut app, secs(3.0), |app| state(app) == AppState::Lost);
    frame(&mut app);
    assert!(!app.world().resource::<LastRun>().new_best);
    let progress = app.world().resource::<Progress>().clone();
    let one = progress.level("one").unwrap();
    assert_eq!((one.flights, one.landings), (2, 1));
    assert!(one.best.unwrap().landed);
}

#[test]
fn damage_counts_how_long_each_fault_ran() {
    let mut app = flown(Scenario::Breach, Storage::memory());
    run_frames(&mut app, secs(2.0));
    mouse_down(&mut app);
    run_until(&mut app, secs(6.0), |app| {
        app.world().resource::<RunStats>().fixed == 1
    });
    let damage = app.world().resource::<RunStats>().damage;
    // Two seconds of waiting plus about three of taping.
    assert!((4.5..6.0).contains(&damage.oxygen), "{damage:?}");
    assert_eq!((damage.course, damage.engine), (0.0, 0.0));
    run_frames(&mut app, 30);
    assert_eq!(app.world().resource::<RunStats>().damage, damage);
}

#[test]
fn an_unreadable_save_falls_back_to_defaults_with_a_notice() {
    let mut storage = Storage::memory();
    storage
        .write(SETTINGS_KEY, "(version: 1, settings: (shake: \"loud\"))")
        .unwrap();
    storage
        .write(SAVE_KEY, "(version: 7, progress: ())")
        .unwrap();
    let mut app = flown(Scenario::Quiet, storage);
    assert_eq!(*app.world().resource::<Settings>(), Settings::default());
    assert_eq!(*app.world().resource::<Progress>(), Progress::default());
    frame(&mut app);
    let notice = app
        .world()
        .resource::<Notices>()
        .current()
        .unwrap()
        .to_string();
    assert!(notice.contains("settings"), "{notice}");
    let storage = saved(app);
    assert!(
        storage
            .read("settings.bak")
            .unwrap()
            .unwrap()
            .contains("loud")
    );
    assert!(
        storage
            .read("save.bak")
            .unwrap()
            .unwrap()
            .contains("version: 7")
    );
}
