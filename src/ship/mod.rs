//! The ship: rooms, walls, doors and props drawn from the map (`map.rs`)
//! with the derelict-ship tiles, the room the player is in, and a camera
//! that frames only that room.

pub mod depth;
pub mod doors;
pub mod layout;
pub mod map;

use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub use map::RoomId;

use crate::art::maintenance::MaintenanceArt;
use crate::art::tiles::Tile;
use crate::art::{Art, TILE};
use crate::player::Player;
use crate::shapes::{at, rect};
use crate::{AppState, GameSet, RunSet, palette, running};
use map::{Kind, ship};

/// Draw order (z) of the ship's layers. Faults, tools and the engineer sit
/// above these; the curtains above everything.
pub mod z {
    pub const SPACE: f32 = 0.0;
    pub const FLOOR: f32 = 0.1;
    pub const STRUCTURE: f32 = 1.0;
    pub const PROP: f32 = 1.2;
    pub const CURTAIN: f32 = 10.0;
}

/// The room the player occupies. The camera shows only this room.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentRoom(pub RoomId);

/// Everything static and solid: walls and props (door leaves are separate).
#[derive(Resource, Debug, Clone)]
pub struct Colliders(pub Vec<Rect>);

/// Walls only (hull, bulkheads, door jambs). Tape sticks to these.
#[derive(Resource, Debug, Clone)]
pub struct Walls(pub Vec<Rect>);

/// The game camera. `anchor` is the current room's center; the alarm adds
/// `shake` on top. Aiming uses `anchor`, so shake never moves the aim.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CameraRig {
    pub anchor: Vec2,
    pub shake: Vec2,
}

/// An engine assembly. Tinted by the
/// loose-bolts fault as the engine heats up.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineBlock {
    Port,
    Starboard,
}

/// The sprite of a wall cell, projected by the depth renderer.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WallCell(pub IVec2);

/// Void-colored sprites that hide everything beyond the current room.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Curtain {
    North,
    East,
    South,
    West,
}

const CURTAIN_SIZE: f32 = 20_000.0;

/// Sets an atlas sprite to `tile` (no change detection when it already is).
pub fn show_tile(sprite: &mut Sprite, tile: Tile) {
    if let Some(atlas) = sprite.texture_atlas.as_mut()
        && atlas.index != tile.index()
    {
        atlas.index = tile.index();
    }
}

/// What the camera shows of a room (everything else is curtained off).
pub fn room_frame(room: RoomId) -> Rect {
    let mut frame = ship().frame(room).inflate(12.0);
    frame.max.y += 22.0;
    frame
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentRoom(RoomId::at(layout::player_spawn())))
            .insert_resource(Colliders(layout::colliders()))
            .insert_resource(Walls(layout::walls()))
            .insert_resource(ClearColor(palette::VOID))
            .add_plugins((doors::DoorsPlugin, depth::DepthPlugin))
            .add_systems(Startup, (spawn_camera, spawn_ship).in_set(GameSet::Input))
            .add_systems(OnEnter(AppState::Playing), reset_room.in_set(RunSet::Spawn))
            .add_systems(FixedUpdate, track_room.in_set(GameSet::Act).run_if(running))
            .add_systems(
                Update,
                (frame_camera, place_curtains).in_set(GameSet::Present),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    let anchor = RoomId::at(layout::player_spawn()).center();
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: layout::VIEW.x,
                min_height: layout::VIEW.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        // Pixel art and flat shapes gain nothing from MSAA, and it
        // quadruples fill cost on software renderers.
        Msaa::Off,
        CameraRig {
            anchor,
            shake: Vec2::ZERO,
        },
        at(anchor, 0.0),
    ));
}

/// Draws every cell of the map: space, floor (only the inside quadrants
/// under hull walls), walls, the cockpit module and props. Doors are
/// `doors.rs`'s; the diagnostic console and the nav display belong to
/// their features.
fn spawn_ship(mut commands: Commands, art: Res<Art>, maintenance: Res<MaintenanceArt>) {
    let map = ship();
    let half = TILE / 2.0;
    for (cell, c) in map.cells() {
        let center = map.center(cell);
        if map.kind_at(cell) != Kind::Floor {
            // Seen through the hull's outside quadrants and breach holes.
            let space = if (cell.x + cell.y) % 2 == 0 {
                Tile::Space0
            } else {
                Tile::Space1
            };
            commands.spawn((art.tile(space), at(center, z::SPACE)));
        }
        let floor = map.floor_tile(cell);
        match map.floor_quadrants(cell) {
            [true, true, true, true] => {
                commands.spawn((art.tile(floor), at(center, z::FLOOR)));
            }
            quadrants => {
                for (i, _) in quadrants.iter().enumerate().filter(|(_, inside)| **inside) {
                    let (x, y) = ((i % 2) as f32 * half, (i / 2) as f32 * half);
                    commands.spawn((
                        art.tile_part(
                            floor,
                            Rect::new(
                                if x == 0.0 { 0.0 } else { 14.0 },
                                if y == 0.0 { 0.0 } else { 14.0 },
                                if x == 0.0 { 50.0 } else { 64.0 },
                                if y == 0.0 { 50.0 } else { 64.0 },
                            ),
                        ),
                        at(
                            center
                                + Vec2::new(
                                    if x == 0.0 { -7.0 } else { 7.0 },
                                    if y == 0.0 { 7.0 } else { -7.0 },
                                ),
                            z::FLOOR,
                        ),
                    ));
                }
            }
        }
        let prop = |tile: Tile| (art.tile(tile), at(center, z::PROP));
        match c {
            c if c == '#' || crate::ship::map::is_breach_mark(c) => {
                commands.spawn((
                    WallCell(cell),
                    maintenance.sprite("wall-depth", 0, Vec2::new(64.0, 88.0)),
                    at(center + Vec2::Y * 12.0, z::STRUCTURE),
                ));
            }
            'V' => {
                if let Some(tile) = map.structure_tile(cell) {
                    commands.spawn((
                        art.tile(tile),
                        at(center + Vec2::Y * depth::WALL_HEIGHT, z::STRUCTURE),
                    ));
                }
            }
            'v' => {
                commands.spawn(prop(Tile::cockpit(map.module_column(cell), 1)));
            }
            'c' => {
                commands.spawn(prop(Tile::PropCrate));
            }
            'l' => {
                commands.spawn(prop(Tile::PropLocker));
            }
            'o' => {
                commands.spawn(prop(Tile::PropOxygenRack));
            }
            'p' => {
                commands.spawn(prop(Tile::PropPipeStack));
            }
            'k' => {
                commands.spawn(prop(Tile::PropControlConsole));
            }
            _ => {}
        }
    }
    for (engine, block) in [
        (EngineBlock::Port, layout::port_engine()),
        (EngineBlock::Starboard, layout::starboard_engine()),
    ] {
        spawn_engine(&mut commands, &maintenance, engine, block);
    }
    let bunk = layout::bunk();
    commands.spawn((
        rect(bunk.size() - 8.0, palette::BUNK),
        at(bunk.center(), z::PROP),
    ));
    let pillow_at = Vec2::new(bunk.min.x + 26.0, bunk.center().y);
    commands.spawn((
        rect(Vec2::new(28.0, bunk.height() - 22.0), palette::PILLOW),
        at(pillow_at, z::PROP + 0.01),
    ));

    for curtain in [Curtain::North, Curtain::East, Curtain::South, Curtain::West] {
        commands.spawn((
            curtain,
            rect(Vec2::splat(CURTAIN_SIZE), palette::VOID),
            at(Vec2::ZERO, z::CURTAIN),
        ));
    }
}

/// Choose an assembly matching the map's engine footprint and orientation.
fn spawn_engine(commands: &mut Commands, art: &MaintenanceArt, engine: EngineBlock, block: Rect) {
    let vertical = block.height() > block.width();
    let (name, size, offset) = if vertical {
        ("engine-vertical", block.size() + Vec2::Y * 16.0, 8.0)
    } else {
        (
            "engine",
            block.size() * Vec2::new(224.0 / 200.0, 88.0 / 56.0),
            18.0,
        )
    };
    commands.spawn((
        engine,
        art.sprite(name, 0, size),
        at(block.center() + Vec2::Y * offset, z::PROP),
    ));
}

fn reset_room(mut room: ResMut<CurrentRoom>) {
    room.0 = RoomId::at(layout::player_spawn());
}

fn track_room(players: Query<&Transform, With<Player>>, mut room: ResMut<CurrentRoom>) {
    if let Some(t) = players.iter().next() {
        let now = RoomId::at(t.translation.truncate());
        if room.0 != now {
            room.0 = now;
        }
    }
}

/// Hard cut to the current room: the camera never shows two rooms.
fn frame_camera(room: Res<CurrentRoom>, mut cameras: Query<(&mut CameraRig, &mut Transform)>) {
    for (mut rig, mut transform) in &mut cameras {
        rig.anchor = room.0.center();
        transform.translation = (rig.anchor + rig.shake).extend(transform.translation.z);
    }
}

/// Covers everything outside the current room's walls.
fn place_curtains(room: Res<CurrentRoom>, mut curtains: Query<(&Curtain, &mut Transform)>) {
    let frame = room_frame(room.0);
    let half = CURTAIN_SIZE / 2.0;
    for (curtain, mut transform) in &mut curtains {
        let pos = match curtain {
            Curtain::North => Vec2::new(frame.center().x, frame.max.y + half),
            Curtain::South => Vec2::new(frame.center().x, frame.min.y - half),
            Curtain::East => Vec2::new(frame.max.x + half, frame.center().y),
            Curtain::West => Vec2::new(frame.min.x - half, frame.center().y),
        };
        transform.translation = pos.extend(transform.translation.z);
    }
}

/// World position under a cursor at `cursor` (logical pixels, origin top
/// left) in a window of `window` logical size, for a camera centered on
/// `anchor`. Matches the camera's `ScalingMode::AutoMin` projection.
pub fn cursor_to_world(cursor: Vec2, window: Vec2, anchor: Vec2) -> Vec2 {
    let units_per_pixel = (layout::VIEW.x / window.x).max(layout::VIEW.y / window.y);
    let offset = (cursor - window / 2.0) * units_per_pixel;
    anchor + Vec2::new(offset.x, -offset.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_at_center_is_the_anchor() {
        let anchor = Vec2::new(100.0, -5.0);
        assert_eq!(
            cursor_to_world(layout::VIEW / 2.0, layout::VIEW, anchor),
            anchor
        );
    }

    #[test]
    fn cursor_top_left_is_up_and_left() {
        let w = cursor_to_world(Vec2::ZERO, layout::VIEW, Vec2::ZERO);
        assert_eq!(w, layout::VIEW * Vec2::new(-0.5, 0.5));
    }

    #[test]
    fn small_window_still_shows_the_reference_view() {
        // Half-size window: each pixel covers two world units.
        let w = cursor_to_world(Vec2::ZERO, layout::VIEW / 2.0, Vec2::ZERO);
        assert_eq!(w, layout::VIEW * Vec2::new(-0.5, 0.5));
    }

    #[test]
    fn wide_window_keeps_the_full_height() {
        let window = Vec2::new(2560.0, 720.0);
        let w = cursor_to_world(Vec2::new(1280.0, 0.0), window, Vec2::ZERO);
        assert_eq!(w, Vec2::new(0.0, layout::VIEW.y / 2.0));
    }

    #[test]
    fn a_room_frame_fits_the_view() {
        for room in RoomId::ALL {
            let f = room_frame(room);
            assert!(
                f.width() <= layout::VIEW.x && f.height() <= layout::VIEW.y,
                "{room:?}"
            );
            assert!(f.contains(room.center()));
        }
    }
}
