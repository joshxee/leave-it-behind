//! Ambient cues: the only warning outside the diagnostic screen. They say
//! that something is wrong and how urgent it is, never where.
//!
//! - A jolt of screen shake when a fault starts.
//! - A pulsing red tint while any fault is active, deeper and faster as the
//!   most urgent fault's clock runs down.
//! - Growing shake over the last half of the most urgent fault's clock.

use bevy::prelude::*;

use crate::faults::Fault;
use crate::ship::CameraRig;
use crate::{AppState, GameSet, RunSet, palette};

/// Seconds a new fault's jolt lasts.
pub const JOLT_SECS: f32 = 0.6;
/// Peak jolt shake in world units.
pub const JOLT_SHAKE: f32 = 10.0;
/// Shake at the moment a fault becomes fatal.
pub const CRITICAL_SHAKE: f32 = 6.0;
/// Tint alpha at the moment a fault becomes fatal. (Blending is in linear
/// space, so small alphas already read clearly on the dark rooms.)
pub const MAX_TINT: f32 = 0.2;
/// Tint alpha as soon as any fault is active.
pub const MIN_TINT: f32 = 0.02;
/// Tint alpha on the lost screen.
const LOST_TINT: f32 = 0.3;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Alarm {
    /// Urgency of the worst active fault, 0 to 1.
    pub level: f32,
    /// Number of active faults.
    pub active: usize,
    /// Seconds of jolt left.
    pub jolt: f32,
}

impl Alarm {
    /// Tint alpha at time `t`: a pulse whose depth and rate grow with urgency.
    pub fn tint(&self, t: f32) -> f32 {
        if self.active == 0 {
            return 0.0;
        }
        let pulse = 0.5 + 0.5 * (t * (3.0 + 5.0 * self.level)).sin();
        let depth = MIN_TINT + (MAX_TINT - MIN_TINT) * self.level.powf(1.5);
        depth * (0.65 + 0.35 * pulse)
    }

    /// Shake amplitude in world units.
    pub fn shake(&self) -> f32 {
        let critical = ((self.level - 0.5) / 0.5).clamp(0.0, 1.0);
        JOLT_SHAKE * (self.jolt / JOLT_SECS) + CRITICAL_SHAKE * critical * critical
    }
}

/// The full-screen red tint.
#[derive(Component)]
struct AlarmTint;

pub struct AlarmPlugin;

impl Plugin for AlarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Alarm>()
            .add_systems(Startup, spawn_tint.in_set(GameSet::Input))
            .add_systems(OnEnter(AppState::Playing), reset.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                update_alarm
                    .in_set(GameSet::Resolve)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, (shake_camera, tint_screen).in_set(GameSet::Present));
    }
}

fn spawn_tint(mut commands: Commands) {
    commands.spawn((
        AlarmTint,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(5),
    ));
}

fn reset(mut alarm: ResMut<Alarm>) {
    *alarm = Alarm::default();
}

fn update_alarm(
    time: Res<Time>,
    faults: Query<&Fault>,
    new_faults: Query<(), Added<Fault>>,
    mut alarm: ResMut<Alarm>,
) {
    alarm.active = faults.iter().filter(|f| !f.is_repaired()).count();
    alarm.level = faults
        .iter()
        .filter(|f| !f.is_repaired())
        .map(Fault::urgency)
        .fold(0.0, f32::max);
    alarm.jolt = if new_faults.is_empty() {
        (alarm.jolt - time.delta_secs()).max(0.0)
    } else {
        JOLT_SECS
    };
}

/// Deterministic jitter in roughly [-1, 1]² (no RNG: purely cosmetic).
fn jitter(t: f32) -> Vec2 {
    Vec2::new(
        (t * 47.0).sin() + 0.5 * (t * 113.0).sin(),
        (t * 53.0).cos() + 0.5 * (t * 97.0).sin(),
    ) / 1.5
}

fn shake_camera(
    alarm: Res<Alarm>,
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut rigs: Query<&mut CameraRig>,
) {
    let amplitude = if *state.get() == AppState::Playing {
        alarm.shake()
    } else {
        0.0
    };
    for mut rig in &mut rigs {
        rig.shake = jitter(time.elapsed_secs()) * amplitude;
    }
}

fn tint_screen(
    alarm: Res<Alarm>,
    time: Res<Time>,
    state: Res<State<AppState>>,
    mut tints: Query<&mut BackgroundColor, With<AlarmTint>>,
) {
    let alpha = match state.get() {
        AppState::Playing => alarm.tint(time.elapsed_secs()),
        AppState::Lost => LOST_TINT,
        _ => 0.0,
    };
    for mut bg in &mut tints {
        bg.0 = palette::ALERT.with_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_ship_has_no_cues() {
        let alarm = Alarm::default();
        assert_eq!(alarm.tint(1.3), 0.0);
        assert_eq!(alarm.shake(), 0.0);
    }

    #[test]
    fn tint_deepens_with_urgency() {
        let calm = Alarm {
            level: 0.1,
            active: 1,
            jolt: 0.0,
        };
        let dire = Alarm { level: 0.9, ..calm };
        let peak = |a: Alarm| {
            (0..200)
                .map(|i| a.tint(i as f32 * 0.05))
                .fold(0.0, f32::max)
        };
        assert!(peak(calm) > 0.0);
        assert!(peak(dire) > 2.0 * peak(calm));
        assert!(peak(dire) <= MAX_TINT);
    }

    #[test]
    fn shake_is_a_jolt_then_a_late_rumble() {
        let fresh = Alarm {
            level: 0.0,
            active: 1,
            jolt: JOLT_SECS,
        };
        assert_eq!(fresh.shake(), JOLT_SHAKE);
        let halfway = Alarm {
            level: 0.5,
            jolt: 0.0,
            ..fresh
        };
        assert_eq!(halfway.shake(), 0.0);
        let fatal = Alarm {
            level: 1.0,
            ..halfway
        };
        assert_eq!(fatal.shake(), CRITICAL_SHAKE);
    }
}
