use bevy::prelude::*;
use leave_it_behind::scoring::{Score, ScoreChanged, ThresholdReached};

use crate::common::{boot, tap, test_app};

#[test]
fn space_adds_one_point_and_sends_message() {
    let mut app = test_app();
    boot(&mut app);
    let messages = app.world().resource::<Messages<ScoreChanged>>();
    let mut cursor = messages.get_cursor_current();

    tap(&mut app, KeyCode::Space);
    app.update();

    assert_eq!(*app.world().resource::<Score>(), Score(1));
    let messages = app.world().resource::<Messages<ScoreChanged>>();
    let seen: Vec<_> = cursor.read(messages).copied().collect();
    assert_eq!(seen, vec![ScoreChanged { score: 1 }]);
}

#[test]
fn holding_space_scores_once() {
    let mut app = test_app();
    boot(&mut app);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    for _ in 0..10 {
        app.update();
        // Pressed stays true; just_pressed is cleared like InputPlugin would.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
    }
    assert_eq!(*app.world().resource::<Score>(), Score(1));
}

#[test]
fn threshold_message_not_sent_below_threshold() {
    let mut app = test_app();
    boot(&mut app);
    let mut cursor = app
        .world()
        .resource::<Messages<ThresholdReached>>()
        .get_cursor_current();
    tap(&mut app, KeyCode::Space);
    app.update();
    let messages = app.world().resource::<Messages<ThresholdReached>>();
    assert_eq!(cursor.read(messages).count(), 0);
}
