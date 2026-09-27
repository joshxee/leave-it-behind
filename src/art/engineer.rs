//! The engineer sprite pack: atlas layout, animation actions, facing
//! directions and where each held tool touches things. Values come from
//! `assets/characters/engineer/manifest.json` (checked by a unit test).

use bevy::prelude::*;

/// Atlas grid: 16 columns of 64-pixel cells.
pub const COLUMNS: u32 = 16;
pub const ROWS: u32 = 12;
pub const CELL: u32 = 64;
/// The standing ground point, in pixels from the image's top left.
pub const ROOT: Vec2 = Vec2::new(32.0, 38.0);
/// `ROOT` as a normalized sprite anchor (Bevy anchors are positive upward).
pub const ANCHOR: Vec2 = Vec2::new(0.0, -6.0 / 64.0);

/// One animation of the pack. Tools are always in hand except `Idle` and
/// `Walk` (empty-handed, used at the helm).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Idle,
    Walk,
    WrenchHold,
    WrenchUse,
    WrenchWalk,
    TapeHold,
    TapeUse,
    TapeWalk,
}

impl Action {
    pub const ALL: [Action; 8] = [
        Action::Idle,
        Action::Walk,
        Action::WrenchHold,
        Action::WrenchUse,
        Action::WrenchWalk,
        Action::TapeHold,
        Action::TapeUse,
        Action::TapeWalk,
    ];

    /// The manifest's action name.
    pub fn as_str(self) -> &'static str {
        match self {
            Action::Idle => "idle",
            Action::Walk => "walk",
            Action::WrenchHold => "wrench_hold",
            Action::WrenchUse => "wrench_use",
            Action::WrenchWalk => "wrench_walk",
            Action::TapeHold => "tape_hold",
            Action::TapeUse => "tape_use",
            Action::TapeWalk => "tape_walk",
        }
    }

    /// Atlas index of this action's first frame.
    fn base(self) -> usize {
        match self {
            Action::Idle => 0,
            Action::Walk => 16,
            Action::WrenchHold => 48,
            Action::WrenchUse => 64,
            Action::TapeHold => 88,
            Action::TapeUse => 104,
            Action::WrenchWalk => 128,
            Action::TapeWalk => 160,
        }
    }

    /// Facing directions drawn: 16 for standing poses, 8 for motion.
    pub fn directions(self) -> usize {
        match self {
            Action::Idle | Action::WrenchHold | Action::TapeHold => 16,
            _ => 8,
        }
    }

    pub fn frames(self) -> usize {
        match self {
            Action::Walk | Action::WrenchWalk | Action::TapeWalk => 4,
            Action::WrenchUse | Action::TapeUse => 3,
            _ => 1,
        }
    }

    /// Seconds per frame.
    pub fn frame_secs(self) -> f32 {
        match self {
            Action::Walk | Action::WrenchWalk | Action::TapeWalk => 0.11,
            Action::WrenchUse | Action::TapeUse => 0.09,
            _ => 0.12,
        }
    }

    /// Atlas index for a facing (`dir16`, 0 = east, clockwise on screen)
    /// and frame. Eight-direction actions use the nearest even direction.
    pub fn atlas_index(self, dir16: usize, frame: usize) -> usize {
        let dir = dir16 % 16;
        let frame = frame % self.frames();
        match self.directions() {
            16 => self.base() + dir,
            _ => self.base() + (dir.div_ceil(2) % 8) * self.frames() + frame,
        }
    }
}

/// Facing index for a world-space direction (y up): 0 is east, then
/// clockwise on screen. `directions` is 8 or 16; the result is always a
/// 16-direction index (even when `directions` is 8). `None` for a zero vector.
pub fn facing(v: Vec2, directions: usize) -> Option<usize> {
    if v.length_squared() < 1e-8 {
        return None;
    }
    let clockwise = (-v.y).atan2(v.x);
    let step = std::f32::consts::TAU / directions as f32;
    let i = (clockwise / step).round().rem_euclid(directions as f32) as usize % directions;
    Some(i * (16 / directions))
}

/// Unit vector of a 16-direction facing (world space, y up).
pub fn facing_vector(dir16: usize) -> Vec2 {
    let angle = (dir16 % 16) as f32 * std::f32::consts::TAU / 16.0;
    Vec2::new(angle.cos(), -angle.sin())
}

/// Wrench bite centre per 16-direction facing, in image pixels.
const WRENCH_CONTACTS: [(i8, i8); 16] = [
    (59, 38),
    (57, 49),
    (51, 57),
    (40, 57),
    (32, 59),
    (24, 57),
    (13, 57),
    (6, 49),
    (4, 38),
    (6, 27),
    (6, 12),
    (19, 6),
    (32, 4),
    (45, 6),
    (57, 13),
    (57, 27),
];

/// Tape leading edge per 16-direction facing, in image pixels.
const TAPE_CONTACTS: [(i8, i8); 16] = [
    (62, 38),
    (61, 50),
    (54, 60),
    (42, 61),
    (32, 62),
    (22, 61),
    (10, 60),
    (2, 51),
    (1, 38),
    (2, 25),
    (3, 9),
    (17, 2),
    (32, 1),
    (47, 2),
    (60, 10),
    (61, 26),
];

/// Where the held tool touches, relative to the engineer's root (world
/// units, y up). `wrench` false means the tape roll.
pub fn contact(wrench: bool, dir16: usize) -> Vec2 {
    let (x, y) = if wrench {
        WRENCH_CONTACTS[dir16 % 16]
    } else {
        TAPE_CONTACTS[dir16 % 16]
    };
    Vec2::new(x as f32 - ROOT.x, ROOT.y - y as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../assets/characters/engineer/manifest.json"
        ))
        .unwrap()
    }

    #[test]
    fn atlas_indices_and_contacts_match_the_manifest() {
        let m = manifest();
        let frames = m["frames"].as_array().unwrap();
        for f in frames {
            let name = f["action"].as_str().unwrap();
            let action = Action::ALL
                .into_iter()
                .find(|a| a.as_str() == name)
                .unwrap_or_else(|| panic!("unknown action {name}"));
            let dir = f["direction_index"].as_u64().unwrap() as usize;
            let frame = f["frame"].as_u64().unwrap() as usize;
            assert_eq!(
                action.atlas_index(dir, frame),
                f["atlas_index"].as_u64().unwrap() as usize,
                "{}",
                f["file"]
            );
            assert!(
                (action.frame_secs() * 1000.0 - f["duration_ms"].as_f64().unwrap() as f32).abs()
                    < 0.5
            );
            if let Some(c) = f["contact_pixels"].as_array() {
                let px = Vec2::new(c[0].as_f64().unwrap() as f32, c[1].as_f64().unwrap() as f32);
                let wrench = name.starts_with("wrench");
                assert_eq!(contact(wrench, dir), Vec2::new(px.x - 32.0, 38.0 - px.y));
            }
        }
        assert_eq!(frames.len(), (COLUMNS * ROWS) as usize);
        assert_eq!(m["pivot_pixels"][0].as_f64(), Some(ROOT.x as f64));
        assert_eq!(m["pivot_pixels"][1].as_f64(), Some(ROOT.y as f64));
    }

    #[test]
    fn facing_quantizes_clockwise_from_east() {
        assert_eq!(facing(Vec2::X, 16), Some(0));
        assert_eq!(
            facing(Vec2::NEG_Y, 16),
            Some(4),
            "south is a quarter turn clockwise"
        );
        assert_eq!(facing(Vec2::NEG_X, 16), Some(8));
        assert_eq!(facing(Vec2::Y, 16), Some(12));
        assert_eq!(facing(Vec2::new(1.0, -1.0), 8), Some(2));
        assert_eq!(
            facing(Vec2::new(1.0, 0.3), 8),
            Some(0),
            "eight directions snap to even indices"
        );
        assert_eq!(facing(Vec2::ZERO, 16), None);
        for dir in 0..16 {
            assert_eq!(facing(facing_vector(dir), 16), Some(dir));
        }
    }

    #[test]
    fn eight_direction_actions_round_odd_facings() {
        // NNE (13) walks with the N (12) or NE (14) frames, never another action's.
        let i = Action::WrenchWalk.atlas_index(13, 1);
        assert!(i == Action::WrenchWalk.atlas_index(14, 1));
        assert_eq!(
            Action::Walk.atlas_index(15, 0),
            Action::Walk.atlas_index(0, 0)
        );
    }

    #[test]
    fn tools_reach_out_along_the_facing() {
        for dir in 0..16 {
            for wrench in [true, false] {
                let c = contact(wrench, dir);
                assert!(c.length() > 18.0, "{dir}");
                assert!(c.normalize().dot(facing_vector(dir)) > 0.95, "{dir}");
            }
        }
    }
}
