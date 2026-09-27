//! Trajectory drift, fixed at the nav helm. E by the joystick locks the
//! engineer in place; WASD then nudges the course marker on the nav display.
//! While a drift is active the marker wanders off on its own; holding it in
//! the centre band for [`HOLD_SECS`] fixes the course (progress drains
//! while it is outside). E again leaves the helm; a fix leaves it too.
//!
//! Marker coordinates are normalized: the display spans −1..1 on each axis.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{Fault, FaultKind};
use crate::player::{InteractKind, InteractPressed, Interactable, Locked, Player, PlayerIntent};
use crate::shapes::{Shapes, at, rect};
use crate::ship::layout;
use crate::{AppState, GameRng, GameSet, RunSet, palette};

/// Half-width of the centre band.
pub const BAND_HALF: f32 = 0.35;
/// Seconds in the band to fix the course.
pub const HOLD_SECS: f32 = 3.0;
/// Hold progress lost per second outside the band.
pub const HOLD_DECAY: f32 = 0.25;
/// Marker drift speed while the fault is active (display half-widths per second).
pub const DRIFT_SPEED: f32 = 0.2;
/// Marker speed under WASD.
pub const NUDGE_SPEED: f32 = 0.9;
/// Without a drift the marker settles back to the centre at this speed.
pub const RECENTER_SPEED: f32 = 0.6;
/// How far off-centre a new drift throws the marker.
pub const KICK: f32 = 0.55;
/// Drift heading turn rate range (radians per second, either direction).
const TURN_RANGE: (f32, f32) = (0.2, 0.45);
/// E works within this distance of the joystick.
pub const HELM_RANGE: f32 = 75.0;

/// Nav display state.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Nav {
    pub marker: Vec2,
    /// Direction the course drifts (radians) and how fast that turns.
    pub heading: f32,
    pub turn: f32,
    /// The engineer is at the helm.
    pub engaged: bool,
    /// Current WASD input at the helm (tilts the joystick).
    pub stick: Vec2,
}

pub fn in_band(marker: Vec2) -> bool {
    marker.x.abs() <= BAND_HALF && marker.y.abs() <= BAND_HALF
}

/// One tick of marker motion: drift along `heading` (when drifting) or
/// settle to the centre, plus the helm input.
pub fn step_marker(marker: Vec2, heading: Option<f32>, input: Vec2, dt: f32) -> Vec2 {
    let push = input.normalize_or_zero() * NUDGE_SPEED * dt;
    let own = match heading {
        Some(h) => Vec2::from_angle(h) * DRIFT_SPEED * dt,
        None => (-marker).clamp_length_max(RECENTER_SPEED * dt),
    };
    (marker + own + push).clamp(Vec2::NEG_ONE, Vec2::ONE)
}

/// Hold progress after one tick in or out of the band. A completed hold
/// stays complete.
pub fn step_hold(hold: f32, inside: bool, dt: f32) -> f32 {
    if hold >= 1.0 {
        1.0
    } else if inside {
        (hold + dt / HOLD_SECS).min(1.0)
    } else {
        (hold - HOLD_DECAY * dt).max(0.0)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum NavPart {
    Frame,
    Marker,
    Hold,
    Stick,
}

pub struct DriftPlugin;

impl Plugin for DriftPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nav>()
            .add_systems(Startup, spawn_helm.in_set(GameSet::Input))
            .add_systems(OnEnter(AppState::Playing), reset_nav.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                use_helm
                    .in_set(GameSet::Act)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (start_drift, steer)
                    .chain()
                    .in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, draw_nav.in_set(GameSet::Present));
    }
}

fn spawn_helm(mut commands: Commands, shapes: Res<Shapes>) {
    let screen = layout::nav_screen();
    let c = screen.center();
    commands.spawn((
        NavPart::Frame,
        rect(screen.size() + 8.0, palette::PROP),
        at(c, 1.1),
    ));
    commands.spawn((rect(screen.size(), palette::NAV_SCREEN), at(c, 1.12)));
    // Two crossing bands; their overlap in the middle is the target.
    let band = screen.size() * BAND_HALF;
    commands.spawn((
        rect(Vec2::new(screen.width(), band.y * 2.0), palette::NAV_BAND),
        at(c, 1.14),
    ));
    commands.spawn((
        rect(Vec2::new(band.x * 2.0, screen.height()), palette::NAV_BAND),
        at(c, 1.14),
    ));
    commands.spawn((
        NavPart::Hold,
        rect(Vec2::new(0.0, 6.0), palette::NAV_MARKER),
        at(c, 1.15),
    ));
    commands.spawn((
        NavPart::Marker,
        shapes.circle(8.0, palette::NAV_MARKER),
        at(c, 1.16),
    ));

    let joystick = layout::joystick();
    commands.spawn((
        Interactable {
            kind: InteractKind::Helm,
            range: HELM_RANGE,
        },
        shapes.circle(16.0, palette::JOYSTICK_BASE),
        at(joystick, 1.3),
    ));
    commands.spawn((shapes.ring(16.0, palette::STEEL), at(joystick, 1.32)));
    commands.spawn((
        NavPart::Stick,
        shapes.circle(8.0, palette::JOYSTICK),
        at(joystick, 1.35),
    ));
}

fn reset_nav(mut nav: ResMut<Nav>) {
    *nav = Nav::default();
}

fn use_helm(
    mut commands: Commands,
    mut presses: MessageReader<InteractPressed>,
    mut nav: ResMut<Nav>,
    mut players: Query<(Entity, &mut Transform), With<Player>>,
) {
    for press in presses.read() {
        let Some((player, mut transform)) = players.iter_mut().next() else {
            return;
        };
        if nav.engaged {
            nav.engaged = false;
            commands.entity(player).remove::<Locked>();
        } else if press.target == Some(InteractKind::Helm) {
            nav.engaged = true;
            commands.entity(player).insert(Locked);
            transform.translation = layout::helm_seat().extend(transform.translation.z);
        }
    }
}

fn start_drift(
    new_faults: Query<&Fault, Added<Fault>>,
    mut nav: ResMut<Nav>,
    mut rng: ResMut<GameRng>,
) {
    for fault in &new_faults {
        if fault.kind() != FaultKind::TrajectoryDrift {
            continue;
        }
        nav.heading = rng.range(0.0, TAU);
        let rate = rng.range(TURN_RANGE.0, TURN_RANGE.1);
        nav.turn = if rng.unit() < 0.5 { rate } else { -rate };
        nav.marker = Vec2::from_angle(nav.heading) * KICK;
    }
}

fn steer(
    mut commands: Commands,
    time: Res<Time>,
    intent: Res<PlayerIntent>,
    mut nav: ResMut<Nav>,
    mut faults: Query<&mut Fault>,
    players: Query<Entity, With<Player>>,
) {
    let dt = time.delta_secs();
    nav.stick = if nav.engaged {
        intent.direction.normalize_or_zero()
    } else {
        Vec2::ZERO
    };
    let drift = faults
        .iter_mut()
        .find(|f| f.kind() == FaultKind::TrajectoryDrift);
    let heading = drift.as_ref().map(|_| nav.heading);
    nav.marker = step_marker(nav.marker, heading, nav.stick, dt);
    if let Some(mut fault) = drift {
        nav.heading += nav.turn * dt;
        fault.repair = step_hold(fault.repair, in_band(nav.marker), dt);
        if fault.is_repaired() && nav.engaged {
            // Course locked in: the engineer is free to go.
            nav.engaged = false;
            for player in &players {
                commands.entity(player).remove::<Locked>();
            }
        }
    }
}

fn draw_nav(
    nav: Res<Nav>,
    time: Res<Time>,
    faults: Query<&Fault>,
    mut parts: Query<(&NavPart, &mut Transform, &mut Sprite)>,
) {
    let screen = layout::nav_screen();
    let half = screen.half_size() - Vec2::splat(8.0);
    let drift = faults
        .iter()
        .find(|f| f.kind() == FaultKind::TrajectoryDrift);
    let alarm = drift.is_some() && !in_band(nav.marker);
    let blink = (time.elapsed_secs() * 4.0).fract() < 0.5;
    for (part, mut transform, mut sprite) in &mut parts {
        match part {
            NavPart::Frame => {
                sprite.color = if alarm && blink {
                    palette::NAV_ALERT
                } else {
                    palette::PROP
                };
            }
            NavPart::Marker => {
                let p = screen.center() + nav.marker * half;
                transform.translation = p.extend(transform.translation.z);
                sprite.color = if alarm {
                    palette::NAV_ALERT
                } else {
                    palette::NAV_MARKER
                };
            }
            NavPart::Hold => {
                let hold = drift.map_or(0.0, |f| f.repair);
                let width = screen.width() * hold;
                transform.translation.x = screen.min.x + width / 2.0;
                transform.translation.y = screen.min.y + 6.0;
                sprite.custom_size = Some(Vec2::new(width, 6.0));
            }
            NavPart::Stick => {
                let p = layout::joystick() + nav.stick * 7.0;
                transform.translation = p.extend(transform.translation.z);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn marker_drifts_along_the_heading() {
        let m = step_marker(Vec2::ZERO, Some(0.0), Vec2::ZERO, 1.0);
        assert!((m - Vec2::new(DRIFT_SPEED, 0.0)).length() < 1e-5);
    }

    #[test]
    fn marker_settles_to_the_centre_without_overshooting() {
        let m = step_marker(Vec2::new(0.01, 0.0), None, Vec2::ZERO, 1.0);
        assert_eq!(m, Vec2::ZERO);
    }

    #[test]
    fn marker_stays_on_the_display() {
        let m = step_marker(Vec2::new(0.99, 0.0), Some(0.0), Vec2::X, 1.0);
        assert_eq!(m.x, 1.0);
    }

    #[test]
    fn nudging_beats_the_drift() {
        // Pushing straight against the drift still makes headway.
        let m = step_marker(Vec2::new(0.5, 0.0), Some(0.0), Vec2::NEG_X, 1.0);
        assert!(m.x < 0.5);
    }

    #[test]
    fn three_seconds_in_the_band_fix_the_course() {
        let mut hold = 0.0;
        for _ in 0..(HOLD_SECS / DT).round() as usize {
            hold = step_hold(hold, true, DT);
        }
        assert!(hold >= 0.999, "{hold}");
        hold = step_hold(0.5, false, 1.0);
        assert_eq!(hold, 0.5 - HOLD_DECAY);
    }

    #[test]
    fn band_is_the_centre_square() {
        assert!(in_band(Vec2::new(BAND_HALF, -BAND_HALF)));
        assert!(!in_band(Vec2::new(BAND_HALF + 0.01, 0.0)));
    }
}
