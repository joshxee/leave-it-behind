//! Hull breaches are projected onto the raised wall face. Air pulses out
//! until [`SEAL_SECS`] of tape seals the hole. The woven strips persist
//! for the rest of the run; tape laid elsewhere is wasted.

use bevy::prelude::*;

use super::{Fault, FaultKind};
use crate::art::maintenance::MaintenanceArt;
use crate::shapes::Shapes;
use crate::tools::TapeLaid;
use crate::{GameSet, not_paused, palette, running};

/// Seconds of tape on the hole to seal it.
pub const SEAL_SECS: f32 = 3.0;
/// Tape laid within this distance of the hole counts toward sealing it.
pub const BREACH_RADIUS: f32 = 26.0;

/// Air rushing out of a breach (a child of the fault).
#[derive(Component, Debug)]
struct Vent;

#[derive(Component, Debug)]
struct BreachHole;

pub struct BreachPlugin;

impl Plugin for BreachPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (open_breaches, seal_breaches)
                .chain()
                .in_set(GameSet::Simulate)
                .run_if(running),
        )
        .add_systems(
            Update,
            draw_breaches.in_set(GameSet::Present).run_if(not_paused),
        );
    }
}

/// Draw the tear and its vent on the same surface as applied tape.
fn open_breaches(
    mut commands: Commands,
    shapes: Res<Shapes>,
    new_faults: Query<(Entity, &Fault), Added<Fault>>,
    art: Res<MaintenanceArt>,
) {
    for (entity, fault) in &new_faults {
        if fault.kind() != FaultKind::HullBreach {
            continue;
        }
        let offset = crate::ship::depth::wall_art_point(fault.site.pos(), fault.site.normal())
            - fault.site.pos();
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                BreachHole,
                art.sprite("breach", 0, Vec2::new(40.0, 32.0)),
                Transform::from_translation(offset.extend(4.8))
                    .with_rotation(Quat::from_rotation_z(fault.site.normal().perp().to_angle())),
            ));
            parent.spawn((
                Vent,
                shapes.ring(30.0, palette::BREACH_AIR),
                Transform::from_translation(offset.extend(4.9)),
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
    mut vents: Query<(&mut Transform, &mut Sprite), With<Vent>>,
) {
    let t = time.elapsed_secs();
    for (fault, children) in &faults {
        for child in children {
            let Ok((mut transform, mut sprite)) = vents.get_mut(*child) else {
                continue;
            };
            let phase = (t * 1.7).fract();
            transform.scale = Vec3::splat(0.5 + phase);
            let strength = (1.0 - phase) * (1.0 - fault.repair);
            sprite.color = palette::BREACH_AIR.with_alpha(0.9 * strength);
        }
    }
}
