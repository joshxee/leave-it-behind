//! Loose bolts. A fault loosens the three bolts of one engine panel; each
//! needs one wrench turn (walk up, click; the wrench head snaps to the bolt).
//! The engine block glows hotter as the fault's clock runs down.

use bevy::prelude::*;

use super::sites::BOLT_SITES;
use super::{Fault, FaultKind, Site};
use crate::shapes::{Shapes, at};
use crate::ship::{BaseColor, EngineBlock};
use crate::tools::{WrenchTarget, WrenchTightened};
use crate::{AppState, GameSet, RunSet, palette};

pub const BOLTS_PER_PANEL: usize = 3;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Bolt {
    pub panel: Site,
    /// Position on the engine face (what the wrench snaps to).
    pub pos: Vec2,
    pub loose: bool,
}

pub struct BoltsPlugin;

impl Plugin for BoltsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_bolts.in_set(GameSet::Input))
            .add_systems(
                OnEnter(AppState::Playing),
                tighten_all.in_set(RunSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                (loosen_bolts, tighten_bolts)
                    .chain()
                    .in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, (draw_bolts, heat_engines).in_set(GameSet::Present));
    }
}

fn spawn_bolts(mut commands: Commands, shapes: Res<Shapes>) {
    for panel in BOLT_SITES {
        for pos in panel.bolts().into_iter().flatten() {
            commands.spawn((
                Bolt {
                    panel,
                    pos,
                    loose: false,
                },
                shapes.circle(7.0, palette::BOLT_TIGHT),
                at(pos, 1.4),
            ));
        }
    }
}

fn tighten_all(mut commands: Commands, mut bolts: Query<(Entity, &mut Bolt)>) {
    for (entity, mut bolt) in &mut bolts {
        bolt.loose = false;
        commands.entity(entity).remove::<WrenchTarget>();
    }
}

fn loosen_bolts(
    mut commands: Commands,
    new_faults: Query<&Fault, Added<Fault>>,
    mut bolts: Query<(Entity, &mut Bolt)>,
) {
    for fault in &new_faults {
        if fault.kind() != FaultKind::LooseBolts {
            continue;
        }
        for (entity, mut bolt) in &mut bolts {
            if bolt.panel == fault.site {
                bolt.loose = true;
                commands
                    .entity(entity)
                    .insert(WrenchTarget { pos: bolt.pos });
            }
        }
    }
}

fn tighten_bolts(
    mut commands: Commands,
    mut tightened: MessageReader<WrenchTightened>,
    mut bolts: Query<&mut Bolt>,
    mut faults: Query<&mut Fault>,
) {
    for msg in tightened.read() {
        let Ok(mut bolt) = bolts.get_mut(msg.target) else {
            continue;
        };
        if !bolt.loose {
            continue;
        }
        bolt.loose = false;
        let panel = bolt.panel;
        commands.entity(msg.target).remove::<WrenchTarget>();
        let still_loose = bolts.iter().filter(|b| b.panel == panel && b.loose).count();
        for mut fault in &mut faults {
            if fault.site == panel {
                fault.repair = 1.0 - still_loose as f32 / BOLTS_PER_PANEL as f32;
            }
        }
    }
}

/// Loose bolts stick out of the face and rattle; tight ones sit flush.
fn draw_bolts(time: Res<Time>, mut bolts: Query<(&Bolt, &mut Transform, &mut Sprite)>) {
    let t = time.elapsed_secs();
    for (bolt, mut transform, mut sprite) in &mut bolts {
        let n = bolt.panel.normal();
        let (offset, color, radius) = if bolt.loose {
            let rattle = (t * 21.0 + bolt.pos.x * 0.07).sin() * 2.5;
            (n * 4.0 + n.perp() * rattle, palette::BOLT_LOOSE, 9.0)
        } else {
            (-n * 8.0, palette::BOLT_TIGHT, 7.0)
        };
        transform.translation = (bolt.pos + offset).extend(1.4);
        sprite.color = color;
        sprite.custom_size = Some(Vec2::splat(radius * 2.0));
    }
}

pub fn engine_of(site: Site) -> Option<EngineBlock> {
    match site {
        Site::PortEngineInner | Site::PortEngineOuter => Some(EngineBlock::Port),
        Site::StarboardEngineInner | Site::StarboardEngineOuter => Some(EngineBlock::Starboard),
        _ => None,
    }
}

/// Engine heat: the in-room cue that this engine has a bolt fault.
fn heat_engines(
    faults: Query<&Fault>,
    mut engines: Query<(&EngineBlock, &BaseColor, &mut Sprite)>,
) {
    for (engine, base, mut sprite) in &mut engines {
        let heat = faults
            .iter()
            .filter(|f| engine_of(f.site) == Some(*engine))
            .map(|f| f.urgency())
            .fold(0.0, f32::max);
        sprite.color = base.0.mix(&palette::ENGINE_HOT, heat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_bolt_panel_has_an_engine() {
        for site in BOLT_SITES {
            assert!(engine_of(site).is_some());
            assert_eq!(site.bolts().unwrap().len(), BOLTS_PER_PANEL);
        }
        assert_eq!(engine_of(Site::Helm), None);
    }
}
