//! Sliding doors. A door between two rooms slides open as the engineer
//! comes within [`DOOR_REACH`] (four frames, [`FRAME_SECS`] each) and closes
//! once they have gone. Only a fully open door lets anyone through: until
//! then its leaf is solid. Near a door, [`door_assist`] steers the engineer
//! toward its centre line so they don't catch on the frame.
//!
//! The airlock's outer hatch is a locked door: part of the wall, it never opens.

use bevy::prelude::*;

use super::map::ship;
use crate::art::maintenance::MaintenanceArt;
use crate::art::tiles::{Tile, local_rect};
use crate::art::{Art, TILE};
use crate::player::Player;
use crate::shapes::at;
use crate::{AppState, GameSet, RunSet, running};

/// A door opens when the engineer is this close to its centre. Far enough
/// that a running engineer never waits for it.
pub const DOOR_REACH: f32 = 3.0 * TILE;
/// Seconds per door frame.
pub const FRAME_SECS: f32 = 0.1;
/// The fully open frame (the only passable one).
pub const OPEN_FRAME: usize = 3;
/// The steer assist works this far from the wall line...
pub const ASSIST_REACH: f32 = 1.5 * TILE;
/// ...for an engineer this far off the door's centre line...
pub const ASSIST_WIDTH: f32 = 0.75 * TILE;
/// ...moving them sideways toward it at up to this speed.
pub const ASSIST_SPEED: f32 = 260.0;

/// The visual follows the room's outboard wall; the doorway remains on the map.
#[derive(Component)]
pub struct DoorVisual {
    pub center: Vec2,
    pub across_x: bool,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Door {
    pub center: Vec2,
    /// The wall runs east-west (people pass north-south).
    pub across_x: bool,
    /// 0 (closed) to [`OPEN_FRAME`].
    pub frame: usize,
    /// Seconds into the current frame.
    pub timer: f32,
}

impl Door {
    pub fn new(center: Vec2, across_x: bool) -> Self {
        Self {
            center,
            across_x,
            frame: 0,
            timer: 0.0,
        }
    }

    pub fn is_open(&self) -> bool {
        self.frame == OPEN_FRAME
    }

    /// The solid leaf between the jambs, unless the door is fully open.
    pub fn leaf(&self) -> Option<Rect> {
        (!self.is_open()).then(|| leaf_rect(self.center, self.across_x))
    }

    /// Unit vector across the door (the way people pass through it).
    pub fn across(&self) -> Vec2 {
        if self.across_x { Vec2::Y } else { Vec2::X }
    }
}

/// The gap between a door's jambs (the tiles' 36-pixel clear aperture).
pub fn leaf_rect(center: Vec2, across_x: bool) -> Rect {
    let local = if across_x {
        [14.0, 20.0, 36.0, 24.0]
    } else {
        [20.0, 14.0, 24.0, 36.0]
    };
    local_rect(center, local)
}

/// One tick of a door's animation: a frame every [`FRAME_SECS`], toward
/// open when `want_open`, else toward closed. Returns `(frame, timer)`.
pub fn step_door(frame: usize, timer: f32, want_open: bool, dt: f32) -> (usize, f32) {
    let target = if want_open { OPEN_FRAME } else { 0 };
    let (mut frame, mut timer) = (frame, timer + dt);
    while frame != target && timer >= FRAME_SECS - 1e-4 {
        timer -= FRAME_SECS;
        frame = if target > frame { frame + 1 } else { frame - 1 };
    }
    if frame == target {
        timer = 0.0;
    }
    (frame, timer)
}

/// Sideways correction for an engineer at `pos` heading `dir` near a door:
/// toward the door's centre line when they are close to the wall, heading
/// into it, and not far off to the side. Zero otherwise.
pub fn door_assist(pos: Vec2, dir: Vec2, door: &Door, dt: f32) -> Vec2 {
    let across = door.across();
    let along = across.perp();
    let offset = pos - door.center;
    let (depth, side) = (offset.dot(across), offset.dot(along));
    let heading_in = dir.dot(across) * -depth.signum() > 0.1;
    if !heading_in || depth.abs() > ASSIST_REACH || side.abs() > ASSIST_WIDTH {
        return Vec2::ZERO;
    }
    along * (-side).clamp(-ASSIST_SPEED * dt, ASSIST_SPEED * dt)
}

pub struct DoorsPlugin;

impl Plugin for DoorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_doors.in_set(GameSet::Input))
            .add_systems(
                OnEnter(AppState::Playing),
                close_doors.in_set(RunSet::Spawn),
            )
            .add_systems(
                FixedUpdate,
                operate_doors.in_set(GameSet::Move).run_if(running),
            )
            .add_systems(Update, draw_doors.in_set(GameSet::Present));
    }
}

fn spawn_doors(mut commands: Commands, art: Res<Art>, maintenance: Res<MaintenanceArt>) {
    for spec in ship().doors() {
        let tile = if spec.locked {
            Tile::locked_door(spec.across_x)
        } else {
            Tile::door(spec.across_x, 0)
        };
        let mut door = commands.spawn((
            Name::new("Door"),
            DoorVisual {
                center: spec.center,
                across_x: spec.across_x,
            },
            if spec.across_x {
                maintenance.sprite(
                    "door-depth",
                    if spec.locked { 4 } else { 0 },
                    Vec2::new(64.0, 88.0),
                )
            } else {
                art.tile(tile)
            },
            at(spec.center, super::z::STRUCTURE),
        ));
        if !spec.locked {
            door.insert(Door::new(spec.center, spec.across_x));
        }
    }
}

fn close_doors(mut doors: Query<&mut Door>) {
    for mut door in &mut doors {
        door.frame = 0;
        door.timer = 0.0;
    }
}

/// Opens doors near the engineer, closes the rest.
fn operate_doors(
    time: Res<Time>,
    players: Query<&Transform, With<Player>>,
    mut doors: Query<&mut Door>,
) {
    let player = players.iter().next().map(|t| t.translation.truncate());
    for mut door in &mut doors {
        let want_open = player.is_some_and(|p| p.distance(door.center) <= DOOR_REACH);
        let (frame, timer) = step_door(door.frame, door.timer, want_open, time.delta_secs());
        if (door.frame, door.timer) != (frame, timer) {
            door.frame = frame;
            door.timer = timer;
        }
    }
}

fn draw_doors(art: Res<MaintenanceArt>, mut doors: Query<(&Door, &mut Sprite)>) {
    for (door, mut sprite) in &mut doors {
        if door.across_x {
            art.frame(&mut sprite, "door-depth", door.frame);
            continue;
        }
        let index = Tile::door(door.across_x, door.frame).index();
        if let Some(atlas) = sprite.texture_atlas.as_mut()
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn a_door_opens_in_three_frames_and_closes_the_same_way() {
        let (mut frame, mut timer) = (0, 0.0);
        let mut ticks = 0;
        while frame != OPEN_FRAME {
            (frame, timer) = step_door(frame, timer, true, DT);
            ticks += 1;
            assert!(ticks < 100);
        }
        assert_eq!(ticks, 18, "0.3 s at 60 ticks per second");
        (frame, timer) = step_door(frame, timer, true, DT);
        assert_eq!((frame, timer), (OPEN_FRAME, 0.0), "stays open");
        for _ in 0..6 {
            (frame, timer) = step_door(frame, timer, false, DT);
        }
        assert_eq!(frame, 2, "closing reverses the frames");
    }

    #[test]
    fn only_a_fully_open_door_is_passable() {
        let mut door = Door::new(Vec2::ZERO, true);
        for frame in 0..OPEN_FRAME {
            door.frame = frame;
            assert!(door.leaf().is_some());
        }
        door.frame = OPEN_FRAME;
        assert_eq!(door.leaf(), None);
        let leaf = leaf_rect(Vec2::ZERO, true);
        assert_eq!((leaf.width(), leaf.height()), (36.0, 24.0));
    }

    #[test]
    fn a_running_engineer_never_waits_for_a_door() {
        // Straight at a closed door from the edge of its reach: it is fully
        // open before the engineer's leading edge reaches the leaf, even
        // with every pick of the campaign spent on running faster.
        let fastest = crate::upgrades::Upgrades::all_of(
            crate::upgrades::Upgrade::RunFaster,
            crate::upgrades::MAX_PICKS,
        );
        let speed = crate::player::PLAYER_SPEED * fastest.speed_factor();
        let radius = crate::player::PLAYER_RADIUS;
        let travel = DOOR_REACH - radius - 12.0;
        let opens_in = OPEN_FRAME as f32 * FRAME_SECS;
        assert!(
            travel / speed > opens_in + 2.0 * DT,
            "{}s < {opens_in}s",
            travel / speed
        );
    }

    #[test]
    fn assist_steers_toward_the_centre_line_only_when_heading_in() {
        let door = Door::new(Vec2::ZERO, true);
        // Below the door, 20 units to its right, walking up into it.
        let nudge = door_assist(Vec2::new(20.0, -50.0), Vec2::Y, &door, DT);
        assert!(nudge.x < 0.0 && nudge.y == 0.0, "{nudge}");
        assert!(nudge.length() <= ASSIST_SPEED * DT + 1e-4);
        // Walking away, too far off to the side, or too far from the wall: nothing.
        assert_eq!(
            door_assist(Vec2::new(20.0, -50.0), Vec2::NEG_Y, &door, DT),
            Vec2::ZERO
        );
        assert_eq!(
            door_assist(Vec2::new(90.0, -50.0), Vec2::Y, &door, DT),
            Vec2::ZERO
        );
        assert_eq!(
            door_assist(Vec2::new(20.0, -200.0), Vec2::Y, &door, DT),
            Vec2::ZERO
        );
        // Never overshoots the centre line.
        let close = door_assist(Vec2::new(1.0, -50.0), Vec2::Y, &door, DT);
        assert!((close.x + 1.0).abs() < 1e-5, "{close}");
    }
}
