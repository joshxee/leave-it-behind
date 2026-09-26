//! Where each kind of fault can happen. Hull breaches concentrate around the
//! airlock: its sites carry three times the weight of the main hull's.

use bevy::prelude::*;

use super::FaultKind;
use crate::ship::RoomId;
use crate::ship::layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Site {
    /// Bolt panels: three bolts along one face of an engine block.
    PortEngineInner,
    PortEngineOuter,
    StarboardEngineInner,
    StarboardEngineOuter,
    /// The nav helm in the cockpit.
    Helm,
    /// Breach points on the airlock walls.
    AirlockHatchUpper,
    AirlockHatchLower,
    AirlockPortAft,
    AirlockPortFore,
    AirlockStarboardAft,
    AirlockStarboardFore,
    /// Breach points on the main hull walls.
    HullPortAft,
    HullStarboardMid,
    HullPortFore,
}

pub const BOLT_SITES: [Site; 4] = [
    Site::PortEngineInner,
    Site::PortEngineOuter,
    Site::StarboardEngineInner,
    Site::StarboardEngineOuter,
];
pub const DRIFT_SITES: [Site; 1] = [Site::Helm];
pub const BREACH_SITES: [Site; 9] = [
    Site::AirlockHatchUpper,
    Site::AirlockHatchLower,
    Site::AirlockPortAft,
    Site::AirlockPortFore,
    Site::AirlockStarboardAft,
    Site::AirlockStarboardFore,
    Site::HullPortAft,
    Site::HullStarboardMid,
    Site::HullPortFore,
];

/// Bolt spacing along an engine face.
const BOLT_SPACING: f32 = 120.0;

impl Site {
    pub const ALL: [Site; 14] = [
        Site::PortEngineInner,
        Site::PortEngineOuter,
        Site::StarboardEngineInner,
        Site::StarboardEngineOuter,
        Site::Helm,
        Site::AirlockHatchUpper,
        Site::AirlockHatchLower,
        Site::AirlockPortAft,
        Site::AirlockPortFore,
        Site::AirlockStarboardAft,
        Site::AirlockStarboardFore,
        Site::HullPortAft,
        Site::HullStarboardMid,
        Site::HullPortFore,
    ];

    pub fn kind(self) -> FaultKind {
        match self {
            Site::PortEngineInner
            | Site::PortEngineOuter
            | Site::StarboardEngineInner
            | Site::StarboardEngineOuter => FaultKind::LooseBolts,
            Site::Helm => FaultKind::TrajectoryDrift,
            _ => FaultKind::HullBreach,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Site::PortEngineInner => "PortEngineInner",
            Site::PortEngineOuter => "PortEngineOuter",
            Site::StarboardEngineInner => "StarboardEngineInner",
            Site::StarboardEngineOuter => "StarboardEngineOuter",
            Site::Helm => "Helm",
            Site::AirlockHatchUpper => "AirlockHatchUpper",
            Site::AirlockHatchLower => "AirlockHatchLower",
            Site::AirlockPortAft => "AirlockPortAft",
            Site::AirlockPortFore => "AirlockPortFore",
            Site::AirlockStarboardAft => "AirlockStarboardAft",
            Site::AirlockStarboardFore => "AirlockStarboardFore",
            Site::HullPortAft => "HullPortAft",
            Site::HullStarboardMid => "HullStarboardMid",
            Site::HullPortFore => "HullPortFore",
        }
    }

    /// Where in the room, in screen terms (the diagnostic list).
    pub fn label(self) -> &'static str {
        match self {
            Site::PortEngineInner => "upper engine, spine side",
            Site::PortEngineOuter => "upper engine, wall side",
            Site::StarboardEngineInner => "lower engine, spine side",
            Site::StarboardEngineOuter => "lower engine, wall side",
            Site::Helm => "nav helm",
            Site::AirlockHatchUpper => "hatch seal, upper",
            Site::AirlockHatchLower => "hatch seal, lower",
            Site::AirlockPortAft | Site::HullPortAft => "upper wall, rear",
            Site::AirlockPortFore | Site::HullPortFore => "upper wall, front",
            Site::AirlockStarboardAft => "lower wall, rear",
            Site::AirlockStarboardFore => "lower wall, front",
            Site::HullStarboardMid => "lower wall, middle",
        }
    }

    pub fn room(self) -> RoomId {
        RoomId::at(self.pos().x)
    }

    /// The fault's location: the middle bolt of a panel, the breach point on
    /// the wall's inner face, or the joystick.
    pub fn pos(self) -> Vec2 {
        let (a, h) = (RoomId::Airlock.interior(), RoomId::Hull.interior());
        match self {
            Site::PortEngineInner => bottom_mid(layout::port_engine()),
            Site::PortEngineOuter => top_mid(layout::port_engine()),
            Site::StarboardEngineInner => top_mid(layout::starboard_engine()),
            Site::StarboardEngineOuter => bottom_mid(layout::starboard_engine()),
            Site::Helm => layout::joystick(),
            Site::AirlockHatchUpper => Vec2::new(a.min.x, 150.0),
            Site::AirlockHatchLower => Vec2::new(a.min.x, -150.0),
            Site::AirlockPortAft => Vec2::new(a.min.x + 190.0, a.max.y),
            Site::AirlockPortFore => Vec2::new(a.max.x - 190.0, a.max.y),
            Site::AirlockStarboardAft => Vec2::new(a.min.x + 230.0, a.min.y),
            Site::AirlockStarboardFore => Vec2::new(a.max.x - 150.0, a.min.y),
            Site::HullPortAft => Vec2::new(h.min.x + 170.0, h.max.y),
            Site::HullStarboardMid => Vec2::new(h.center().x, h.min.y),
            Site::HullPortFore => Vec2::new(h.max.x - 170.0, h.max.y),
        }
    }

    /// Unit vector out of the surface the fault sits on, into the room.
    pub fn normal(self) -> Vec2 {
        match self {
            Site::PortEngineInner | Site::StarboardEngineOuter => Vec2::NEG_Y,
            Site::PortEngineOuter | Site::StarboardEngineInner => Vec2::Y,
            Site::Helm => Vec2::NEG_X,
            Site::AirlockHatchUpper | Site::AirlockHatchLower => Vec2::X,
            Site::AirlockPortAft
            | Site::AirlockPortFore
            | Site::HullPortAft
            | Site::HullPortFore => Vec2::NEG_Y,
            Site::AirlockStarboardAft | Site::AirlockStarboardFore | Site::HullStarboardMid => {
                Vec2::Y
            }
        }
    }

    /// Default pick weight among the sites of the same kind.
    pub fn weight(self) -> f32 {
        match self.room() {
            RoomId::Airlock => 3.0,
            _ => 1.0,
        }
    }

    /// The three bolt positions of an engine panel.
    pub fn bolts(self) -> Option<[Vec2; 3]> {
        (self.kind() == FaultKind::LooseBolts).then(|| {
            let mid = self.pos();
            [-BOLT_SPACING, 0.0, BOLT_SPACING].map(|dx| mid + Vec2::new(dx, 0.0))
        })
    }
}

fn top_mid(r: Rect) -> Vec2 {
    Vec2::new(r.center().x, r.max.y)
}

fn bottom_mid(r: Rect) -> Vec2 {
    Vec2::new(r.center().x, r.min.y)
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn breaches_sit_on_airlock_or_hull_walls() {
        let walls = walls();
        for site in BREACH_SITES {
            assert!(
                matches!(site.room(), RoomId::Airlock | RoomId::Hull),
                "{site:?}"
            );
            let contact = wall_contact(site.pos(), 0.5, &walls).expect("on a wall");
            assert_eq!(contact.normal, site.normal(), "{site:?}");
        }
    }

    #[test]
    fn breaches_concentrate_around_the_airlock() {
        let weight = |room| {
            BREACH_SITES
                .iter()
                .filter(|s| s.room() == room)
                .map(|s| s.weight())
                .sum::<f32>()
        };
        assert!(weight(RoomId::Airlock) >= 3.0 * weight(RoomId::Hull));
    }

    #[test]
    fn every_site_can_be_worked_from_open_floor() {
        // A player standing 40 units out along the normal is not inside anything.
        let colliders = colliders();
        for site in Site::ALL {
            let stand = site.pos() + site.normal() * 40.0;
            assert_eq!(resolve_circle(stand, 18.0, &colliders), stand, "{site:?}");
            if let Some(bolts) = site.bolts() {
                for bolt in bolts {
                    let stand = bolt + site.normal() * 40.0;
                    assert_eq!(resolve_circle(stand, 18.0, &colliders), stand, "{site:?}");
                }
            }
        }
        // The helm is worked from the seat, which faces the joystick.
        assert!(layout::helm_seat().distance(Site::Helm.pos()) < 60.0);
    }
}
