use leave_it_behind::Scenario;
use leave_it_behind::alarm::{Alarm, JOLT_SECS};

use crate::common::{frame, run_frames, secs, test_app_with};

fn alarm(app: &bevy::prelude::App) -> Alarm {
    *app.world().resource::<Alarm>()
}

#[test]
fn a_quiet_ship_raises_no_alarm() {
    let mut app = test_app_with(Scenario::Quiet);
    run_frames(&mut app, secs(2.0));
    let a = alarm(&app);
    assert_eq!((a.active, a.level, a.jolt), (0, 0.0, 0.0));
    assert_eq!(a.tint(1.0), 0.0);
    assert_eq!(a.shake(), 0.0);
}

#[test]
fn a_new_fault_jolts_then_the_tint_deepens() {
    let mut app = test_app_with(Scenario::Breach);
    frame(&mut app);
    let start = alarm(&app);
    assert_eq!(start.active, 1);
    assert!(start.jolt > JOLT_SECS - 0.05, "{start:?}");
    run_frames(&mut app, secs(30.0));
    let later = alarm(&app);
    assert_eq!(later.jolt, 0.0);
    assert!(later.level > 0.5, "{later:?}");
    let peak = |a: Alarm| (0..100).map(|i| a.tint(i as f32 * 0.1)).fold(0.0, f32::max);
    assert!(peak(later) > peak(start));
    assert!(later.shake() > 0.0);
}

#[test]
fn the_worst_fault_sets_the_level() {
    let mut app = test_app_with(Scenario::Scramble);
    run_frames(&mut app, secs(10.0));
    // Scramble clocks: 50 s, 45 s and 50 s. The 45 s breach is the worst.
    let a = alarm(&app);
    assert_eq!(a.active, 3);
    assert!((a.level - 10.0 / 45.0).abs() < 0.01, "{a:?}");
}
