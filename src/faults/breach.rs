//! Hull breaches: a hole in an airlock or main hull wall. The wall's tile
//! turns into a breach (space shows through it) and air rushes out in
//! pulses. Holding the tape over it for [`SEAL_SECS`] seals it, and the tile
//! becomes a strapped patch for the rest of the run. Tape laid anywhere else
//! is wasted.

use bevy::prelude::*;

use super::{Fault, FaultFixed, FaultKind, Site};
use crate::art::tiles::Tile;
use crate::shapes::Shapes;
use crate::ship::layout::ship;
use crate::ship::{WallCell, show_tile};
use crate::tools::TapeLaid;
use crate::{AppState, GameSet, palette};

/// Seconds of tape on the hole to seal it.
pub const SEAL_SECS: f32 = 3.0;
/// Tape laid within this distance of the hole counts toward sealing it.
pub const BREACH_RADIUS: f32 = 26.0;

/// Air rushing out of a breach (a child of the fault).
#[derive(Component, Debug)]
struct Vent;

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
        .add_systems(
            FixedUpdate,
            patch_sealed
                .after(super::resolve_faults)
                .in_set(GameSet::Resolve)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(Update, draw_breaches.in_set(GameSet::Present));
    }
}

/// Swaps the wall tile at a breach site.
fn set_wall(walls: &mut Query<(&WallCell, &mut Sprite)>, site: Site, tile: fn(bool) -> Tile) {
    let Some(cell) = site.wall_cell() else {
        return;
    };
    for (wall, mut sprite) in walls.iter_mut() {
        if wall.0 == cell {
            show_tile(&mut sprite, tile(ship().across_x(cell)));
        }
    }
}

fn open_breaches(
    mut commands: Commands,
    shapes: Res<Shapes>,
    new_faults: Query<(Entity, &Fault), Added<Fault>>,
    mut walls: Query<(&WallCell, &mut Sprite)>,
) {
    for (entity, fault) in &new_faults {
        if fault.kind() != FaultKind::HullBreach {
            continue;
        }
        set_wall(&mut walls, fault.site, Tile::breach);
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Vent,
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

/// A sealed breach becomes a strapped patch.
fn patch_sealed(mut fixed: MessageReader<FaultFixed>, mut walls: Query<(&WallCell, &mut Sprite)>) {
    for msg in fixed.read() {
        if msg.site.kind() == FaultKind::HullBreach {
            set_wall(&mut walls, msg.site, Tile::patched);
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
