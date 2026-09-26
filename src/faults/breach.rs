//! Hull breaches: a hole in the airlock or main hull wall. Holding the tape
//! over it for [`SEAL_SECS`] seals it. Tape laid anywhere else is wasted.

use bevy::prelude::*;

use super::{Fault, FaultKind};
use crate::shapes::Shapes;
use crate::tools::TapeLaid;
use crate::{AppState, GameSet, palette};

/// Seconds of tape on the hole to seal it.
pub const SEAL_SECS: f32 = 3.0;
/// Tape laid within this distance of the hole counts toward sealing it.
pub const BREACH_RADIUS: f32 = 26.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum BreachPart {
    Hole,
    Air,
}

pub struct BreachPlugin;

impl Plugin for BreachPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (open_breaches, seal_breaches)
                .chain()
                .in_set(GameSet::Simulate)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, draw_breaches.in_set(GameSet::Present));
    }
}

fn open_breaches(
    mut commands: Commands,
    shapes: Res<Shapes>,
    new_faults: Query<(Entity, &Fault), Added<Fault>>,
) {
    for (entity, fault) in &new_faults {
        if fault.kind() != FaultKind::HullBreach {
            continue;
        }
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                BreachPart::Hole,
                shapes.circle(16.0, palette::BREACH_HOLE),
                Transform::from_xyz(0.0, 0.0, 1.6),
            ));
            parent.spawn((
                BreachPart::Air,
                shapes.ring(30.0, palette::BREACH_AIR),
                Transform::from_xyz(0.0, 0.0, 1.65),
            ));
        });
    }
}

/// Adds each tick's tape to the nearest breach it touches.
fn seal_breaches(mut laid: MessageReader<TapeLaid>, mut faults: Query<&mut Fault>) {
    for tape in laid.read() {
        let nearest = faults
            .iter_mut()
            .map(|f| (f.site.pos().distance(tape.point), f))
            .filter(|(d, f)| f.kind() == FaultKind::HullBreach && *d <= BREACH_RADIUS)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, f)| f);
        if let Some(mut fault) = nearest {
            fault.repair = (fault.repair + tape.secs / SEAL_SECS).min(1.0);
        }
    }
}

/// Air rushing out: a ring expanding from the hole, fading as tape covers it.
fn draw_breaches(
    time: Res<Time>,
    faults: Query<(&Fault, &Children)>,
    mut parts: Query<(&BreachPart, &mut Transform, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (fault, children) in &faults {
        for child in children {
            let Ok((part, mut transform, mut sprite)) = parts.get_mut(*child) else {
                continue;
            };
            match part {
                BreachPart::Hole => {
                    sprite.custom_size = Some(Vec2::splat(32.0 * (1.0 - 0.7 * fault.repair)));
                }
                BreachPart::Air => {
                    let phase = (t * 1.7).fract();
                    transform.scale = Vec3::splat(0.5 + phase);
                    let strength = (1.0 - phase) * (1.0 - fault.repair);
                    sprite.color = palette::BREACH_AIR.with_alpha(0.9 * strength);
                }
            }
        }
    }
}
