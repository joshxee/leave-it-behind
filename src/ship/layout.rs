//! Ship geometry the game asks for, derived from the map (`map.rs`), and the
//! circle-vs-rectangle collision the player uses. No ECS here, so all of it
//! is unit tested directly.
//!
//! World units are logical pixels at the 1280×720 reference view; one map
//! cell is 64 units, y is up. Nothing here assumes which way the ship points.

use bevy::prelude::*;

pub use super::map::{RoomId, WALL_HALF, ship};

/// Smallest area the camera always shows (the reference window size).
pub const VIEW: Vec2 = Vec2::new(1408.0, 792.0);

/// Walls, locked doors and door jambs. Tape sticks to these.
pub fn walls() -> Vec<Rect> {
    ship().walls().to_vec()
}

/// Solid furniture the player walks around.
pub fn props() -> Vec<Rect> {
    ship().props().to_vec()
}

/// Everything static the player collides with. Door leaves move, so the
/// player adds the closed ones itself (`ship::doors`).
pub fn colliders() -> Vec<Rect> {
    let mut all = walls();
    all.extend(props());
    all
}

/// Where the player starts each run.
pub fn player_spawn() -> Vec2 {
    ship().spawn()
}

/// Where the player sits (locked) while at the helm.
pub fn helm_seat() -> Vec2 {
    ship().helm().seat
}

/// The nav joystick in front of the seat: E near the seat takes the helm.
pub fn joystick() -> Vec2 {
    ship().helm().joystick
}

/// The nav display (not solid: it is on the floor).
pub fn nav_screen() -> Rect {
    ship().block('N').expect("the map has a nav display")
}

/// Engine blocks (port and starboard, as marked in the map).
pub fn port_engine() -> Rect {
    ship().block('P').expect("the map has a port engine")
}

pub fn starboard_engine() -> Rect {
    ship().block('S').expect("the map has a starboard engine")
}

/// The engineer's bunk.
pub fn bunk() -> Rect {
    ship().block('b').expect("the map has a bunk")
}

/// Where E works the diagnostic console (its front edge).
pub fn console_point() -> Vec2 {
    ship().console_point()
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

/// Where a tool held out from `from` (a point in the open: the engineer's
/// feet) to `tip` touches a wall. A tip pressed into a wall, or through it,
/// touches where the reach first meets the wall: always the face on the
/// holder's side, never the outside of the hull or the next room's side. A
/// tip in the open touches the nearest wall surface within `max_dist`.
pub fn wall_contact(from: Vec2, tip: Vec2, max_dist: f32, walls: &[Rect]) -> Option<WallContact> {
    let first = walls
        .iter()
        .filter_map(|r| entry(from, tip, *r))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    if let Some((t, normal)) = first {
        return Some(WallContact {
            point: from.lerp(tip, t),
            normal,
        });
    }
    let mut best: Option<(f32, WallContact)> = None;
    for r in walls {
        let closest = tip.clamp(r.min, r.max);
        let d = tip - closest;
        let dist = d.length();
        // A tip in a wall the reach never entered: the holder is in it too,
        // so there is no side to touch.
        if dist > 1e-6 && dist <= max_dist && best.is_none_or(|(b, _)| dist < b) {
            best = Some((
                dist,
                WallContact {
                    point: closest,
                    normal: d / dist,
                },
            ));
        }
    }
    best.map(|(_, c)| c)
}

/// How far along the segment from `a` to `b` (0 to 1) it first enters `r`,
/// and the normal of the face it enters through. `None` if it misses `r`,
/// stops short of it or starts inside it.
fn entry(a: Vec2, b: Vec2, r: Rect) -> Option<(f32, Vec2)> {
    let mut enter = (f32::NEG_INFINITY, Vec2::ZERO);
    let mut leave = f32::INFINITY;
    for axis in [Vec2::X, Vec2::Y] {
        let (start, step) = (a.dot(axis), (b - a).dot(axis));
        let (lo, hi) = (r.min.dot(axis), r.max.dot(axis));
        if step.abs() < 1e-6 {
            // Parallel to these faces: it has to run between them.
            if start < lo || start > hi {
                return None;
            }
            continue;
        }
        let (near, far, normal) = if step > 0.0 {
            ((lo - start) / step, (hi - start) / step, -axis)
        } else {
            ((hi - start) / step, (lo - start) / step, axis)
        };
        if near > enter.0 {
            enter = (near, normal);
        }
        leave = leave.min(far);
    }
    let (t, normal) = enter;
    // A tip resting exactly on the face still touches it.
    ((0.0..=1.0 + 1e-4).contains(&t) && t <= leave).then_some((t, normal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::map::Kind;

    const R: f32 = 14.0;

    /// The path through every door on the spine, front to tail: the helm
    /// seat, each door in a wall across the ship (like the hatch's) in order
    /// of distance from it, then just inside the locked hatch. Side doors
    /// into the rooms beside the corridors are off it.
    fn spine() -> Vec<Vec2> {
        let seat = helm_seat();
        let hatch = ship()
            .doors()
            .into_iter()
            .find(|d| d.locked)
            .expect("a hatch");
        let mut doors = ship().doors();
        doors.retain(|d| d.across_x == hatch.across_x);
        doors.sort_by(|a, b| a.center.distance(seat).total_cmp(&b.center.distance(seat)));
        let mut points = vec![seat];
        for door in &doors {
            if door.locked {
                let steps = if door.across_x {
                    [IVec2::new(0, -1), IVec2::new(0, 1)]
                } else {
                    [IVec2::new(-1, 0), IVec2::new(1, 0)]
                };
                let step = steps
                    .into_iter()
                    .find(|s| ship().kind_at(door.cell + *s) == Kind::Floor)
                    .expect("a room in front of the hatch");
                let inward = Vec2::new(step.x as f32, -step.y as f32);
                points.push(door.center + inward * (WALL_HALF + R + 1.0));
            } else {
                points.push(door.center);
            }
        }
        points
    }

    #[test]
    fn the_spine_is_clear_from_the_helm_to_the_tail() {
        let colliders = colliders();
        for pair in spine().windows(2) {
            let steps = (pair[0].distance(pair[1]) / 5.0).ceil() as usize;
            for i in 0..=steps {
                let p = pair[0].lerp(pair[1], i as f32 / steps.max(1) as f32);
                assert_eq!(resolve_circle(p, R, &colliders), p, "blocked at {p}");
            }
        }
    }

    #[test]
    fn spawn_is_free_and_in_the_quarters() {
        let spawn = player_spawn();
        assert_eq!(RoomId::at(spawn), RoomId::Quarters);
        assert_eq!(resolve_circle(spawn, R, &colliders()), spawn);
    }

    #[test]
    fn walls_block_beside_a_door() {
        let walls = walls();
        for door in ship().doors() {
            let across = if door.across_x { Vec2::Y } else { Vec2::X };
            let along = across.perp();
            // One cell along the wall from the door, walking straight at it.
            let mut p = door.center + along * 64.0 + across * 50.0;
            for _ in 0..60 {
                p = move_circle(p, -across * 7.0, R, &walls);
            }
            let gone_through = (p - door.center).dot(across);
            assert!(
                gone_through > WALL_HALF,
                "went through the wall at {door:?}: {p}"
            );
        }
    }

    #[test]
    fn sliding_along_a_wall_keeps_the_parallel_motion() {
        let walls = walls();
        let hull = RoomId::Hull.interior();
        let start = Vec2::new(hull.center().x + 200.0, hull.max.y - R);
        let end = move_circle(start, Vec2::new(5.0, 5.0), R, &walls);
        assert!(
            (end.x - start.x - 5.0).abs() < 1e-3 && (end.y - start.y).abs() < 1e-3,
            "{end}"
        );
    }

    #[test]
    fn the_helm_seat_and_console_approach_are_reachable() {
        let colliders = colliders();
        let seat = helm_seat();
        assert_eq!(resolve_circle(seat, R, &colliders), seat);
        let stand = console_point() - Vec2::Y * (R + 5.0);
        assert_eq!(resolve_circle(stand, R, &colliders), stand);
    }

    #[test]
    fn there_is_room_to_walk_round_the_engines() {
        let colliders = colliders();
        for engine in [port_engine(), starboard_engine()] {
            let room = RoomId::at(engine.center()).interior();
            let c = engine.center();
            for p in [
                Vec2::new(c.x, (engine.max.y + room.max.y) / 2.0),
                Vec2::new(c.x, (engine.min.y + room.min.y) / 2.0),
                Vec2::new(engine.min.x - 2.0 * R, c.y),
                Vec2::new(engine.max.x + 2.0 * R, c.y),
            ] {
                assert_eq!(resolve_circle(p, R, &colliders), p, "{engine:?} at {p}");
            }
        }
    }

    #[test]
    fn a_tip_short_of_a_wall_touches_its_face_within_reach() {
        let mark = ship().wall_mark('7').expect("a breach point");
        let from = mark.point + mark.normal * 40.0;
        let c = wall_contact(from, mark.point + mark.normal * 10.0, 16.0, &walls()).unwrap();
        assert!(c.point.distance(mark.point) < 1e-3, "{c:?}");
        assert_eq!(c.normal, mark.normal);
        assert!(wall_contact(from, mark.point + mark.normal * 20.0, 16.0, &walls()).is_none());
    }

    #[test]
    fn a_reach_into_the_hull_never_touches_its_outside() {
        let walls = walls();
        // Breach points sit on hull walls: space is on the far side.
        for mark in crate::ship::map::BREACH_MARKS
            .into_iter()
            .filter_map(|m| ship().wall_mark(m))
        {
            let from = mark.point + mark.normal * (R + 10.0);
            let along = mark.normal.perp();
            // Past the wall's centre line (nearer the outside), and through it.
            for depth in [0.0, 4.0, 16.0, 23.0, 40.0] {
                for lean in [-15.0, 0.0, 15.0] {
                    let tip = mark.point - mark.normal * depth + along * lean;
                    let c = wall_contact(from, tip, 14.0, &walls).expect("touches the wall");
                    assert_eq!(c.normal, mark.normal, "{mark:?} {depth} {lean}");
                    assert!(
                        (c.point - mark.point).dot(mark.normal).abs() < 1e-3,
                        "{mark:?} {depth} {lean}: {c:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_reach_into_a_wall_between_rooms_stays_on_its_side() {
        let walls = walls();
        for door in ship().doors().into_iter().filter(|d| !d.locked) {
            let across = if door.across_x { Vec2::Y } else { Vec2::X };
            // One cell along the wall from the door, from either room.
            let wall = door.center + across.perp() * 64.0;
            for side in [across, -across] {
                let from = wall + side * (WALL_HALF + R + 10.0);
                let tip = wall - side * (WALL_HALF - 4.0);
                let c = wall_contact(from, tip, 14.0, &walls).expect("touches the wall");
                assert_eq!(c.normal, side, "{door:?}");
                assert!(
                    ((c.point - wall).dot(side) - WALL_HALF).abs() < 1e-3,
                    "{c:?}"
                );
            }
        }
    }

    #[test]
    fn no_contact_in_the_middle_of_a_room() {
        let c = RoomId::Hull.center();
        assert!(wall_contact(c, c + Vec2::new(30.0, 0.0), 16.0, &walls()).is_none());
    }
}
