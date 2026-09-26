//! The ship: five rooms along a corridor spine, their walls and props, the
//! room the player is in, and a camera that frames only that room.

pub mod layout;

use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub use layout::RoomId;

use crate::player::Player;
use crate::shapes::{at, rect};
use crate::{AppState, GameSet, RunSet, palette};

/// The room the player occupies. The camera shows only this room.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentRoom(pub RoomId);

/// Everything solid: walls and props.
#[derive(Resource, Debug, Clone)]
pub struct Colliders(pub Vec<Rect>);

/// Walls only (outer hull and bulkheads). Tape sticks to these.
#[derive(Resource, Debug, Clone)]
pub struct Walls(pub Vec<Rect>);

/// The game camera. `anchor` is the current room's center; the alarm adds
/// `shake` on top. Aiming uses `anchor`, so shake never moves the aim.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CameraRig {
    pub anchor: Vec2,
    pub shake: Vec2,
}

/// An engine block. Tinted by the loose-bolts fault as the engine heats up.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineBlock {
    Port,
    Starboard,
}

/// Void-colored sprites that hide the neighbouring rooms.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Curtain {
    Tail,
    Front,
}

const CURTAIN_WIDTH: f32 = 4000.0;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentRoom(RoomId::at(layout::player_spawn().x)))
            .insert_resource(Colliders(layout::colliders()))
            .insert_resource(Walls(layout::walls()))
            .insert_resource(ClearColor(palette::VOID))
            .add_systems(Startup, (spawn_camera, spawn_ship).in_set(GameSet::Input))
            .add_systems(OnEnter(AppState::Playing), reset_room.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                track_room
                    .in_set(GameSet::Act)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (frame_camera, place_curtains).in_set(GameSet::Present),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    let anchor = RoomId::at(layout::player_spawn().x).center();
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: layout::VIEW.x,
                min_height: layout::VIEW.y,
            },
            ..OrthographicProjection::default_2d()
        }),
        // Flat axis-aligned shapes and pre-antialiased circle textures gain
        // little from MSAA, and it quadruples fill cost on software renderers.
        Msaa::Off,
        CameraRig {
            anchor,
            shake: Vec2::ZERO,
        },
        at(anchor, 0.0),
    ));
}

fn spawn_ship(mut commands: Commands) {
    let rect_at = |r: Rect, color: Color, z: f32| (rect(r.size(), color), at(r.center(), z));

    for room in RoomId::ALL {
        commands.spawn((
            Name::new(room.name()),
            rect_at(room.interior(), palette::FLOORS[room.index()], 0.0),
        ));
    }
    let spine = Rect::from_center_size(Vec2::ZERO, Vec2::new(layout::ship_length(), 80.0));
    commands.spawn(rect_at(spine, palette::SPINE, 0.1));

    for wall in layout::walls() {
        commands.spawn(rect_at(wall, palette::WALL, 1.0));
    }
    // Yellow frames at both edges of every door.
    for room in RoomId::ALL.iter().skip(1) {
        let x = room.interior().max.x + layout::WALL / 2.0;
        for y in [layout::DOOR_HALF, -layout::DOOR_HALF] {
            let frame = Rect::from_center_size(Vec2::new(x, y), Vec2::new(layout::WALL + 8.0, 6.0));
            commands.spawn(rect_at(frame, palette::DOOR_FRAME, 1.1));
        }
    }
    commands.spawn(rect_at(layout::window(), palette::WINDOW, 1.05));
    commands.spawn(rect_at(layout::hatch(), palette::HATCH, 1.05));

    commands.spawn((
        EngineBlock::Port,
        rect_at(layout::port_engine(), palette::ENGINE, 1.2),
    ));
    commands.spawn((
        EngineBlock::Starboard,
        rect_at(layout::starboard_engine(), palette::ENGINE, 1.2),
    ));
    for c in layout::crates() {
        commands.spawn(rect_at(c, palette::CRATE, 1.2));
    }
    let bunk = layout::bunk();
    commands.spawn(rect_at(bunk, palette::BUNK, 1.2));
    let pillow = Rect::from_center_size(
        Vec2::new(bunk.min.x + 22.0, bunk.center().y),
        Vec2::new(26.0, bunk.height() - 16.0),
    );
    commands.spawn(rect_at(pillow, palette::UI_DIM, 1.25));
    commands.spawn(rect_at(layout::diag_console(), palette::PROP, 1.2));

    for curtain in [Curtain::Tail, Curtain::Front] {
        commands.spawn((
            curtain,
            rect(Vec2::new(CURTAIN_WIDTH, 4000.0), palette::VOID),
            at(Vec2::ZERO, 10.0),
        ));
    }
}

fn reset_room(mut room: ResMut<CurrentRoom>) {
    room.0 = RoomId::at(layout::player_spawn().x);
}

fn track_room(players: Query<&Transform, With<Player>>, mut room: ResMut<CurrentRoom>) {
    if let Some(t) = players.iter().next() {
        let now = RoomId::at(t.translation.x);
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

/// Covers everything beyond the current room's outer wall faces.
fn place_curtains(room: Res<CurrentRoom>, mut curtains: Query<(&Curtain, &mut Transform)>) {
    let interior = room.0.interior();
    for (curtain, mut transform) in &mut curtains {
        let x = match curtain {
            Curtain::Tail => interior.min.x - layout::WALL - CURTAIN_WIDTH / 2.0,
            Curtain::Front => interior.max.x + layout::WALL + CURTAIN_WIDTH / 2.0,
        };
        transform.translation.x = x;
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
            cursor_to_world(Vec2::new(640.0, 360.0), layout::VIEW, anchor),
            anchor
        );
    }

    #[test]
    fn cursor_top_left_is_up_and_left() {
        let w = cursor_to_world(Vec2::ZERO, layout::VIEW, Vec2::ZERO);
        assert_eq!(w, Vec2::new(-640.0, 360.0));
    }

    #[test]
    fn small_window_still_shows_the_reference_view() {
        // Half-size window: each pixel covers two world units.
        let w = cursor_to_world(Vec2::ZERO, layout::VIEW / 2.0, Vec2::ZERO);
        assert_eq!(w, Vec2::new(-640.0, 360.0));
    }

    #[test]
    fn wide_window_keeps_the_full_height() {
        let window = Vec2::new(2560.0, 720.0);
        let w = cursor_to_world(Vec2::new(1280.0, 0.0), window, Vec2::ZERO);
        assert_eq!(w, Vec2::new(0.0, 360.0));
    }
}
