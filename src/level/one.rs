//! Level one: a short first flight that teaches the three fixes. The first
//! three faults are pinned (one of each kind, one at a time, so the coaching
//! can tip each), and a fourth of a random kind comes last, so one fix is
//! done twice. Nothing ever overlaps.
//!
//! Tuning (2:30 flight, about 20 s to reach and fix a fault):
//! - 0:10 loose bolts, 0:34 a breach in the same engine room, 0:58 drift.
//! - 1:24-1:30 one more of any kind, anywhere.
//!
//! Breach and bolt clocks are drains on the ship's oxygen and engine heat
//! (`faults::vitals`): each alone plays exactly as its clock, but one left
//! long drains the pool for the next of its kind.

use super::def::{Envelope, FaultSlot, LevelDef, TimeWindow};
use crate::faults::{FaultKind, Site};

/// Where level one's first loose bolts and first breach are (the coaching
/// points at them).
pub const FIRST_BOLTS: Site = Site::PortEngineInner;
pub const FIRST_BREACH: Site = Site::EngineRoomPort;

pub fn level_one() -> LevelDef {
    LevelDef {
        id: "one".into(),
        name: "Level 1".into(),
        duration_secs: 150.0,
        // Unseeded: the last fault's kind and site differ from run to run.
        seed: None,
        envelope: Envelope {
            max_overlap: 1,
            response_secs: 20.0,
        },
        slots: vec![
            FaultSlot::pinned(10.0, FIRST_BOLTS, 70.0),
            FaultSlot::pinned(34.0, FIRST_BREACH, 70.0),
            FaultSlot::pinned(58.0, Site::Helm, 80.0),
            FaultSlot::any(
                TimeWindow::new(84.0, 90.0),
                &FaultKind::ALL,
                TimeWindow::new(55.0, 58.0),
            ),
        ],
        coaching: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameRng;

    #[test]
    fn level_one_is_two_and_a_half_minutes() {
        let level = level_one();
        assert_eq!(level.validate(), Ok(()));
        assert_eq!(level.duration_secs, 150.0);
        assert!(level.coaching);
    }

    #[test]
    fn every_kind_once_then_one_of_them_again() {
        let level = level_one();
        let mut kinds_seen = std::collections::HashSet::new();
        for seed in 0..300 {
            let plan = level.roll(&mut GameRng::from_seed(seed));
            assert_eq!(plan.len(), 4);
            // Coaching tips each kind's first fault: the first three are one of each.
            let firsts: Vec<FaultKind> = plan.iter().take(3).map(|f| f.site.kind()).collect();
            for kind in FaultKind::ALL {
                assert!(
                    firsts.contains(&kind),
                    "{kind:?} is not among the first three"
                );
            }
            kinds_seen.insert(plan[3].site.kind());
        }
        assert_eq!(kinds_seen.len(), 3, "the repeat can be any kind");
    }

    #[test]
    fn faults_come_one_at_a_time() {
        assert_eq!(level_one().worst_case_overlap(), 1);
    }

    #[test]
    fn the_first_breach_is_in_the_engine_room_with_the_bolts() {
        let level = level_one();
        let plan = level.roll(&mut GameRng::from_seed(0));
        assert_eq!(plan[0].site.room(), plan[1].site.room());
        assert_eq!(plan[1].site.kind(), FaultKind::HullBreach);
    }
}
