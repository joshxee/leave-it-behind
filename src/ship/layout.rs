//! Ship geometry as plain data: rooms along the corridor spine, walls with
//! door gaps, solid props, and the circle-vs-rectangle collision the player
//! uses. No ECS here, so all of it is unit tested directly.
//!
//! World units are logical pixels at the 1280×720 reference view. The ship
//! points right: the cockpit is the front (+x), the airlock the tail (−x).
//! The spine is the line y = 0 through every door.

use bevy::prelude::*;

/// Wall thickness.
pub const WALL: f32 = 20.0;
/// Half the interior height of every room.
pub const HALF_HEIGHT: f32 = 260.0;
/// Half the height of the door gap in each bulkhead, centered on the spine.
pub const DOOR_HALF: f32 = 60.0;
/// Smallest area the camera always shows (the reference window size).
pub const VIEW: Vec2 = Vec2::new(1280.0, 720.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomId {
    Cockpit,
    Quarters,
    Engine,
    Hull,
    Airlock,
}

/// Rooms in spatial order, tail (−x) to front (+x).
const TAIL_TO_FRONT: [RoomId; 5] = [
    RoomId::Airlock,
    RoomId::Hull,
    RoomId::Engine,
    RoomId::Quarters,
    RoomId::Cockpit,
];

impl RoomId {
    /// Front to tail.
    pub const ALL: [RoomId; 5] = [
        RoomId::Cockpit,
        RoomId::Quarters,
        RoomId::Engine,
        RoomId::Hull,
        RoomId::Airlock,
    ];

    /// Position in [`RoomId::ALL`] (0 = cockpit).
    pub fn index(self) -> usize {
        self as usize
    }

    /// Stable identifier (test bridge, logs).
    pub fn as_str(self) -> &'static str {
        match self {
            RoomId::Cockpit => "Cockpit",
            RoomId::Quarters => "Quarters",
            RoomId::Engine => "Engine",
            RoomId::Hull => "Hull",
            RoomId::Airlock => "Airlock",
        }
    }

    /// Player-facing name.
    pub fn name(self) -> &'static str {
        match self {
            RoomId::Cockpit => "Cockpit",
            RoomId::Quarters => "Engineer's quarters",
            RoomId::Engine => "Engine room",
            RoomId::Hull => "Main hull",
            RoomId::Airlock => "Airlock",
        }
    }

    /// Interior width along the spine.
    pub fn width(self) -> f32 {
        match self {
            RoomId::Cockpit => 700.0,
            RoomId::Quarters => 800.0,
            RoomId::Engine => 1000.0,
            RoomId::Hull => 1100.0,
            RoomId::Airlock => 700.0,
        }
    }

    /// Walkable interior (inside the walls).
    pub fn interior(self) -> Rect {
        let mut x = -ship_length() / 2.0;
        for room in TAIL_TO_FRONT {
            if room == self {
                return Rect::new(x, -HALF_HEIGHT, x + room.width(), HALF_HEIGHT);
            }
            x += room.width() + WALL;
        }
        unreachable!("every room is in TAIL_TO_FRONT")
    }

    /// Where the camera looks while the player is in this room.
    pub fn center(self) -> Vec2 {
        self.interior().center()
    }

    /// The room owning world x: rooms meet at the middle of each bulkhead.
    pub fn at(x: f32) -> RoomId {
        for pair in TAIL_TO_FRONT.windows(2) {
            if x < pair[0].interior().max.x + WALL / 2.0 {
                return pair[0];
            }
        }
        RoomId::Cockpit
    }
}

/// Interior length of the ship from the tail wall to the front wall.
pub fn ship_length() -> f32 {
    TAIL_TO_FRONT.iter().map(|r| r.width()).sum::<f32>() + WALL * (TAIL_TO_FRONT.len() - 1) as f32
}

/// Outer walls and bulkheads (with door gaps). Tape sticks to these.
pub fn walls() -> Vec<Rect> {
    let half = ship_length() / 2.0;
    let (min, max) = (
        Vec2::new(-half - WALL, -HALF_HEIGHT - WALL),
        Vec2::new(half + WALL, HALF_HEIGHT + WALL),
    );
    let mut walls = vec![
        Rect::new(min.x, HALF_HEIGHT, max.x, max.y),
        Rect::new(min.x, min.y, max.x, -HALF_HEIGHT),
        Rect::new(min.x, min.y, -half, max.y),
        Rect::new(half, min.y, max.x, max.y),
    ];
    for pair in TAIL_TO_FRONT.windows(2) {
        let x0 = pair[0].interior().max.x;
        walls.push(Rect::new(x0, DOOR_HALF, x0 + WALL, HALF_HEIGHT));
        walls.push(Rect::new(x0, -HALF_HEIGHT, x0 + WALL, -DOOR_HALF));
    }
    walls
}

/// Solid furniture the player walks around.
pub fn props() -> Vec<Rect> {
    let mut props = vec![
        port_engine(),
        starboard_engine(),
        diag_console(),
        bunk(),
        Rect::from_center_size(joystick(), Vec2::splat(24.0)),
    ];
    props.extend(crates());
    props
}

/// Everything the player collides with.
pub fn colliders() -> Vec<Rect> {
    let mut all = walls();
    all.extend(props());
    all
}

/// Where the player starts each run: the engineer's quarters, on the spine.
pub fn player_spawn() -> Vec2 {
    Vec2::new(RoomId::Quarters.center().x - 90.0, 0.0)
}

pub const ENGINE_SIZE: Vec2 = Vec2::new(400.0, 110.0);
/// Distance from the spine to each engine block's center.
const ENGINE_OFFSET: f32 = 145.0;

/// Upper engine block (port side when the ship points right).
pub fn port_engine() -> Rect {
    Rect::from_center_size(
        Vec2::new(RoomId::Engine.center().x, ENGINE_OFFSET),
        ENGINE_SIZE,
    )
}

/// Lower engine block.
pub fn starboard_engine() -> Rect {
    Rect::from_center_size(
        Vec2::new(RoomId::Engine.center().x, -ENGINE_OFFSET),
        ENGINE_SIZE,
    )
}

/// Where the player stands (locked) while at the helm.
pub fn helm_seat() -> Vec2 {
    Vec2::new(RoomId::Cockpit.interior().min.x + 415.0, 0.0)
}

/// The nav joystick: pressing E near it takes the helm.
pub fn joystick() -> Vec2 {
    helm_seat() + Vec2::new(40.0, 0.0)
}

/// The nav display in front of the helm (not solid: it is a floor display).
pub fn nav_screen() -> Rect {
    Rect::from_center_size(
        Vec2::new(RoomId::Cockpit.interior().max.x - 115.0, 0.0),
        Vec2::splat(190.0),
    )
}

/// The diagnostic console against the quarters' upper wall.
pub fn diag_console() -> Rect {
    let top = RoomId::Quarters.interior().max.y;
    Rect::from_center_size(
        Vec2::new(RoomId::Quarters.center().x, top - 22.5),
        Vec2::new(140.0, 45.0),
    )
}

/// The engineer's bunk, lower left of the quarters.
pub fn bunk() -> Rect {
    let room = RoomId::Quarters.interior();
    Rect::new(
        room.min.x + 20.0,
        room.min.y + 5.0,
        room.min.x + 180.0,
        room.min.y + 75.0,
    )
}

/// Cargo in the main hull, clear of the spine and the walls.
pub fn crates() -> [Rect; 2] {
    let c = RoomId::Hull.center();
    [
        Rect::from_center_size(c + Vec2::new(-230.0, 110.0), Vec2::new(110.0, 90.0)),
        Rect::from_center_size(c + Vec2::new(220.0, -120.0), Vec2::new(130.0, 90.0)),
    ]
}

/// The outer airlock hatch on the tail wall (decoration; spacewalks are out of scope).
pub fn hatch() -> Rect {
    let x = -ship_length() / 2.0;
    Rect::new(x - WALL, -70.0, x, 70.0)
}

/// The cockpit window in the front wall (decoration).
pub fn window() -> Rect {
    let x = ship_length() / 2.0;
    Rect::new(x, -200.0, x + WALL, 200.0)
}

/// Pushes a circle out of every rectangle it overlaps. A few passes settle
/// corners where two rectangles meet.
pub fn resolve_circle(mut p: Vec2, radius: f32, rects: &[Rect]) -> Vec2 {
    for _ in 0..4 {
        let mut moved = false;
        for r in rects {
            let closest = p.clamp(r.min, r.max);
            let d = p - closest;
            let dist2 = d.length_squared();
            if dist2 >= radius * radius {
                continue;
            }
            moved = true;
            if dist2 > 1e-8 {
                p = closest + d / dist2.sqrt() * radius;
            } else {
                // Center inside the rectangle: leave through the nearest side.
                let sides = [
                    (p.x - r.min.x, Vec2::new(r.min.x - radius, p.y)),
                    (r.max.x - p.x, Vec2::new(r.max.x + radius, p.y)),
                    (p.y - r.min.y, Vec2::new(p.x, r.min.y - radius)),
                    (r.max.y - p.y, Vec2::new(p.x, r.max.y + radius)),
                ];
                p = sides
                    .iter()
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|s| s.1)
                    .unwrap_or(p);
            }
        }
        if !moved {
            break;
        }
    }
    p
}

/// Moves a circle by `delta`, sliding along anything solid.
pub fn move_circle(pos: Vec2, delta: Vec2, radius: f32, rects: &[Rect]) -> Vec2 {
    resolve_circle(pos + delta, radius, rects)
}

/// A point on a wall surface and the surface normal (pointing out of the wall).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallContact {
    pub point: Vec2,
    pub normal: Vec2,
}

/// The wall surface point nearest `p`, if within `max_dist`. A point inside
/// a wall snaps to that wall's nearest face.
pub fn wall_contact(p: Vec2, max_dist: f32, walls: &[Rect]) -> Option<WallContact> {
    let mut best: Option<(f32, WallContact)> = None;
    for r in walls {
        let closest = p.clamp(r.min, r.max);
        let d = p - closest;
        let (dist, contact) = if d.length_squared() > 1e-8 {
            let dist = d.length();
            (
                dist,
                WallContact {
                    point: closest,
                    normal: d / dist,
                },
            )
        } else {
            let faces = [
                (p.x - r.min.x, Vec2::new(r.min.x, p.y), Vec2::NEG_X),
                (r.max.x - p.x, Vec2::new(r.max.x, p.y), Vec2::X),
                (p.y - r.min.y, Vec2::new(p.x, r.min.y), Vec2::NEG_Y),
                (r.max.y - p.y, Vec2::new(p.x, r.max.y), Vec2::Y),
            ];
            let face = faces
                .iter()
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .expect("four faces");
            (
                0.0,
                WallContact {
                    point: face.1,
                    normal: face.2,
                },
            )
        };
        if dist <= max_dist && best.is_none_or(|(b, _)| dist < b) {
            best = Some((dist, contact));
        }
    }
    best.map(|(_, c)| c)
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f32 = 18.0;

    #[test]
    fn rooms_run_front_to_tail_without_overlap() {
        let mut last_min = f32::INFINITY;
        for room in RoomId::ALL {
            let i = room.interior();
            assert!(i.max.x + WALL <= last_min + 1e-3, "{room:?} overlaps");
            last_min = i.min.x;
            assert!(
                i.width() + 2.0 * WALL <= VIEW.x,
                "{room:?} wider than the view"
            );
            assert_eq!(RoomId::at(i.center().x), room);
        }
    }

    #[test]
    fn room_boundary_is_the_middle_of_the_bulkhead() {
        let q = RoomId::Quarters.interior();
        assert_eq!(RoomId::at(q.max.x + WALL / 2.0 - 0.1), RoomId::Quarters);
        assert_eq!(RoomId::at(q.max.x + WALL / 2.0 + 0.1), RoomId::Cockpit);
        assert_eq!(RoomId::at(-10_000.0), RoomId::Airlock);
        assert_eq!(RoomId::at(10_000.0), RoomId::Cockpit);
    }

    #[test]
    fn spine_is_clear_from_the_helm_to_the_tail() {
        let colliders = colliders();
        let mut x = -ship_length() / 2.0 + R;
        while x <= helm_seat().x {
            let p = Vec2::new(x, 0.0);
            assert_eq!(resolve_circle(p, R, &colliders), p, "blocked at x = {x}");
            x += 5.0;
        }
    }

    #[test]
    fn spawn_is_free_and_in_the_quarters() {
        let spawn = player_spawn();
        assert_eq!(RoomId::at(spawn.x), RoomId::Quarters);
        assert_eq!(resolve_circle(spawn, R, &colliders()), spawn);
    }

    #[test]
    fn walls_block_outside_the_door() {
        let walls = walls();
        let bulkhead_x = RoomId::Quarters.interior().max.x + WALL / 2.0;
        let mut p = Vec2::new(bulkhead_x - 40.0, 150.0);
        for _ in 0..60 {
            p = move_circle(p, Vec2::new(7.0, 0.0), R, &walls);
        }
        assert!(
            p.x < bulkhead_x - WALL / 2.0,
            "went through the bulkhead: {p}"
        );
    }

    #[test]
    fn sliding_along_a_wall_keeps_the_parallel_motion() {
        let walls = walls();
        let start = Vec2::new(0.0, HALF_HEIGHT - R);
        let end = move_circle(start, Vec2::new(5.0, 5.0), R, &walls);
        assert!(
            (end.x - 5.0).abs() < 1e-3 && (end.y - start.y).abs() < 1e-3,
            "{end}"
        );
    }

    #[test]
    fn the_helm_seat_and_console_approach_are_reachable() {
        let colliders = colliders();
        let seat = helm_seat();
        assert_eq!(resolve_circle(seat, R, &colliders), seat);
        let below_console = Vec2::new(diag_console().center().x, diag_console().min.y - R - 5.0);
        assert_eq!(resolve_circle(below_console, R, &colliders), below_console);
    }

    #[test]
    fn engine_gaps_fit_the_player() {
        let colliders = colliders();
        let gap = Vec2::new(
            port_engine().center().x,
            (port_engine().max.y + HALF_HEIGHT) / 2.0,
        );
        assert_eq!(resolve_circle(gap, R, &colliders), gap);
    }

    #[test]
    fn contact_inside_a_wall_snaps_to_the_inner_face() {
        let c = wall_contact(Vec2::new(0.0, HALF_HEIGHT + 4.0), 16.0, &walls()).unwrap();
        assert_eq!(c.point, Vec2::new(0.0, HALF_HEIGHT));
        assert_eq!(c.normal, Vec2::NEG_Y);
    }

    #[test]
    fn no_contact_in_the_middle_of_a_room() {
        assert!(wall_contact(RoomId::Hull.center(), 16.0, &walls()).is_none());
    }
}
