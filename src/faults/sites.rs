//! Where each kind of fault can happen. Positions come from the ship map
//! (`ship::map`): breach points are marked on hull walls (`BREACH_MARKS`),
//! bolt panels are the long faces of the engine blocks `'P'` and `'S'`, and
//! the helm is the cockpit's pilot seat. Names are ship-relative (port and
//! starboard, fore and aft), so they hold whichever way the ship points; the
//! labels the diagnostic screen shows are worked out from the layout.
//!
//! Hull breaches can open in the airlock, the main hull, and the cockpit and
//! both engine rooms (where the other faults are, so there is less running
//! between them); never in the quarters or the gun room. An unpinned breach
//! picks any of them with equal odds.

use bevy::prelude::*;

use super::FaultKind;
use crate::ship::RoomId;
use crate::ship::layout::{self, ship};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Site {
    /// Bolt panels: three bolts along one long face of an engine block.
    /// Inner faces look toward the spine, outer faces toward the hull.
    PortEngineInner,
    PortEngineOuter,
    StarboardEngineInner,
    StarboardEngineOuter,
    /// The nav helm in the cockpit.
    Helm,
    /// Breach points on the airlock walls: either side of the outer hatch,
    /// and the port and starboard walls.
    AirlockHatchPort,
    AirlockHatchStarboard,
    AirlockPortAft,
    AirlockPortFore,
    AirlockStarboardAft,
    AirlockStarboardFore,
    /// Breach points on the main hull walls.
    HullPortAft,
    HullStarboardMid,
    HullPortFore,
    /// Breach points on the cockpit's side walls and the engine rooms'
    /// outer walls.
    CockpitPort,
    CockpitStarboard,
    EngineRoomPort,
    EngineRoomStarboard,
}

pub const BOLT_SITES: [Site; 4] = [
    Site::PortEngineInner,
    Site::PortEngineOuter,
    Site::StarboardEngineInner,
    Site::StarboardEngineOuter,
];
pub const DRIFT_SITES: [Site; 1] = [Site::Helm];
pub const BREACH_SITES: [Site; 13] = [
    Site::AirlockHatchPort,
    Site::AirlockHatchStarboard,
    Site::AirlockPortAft,
    Site::AirlockPortFore,
    Site::AirlockStarboardAft,
    Site::AirlockStarboardFore,
    Site::HullPortAft,
    Site::HullStarboardMid,
    Site::HullPortFore,
    Site::CockpitPort,
    Site::CockpitStarboard,
    Site::EngineRoomPort,
    Site::EngineRoomStarboard,
];

/// Bolt spacing along an engine face.
const BOLT_SPACING: f32 = 120.0;

/// Where on the map a site is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Anchor {
    /// An engine block's long face: the inner one, or the outer one.
    Engine(char, bool),
    Helm,
    /// A breach mark on a hull wall.
    Wall(char),
}

impl Site {
    pub const ALL: [Site; 18] = [
        Site::PortEngineInner,
        Site::PortEngineOuter,
        Site::StarboardEngineInner,
        Site::StarboardEngineOuter,
        Site::Helm,
        Site::AirlockHatchPort,
        Site::AirlockHatchStarboard,
        Site::AirlockPortAft,
        Site::AirlockPortFore,
        Site::AirlockStarboardAft,
        Site::AirlockStarboardFore,
        Site::HullPortAft,
        Site::HullStarboardMid,
        Site::HullPortFore,
        Site::CockpitPort,
        Site::CockpitStarboard,
        Site::EngineRoomPort,
        Site::EngineRoomStarboard,
    ];

    pub fn kind(self) -> FaultKind {
        match self.anchor() {
            Anchor::Engine(..) => FaultKind::LooseBolts,
            Anchor::Helm => FaultKind::TrajectoryDrift,
            Anchor::Wall(_) => FaultKind::HullBreach,
        }
    }

    fn anchor(self) -> Anchor {
        match self {
            Site::PortEngineInner => Anchor::Engine('P', true),
            Site::PortEngineOuter => Anchor::Engine('P', false),
            Site::StarboardEngineInner => Anchor::Engine('S', true),
            Site::StarboardEngineOuter => Anchor::Engine('S', false),
            Site::Helm => Anchor::Helm,
            Site::AirlockHatchPort => Anchor::Wall('1'),
            Site::AirlockHatchStarboard => Anchor::Wall('2'),
            Site::AirlockPortAft => Anchor::Wall('3'),
            Site::AirlockPortFore => Anchor::Wall('4'),
            Site::AirlockStarboardAft => Anchor::Wall('5'),
            Site::AirlockStarboardFore => Anchor::Wall('6'),
            Site::HullPortAft => Anchor::Wall('7'),
            Site::HullStarboardMid => Anchor::Wall('8'),
            Site::HullPortFore => Anchor::Wall('9'),
            Site::CockpitPort => Anchor::Wall('r'),
            Site::CockpitStarboard => Anchor::Wall('t'),
            Site::EngineRoomPort => Anchor::Wall('u'),
            Site::EngineRoomStarboard => Anchor::Wall('w'),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Site::PortEngineInner => "PortEngineInner",
            Site::PortEngineOuter => "PortEngineOuter",
            Site::StarboardEngineInner => "StarboardEngineInner",
            Site::StarboardEngineOuter => "StarboardEngineOuter",
            Site::Helm => "Helm",
            Site::AirlockHatchPort => "AirlockHatchPort",
            Site::AirlockHatchStarboard => "AirlockHatchStarboard",
            Site::AirlockPortAft => "AirlockPortAft",
            Site::AirlockPortFore => "AirlockPortFore",
            Site::AirlockStarboardAft => "AirlockStarboardAft",
            Site::AirlockStarboardFore => "AirlockStarboardFore",
            Site::HullPortAft => "HullPortAft",
            Site::HullStarboardMid => "HullStarboardMid",
            Site::HullPortFore => "HullPortFore",
            Site::CockpitPort => "CockpitPort",
            Site::CockpitStarboard => "CockpitStarboard",
            Site::EngineRoomPort => "EngineRoomPort",
            Site::EngineRoomStarboard => "EngineRoomStarboard",
        }
    }

    /// Where in the room, in screen terms (the diagnostic list), worked out
    /// from the layout: "left wall, bottom", "right engine, spine side"...
    pub fn label(self) -> String {
        let room = self.room().interior();
        match self.anchor() {
            Anchor::Helm => "nav helm".into(),
            Anchor::Engine(block, inner) => {
                let offset = layout::ship()
                    .block(block)
                    .map_or(Vec2::ZERO, |b| b.center())
                    - room.center();
                let which = if offset.x.abs() >= offset.y.abs() {
                    if offset.x < 0.0 { "left" } else { "right" }
                } else if offset.y > 0.0 {
                    "upper"
                } else {
                    "lower"
                };
                let face = if inner { "spine side" } else { "wall side" };
                format!("{which} engine, {face}")
            }
            Anchor::Wall(_) => {
                let (pos, normal) = (self.pos(), self.normal());
                let horizontal = normal.x == 0.0;
                let hatch = ship()
                    .doors()
                    .iter()
                    .find(|d| {
                        d.locked && d.across_x == horizontal && d.center.distance(pos) <= 2.5 * 64.0
                    })
                    .map(|d| d.center);
                let along = |lo: &'static str, mid: &'static str, hi: &'static str| {
                    let (v, min, max) = if horizontal {
                        (pos.x, room.min.x, room.max.x)
                    } else {
                        (pos.y, room.min.y, room.max.y)
                    };
                    let t = (v - min) / (max - min);
                    if t < 1.0 / 3.0 {
                        lo
                    } else if t > 2.0 / 3.0 {
                        hi
                    } else {
                        mid
                    }
                };
                if let Some(hatch) = hatch {
                    let side = if horizontal {
                        if pos.x < hatch.x { "left" } else { "right" }
                    } else if pos.y > hatch.y {
                        "top"
                    } else {
                        "bottom"
                    };
                    return format!("hatch seal, {side}");
                }
                let wall = match (normal.x, normal.y) {
                    (_, y) if y < 0.0 => "upper wall",
                    (_, y) if y > 0.0 => "lower wall",
                    (x, _) if x > 0.0 => "left wall",
                    _ => "right wall",
                };
                let spot = if horizontal {
                    along("left", "middle", "right")
                } else {
                    along("bottom", "middle", "top")
                };
                format!("{wall}, {spot}")
            }
        }
    }

    pub fn room(self) -> RoomId {
        RoomId::at(self.pos())
    }

    /// The fault's location: the middle bolt of a panel, the breach point on
    /// the wall's inner face, or the joystick.
    pub fn pos(self) -> Vec2 {
        match self.anchor() {
            Anchor::Engine(block, inner) => engine_face(block, inner).0,
            Anchor::Helm => layout::joystick(),
            Anchor::Wall(mark) => wall_mark(mark).point,
        }
    }

    /// Unit vector out of the surface the fault sits on, into the room.
    pub fn normal(self) -> Vec2 {
        match self.anchor() {
            Anchor::Engine(block, inner) => engine_face(block, inner).1,
            Anchor::Helm => -ship().helm().facing,
            Anchor::Wall(mark) => wall_mark(mark).normal,
        }
    }

    /// The wall cell of a breach site.
    pub fn wall_cell(self) -> Option<IVec2> {
        match self.anchor() {
            Anchor::Wall(mark) => Some(wall_mark(mark).cell),
            _ => None,
        }
    }

    /// The three bolt positions of an engine panel, along the face.
    pub fn bolts(self) -> Option<[Vec2; 3]> {
        (self.kind() == FaultKind::LooseBolts).then(|| {
            let (mid, normal) = (self.pos(), self.normal());
            [-BOLT_SPACING, 0.0, BOLT_SPACING].map(|d| mid + normal.perp() * d)
        })
    }
}

fn wall_mark(mark: char) -> crate::ship::map::WallMark {
    ship()
        .wall_mark(mark)
        .unwrap_or_else(|| panic!("the ship map has no breach point {mark:?}"))
}

/// The middle of an engine block's long face and its outward normal. The
/// inner face looks toward the middle of the room.
fn engine_face(block: char, inner: bool) -> (Vec2, Vec2) {
    let rect = ship()
        .block(block)
        .unwrap_or_else(|| panic!("the ship map has no engine {block:?}"));
    let middle = RoomId::at(rect.center()).center() - rect.center();
    let long_x = rect.width() >= rect.height();
    let normals = if long_x {
        [Vec2::Y, Vec2::NEG_Y]
    } else {
        [Vec2::X, Vec2::NEG_X]
    };
    let normal = normals
        .into_iter()
        .find(|n| (n.dot(middle) > 0.0) == inner)
        .expect("one face looks toward the middle");
    let depth = if long_x { rect.height() } else { rect.width() } / 2.0;
    (rect.center() + normal * depth, normal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PLAYER_RADIUS;
    use crate::ship::layout::{colliders, resolve_circle, wall_contact, walls};

    #[test]
    fn site_lists_match_kinds() {
        for (list, kind) in [
            (&BOLT_SITES[..], FaultKind::LooseBolts),
            (&DRIFT_SITES[..], FaultKind::TrajectoryDrift),
            (&BREACH_SITES[..], FaultKind::HullBreach),
        ] {
            assert!(list.iter().all(|s| s.kind() == kind));
            assert_eq!(
                Site::ALL.iter().filter(|s| s.kind() == kind).count(),
                list.len()
            );
        }
    }

    #[test]
    fn breaches_sit_on_hull_walls_outside_the_quarters_and_gun_room() {
        let walls = walls();
        for site in BREACH_SITES {
            assert_ne!(site.room(), RoomId::Quarters, "{site:?}");
            assert_ne!(site.room(), RoomId::GunRoom, "{site:?}");
            // Reaching for it from the room touches the wall right there.
            let from = site.pos() + site.normal() * 20.0;
            let contact = wall_contact(from, site.pos(), 0.5, &walls).expect("on a wall");
            assert!(contact.point.distance(site.pos()) < 1e-3, "{site:?}");
            assert_eq!(contact.normal, site.normal(), "{site:?}");
        }
    }

    #[test]
    fn the_cockpit_and_engine_rooms_have_breaches_near_the_other_faults() {
        // Close to the helm and the bolts: less running between faults.
        let sides: Vec<Vec2> = BREACH_SITES
            .iter()
            .filter(|s| s.room() == RoomId::Cockpit)
            .map(|s| s.normal())
            .collect();
        assert_eq!(sides.len(), 2);
        assert_eq!(sides[0], -sides[1], "opposite walls");
        for (site, engine) in [
            (Site::EngineRoomPort, Site::PortEngineOuter),
            (Site::EngineRoomStarboard, Site::StarboardEngineOuter),
        ] {
            assert_eq!(site.room(), engine.room(), "{site:?}");
            assert_eq!(
                site.normal(),
                -engine.normal(),
                "{site:?}: facing the engine"
            );
        }
        assert!(Site::CockpitPort.pos().x < Site::CockpitStarboard.pos().x);
        assert!(Site::EngineRoomPort.pos().x < Site::EngineRoomStarboard.pos().x);
    }

    #[test]
    fn every_site_can_be_worked_from_open_floor() {
        // A player standing 40 units out along the normal is not inside anything.
        let colliders = colliders();
        for site in Site::ALL {
            let stand = site.pos() + site.normal() * 40.0;
            assert_eq!(
                resolve_circle(stand, PLAYER_RADIUS, &colliders),
                stand,
                "{site:?}"
            );
            if let Some(bolts) = site.bolts() {
                for bolt in bolts {
                    let stand = bolt + site.normal() * 40.0;
                    assert_eq!(
                        resolve_circle(stand, PLAYER_RADIUS, &colliders),
                        stand,
                        "{site:?}"
                    );
                }
            }
        }
        // The helm is worked from the seat, which faces the joystick.
        assert!(layout::helm_seat().distance(Site::Helm.pos()) < 60.0);
    }

    #[test]
    fn port_is_left_and_fore_is_up_on_a_ship_pointing_up() {
        // The map points the ship up: port on the left, the bow at the top.
        assert!(Site::PortEngineInner.pos().x < 0.0);
        assert!(Site::StarboardEngineInner.pos().x > 0.0);
        assert!(Site::HullPortFore.pos().y > Site::HullPortAft.pos().y);
        assert!(Site::AirlockHatchPort.pos().x < Site::AirlockHatchStarboard.pos().x);
    }

    #[test]
    fn labels_follow_the_layout() {
        assert_eq!(Site::HullPortAft.label(), "left wall, bottom");
        assert_eq!(Site::HullStarboardMid.label(), "right wall, middle");
        assert_eq!(Site::AirlockHatchPort.label(), "hatch seal, left");
        assert_eq!(Site::PortEngineInner.label(), "left engine, spine side");
        assert_eq!(
            Site::StarboardEngineOuter.label(),
            "right engine, wall side"
        );
        assert_eq!(Site::Helm.label(), "nav helm");
        for site in Site::ALL {
            assert!(site.label().is_ascii(), "{site:?}");
        }
    }

    #[test]
    fn inner_bolt_faces_look_toward_the_spine() {
        for site in [Site::PortEngineInner, Site::StarboardEngineInner] {
            assert!(site.normal().dot(-site.pos().with_y(0.0)) > 0.0, "{site:?}");
        }
    }
}
