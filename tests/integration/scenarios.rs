use bevy::prelude::*;
use leave_it_behind::Scenario;
use leave_it_behind::scoring::{SCORE_THRESHOLD, Score, ThresholdReached};

use crate::common::{tap, test_app_with};

#[test]
fn every_scenario_boots() {
    for &scenario in Scenario::ALL {
        let _ = test_app_with(scenario);
    }
}

#[test]
fn score_nine_reaches_threshold_with_one_press() {
    let mut app = test_app_with(Scenario::ScoreNine);
    assert_eq!(*app.world().resource::<Score>(), Score(SCORE_THRESHOLD - 1));
    let mut cursor = app
        .world()
        .resource::<Messages<ThresholdReached>>()
        .get_cursor_current();

    tap(&mut app, KeyCode::Space);
    app.update();

    assert_eq!(*app.world().resource::<Score>(), Score(SCORE_THRESHOLD));
    let messages = app.world().resource::<Messages<ThresholdReached>>();
    assert_eq!(cursor.read(messages).count(), 1);
}
