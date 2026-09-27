//! Raised walls with narrow caps at the outside of the room's grid cells.
//! The collision core remains the walking guard band; no wall is lowered
//! or faded when the engineer approaches it.
use bevy::prelude::*;

use super::{CurrentRoom, WallCell, map::ship};
use crate::GameSet;
use crate::art::maintenance::MaintenanceArt;
use crate::player::Player;

pub const WALL_HEIGHT: f32 = 22.0;
pub const WALL_OUTSET: f32 = 18.0;

/// Project a contact on the floor-level wall core onto its raised face.
pub fn wall_art_point(point: Vec2, inward: Vec2) -> Vec2 {
    point - inward * (super::map::WALL_HALF + WALL_OUTSET) + Vec2::Y * WALL_HEIGHT / 2.0
}

/// Atlas variant for the room-facing quadrants of a joining wall cell.
pub fn wall_frame(center: Vec2, interior: Rect, mask: u8) -> usize {
    let mut outside = 0;
    for (i, corner) in [
        Vec2::new(-16.0, 16.0),
        Vec2::new(16.0, 16.0),
        Vec2::new(-16.0, -16.0),
        Vec2::new(16.0, -16.0),
    ]
    .into_iter()
    .enumerate()
    {
        if !interior.contains(center + corner) {
            outside |= 1 << i;
        }
    }
    outside * 16 + mask as usize
}

pub struct DepthPlugin;
impl Plugin for DepthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_depth, place_door_art, sort_engines).in_set(GameSet::Present),
        );
    }
}

fn sort_engines(room: Res<CurrentRoom>, mut engines: Query<(&super::EngineBlock, &mut Transform)>) {
    for (engine, mut transform) in &mut engines {
        let block = match engine {
            super::EngineBlock::Port => super::layout::port_engine(),
            super::EngineBlock::Starboard => super::layout::starboard_engine(),
        };
        transform.translation.z = 3.0 - (block.min.y - room.0.center().y) * 0.001;
    }
}

fn place_door_art(
    room: Res<CurrentRoom>,
    mut doors: Query<(&super::doors::DoorVisual, &mut Transform)>,
) {
    let interior = room.0.interior();
    for (door, mut transform) in &mut doors {
        let axis = if door.across_x { Vec2::Y } else { Vec2::X };
        let side = (door.center - interior.center()).dot(axis).signum();
        let pos = door.center + axis * side * WALL_OUTSET + Vec2::Y * 12.0;
        transform.translation = pos.extend(3.0 - (door.center.y - interior.center().y) * 0.001);
    }
}

fn draw_depth(
    room: Res<CurrentRoom>,
    art: Res<MaintenanceArt>,
    mut walls: Query<(&WallCell, &mut Sprite, &mut Transform, &mut Visibility), Without<Player>>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let interior = room.0.interior();
    let ground_z = |y: f32| 3.0 - (y - interior.center().y) * 0.001;
    for mut player in &mut players {
        player.translation.z = ground_z(player.translation.y);
    }
    for (wall, mut sprite, mut transform, mut visibility) in &mut walls {
        let center = ship().center(wall.0);
        let frame = wall_frame(center, interior, ship().mask(wall.0));
        art.frame(&mut sprite, "wall-depth", frame);
        transform.translation.z = ground_z(center.y);
        *visibility = if frame / 16 == 15 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_and_rear_use_the_same_full_height_sheet() {
        let room = Rect::new(-100.0, -100.0, 100.0, 100.0);
        assert_eq!(wall_frame(Vec2::new(0.0, -112.0), room, 10), 12 * 16 + 10);
        assert_eq!(wall_frame(Vec2::new(0.0, 112.0), room, 10), 3 * 16 + 10);
    }

    #[test]
    fn tape_and_holes_share_the_wall_surface() {
        for normal in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            let contact = normal * super::super::map::WALL_HALF;
            assert_eq!(
                wall_art_point(contact, normal),
                -normal * WALL_OUTSET + Vec2::Y * 11.0
            );
        }
    }
}
