//! Level one: a fully pinned instance of the level format. Every slot has
//! an exact start time, site and clock, and the seed fixes the drift
//! headings, so every run of this level is identical.
//!
//! Tuning (4:30 flight, clocks 68-86 s, about 20 s to reach and fix a fault):
//! - 0:10-1:30, settling in: one fault at a time, each kind once.
//! - 1:36-2:40, pairs: two faults at once, the second pair at opposite ends
//!   of the ship (helm and airlock).
//! - 2:56-3:06, final approach: all three kinds at once, spread tip to tail.

use super::def::{Envelope, FaultSlot, LevelDef};
use crate::faults::Site;

pub const LEVEL_ONE_SEED: u64 = 0x1EA7_0001;

pub fn level_one() -> LevelDef {
    LevelDef {
        id: "one".into(),
        name: "Level 1".into(),
        duration_secs: 270.0,
        seed: Some(LEVEL_ONE_SEED),
        envelope: Envelope {
            max_overlap: 3,
            response_secs: 20.0,
        },
        slots: vec![
            // Settling in: comfortable, one at a time.
            FaultSlot::pinned(10.0, Site::PortEngineInner, 80.0),
            FaultSlot::pinned(38.0, Site::AirlockPortAft, 74.0),
            FaultSlot::pinned(66.0, Site::Helm, 86.0),
            // Pairs: tense.
            FaultSlot::pinned(96.0, Site::HullStarboardMid, 74.0),
            FaultSlot::pinned(104.0, Site::StarboardEngineOuter, 80.0),
            FaultSlot::pinned(132.0, Site::Helm, 80.0),
            FaultSlot::pinned(140.0, Site::AirlockHatchStarboard, 68.0),
            // Final approach: a scramble.
            FaultSlot::pinned(176.0, Site::PortEngineOuter, 74.0),
            FaultSlot::pinned(181.0, Site::AirlockStarboardFore, 68.0),
            FaultSlot::pinned(186.0, Site::Helm, 74.0),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameRng;
    use crate::faults::FaultKind;
    use crate::faults::breach::SEAL_SECS;
    use crate::level::def::peak_overlap;
    use crate::tools::TAPE_CAPACITY;

    #[test]
    fn level_one_is_valid_and_fully_pinned() {
        let level = level_one();
        assert_eq!(level.validate(), Ok(()));
        assert!(level.is_pinned());
        assert!(level.seed.is_some());
        assert!(
            (180.0..=300.0).contains(&level.duration_secs),
            "three to five minutes"
        );
    }

    #[test]
    fn level_one_rolls_the_same_for_any_seed() {
        let level = level_one();
        let a = level.roll(&mut GameRng::from_seed(1));
        let b = level.roll(&mut GameRng::from_seed(999));
        assert_eq!(a, b);
        assert_eq!(a.len(), level.slots.len());
    }

    #[test]
    fn difficulty_ramps_from_one_to_three_at_once() {
        let level = level_one();
        let plan = level.roll(&mut GameRng::from_seed(0));
        let r = level.envelope.response_secs;
        let phase = |from: f32, to: f32| {
            let p: Vec<_> = plan
                .iter()
                .copied()
                .filter(|f| (from..to).contains(&f.at))
                .collect();
            peak_overlap(&p, r)
        };
        assert_eq!(phase(0.0, 90.0), 1, "comfortable");
        assert_eq!(phase(90.0, 170.0), 2, "tense");
        assert_eq!(phase(170.0, 270.0), 3, "scramble");
        assert_eq!(peak_overlap(&plan, r), level.envelope.max_overlap);
    }

    #[test]
    fn every_kind_appears_and_the_finale_has_all_three() {
        let plan = level_one().roll(&mut GameRng::from_seed(0));
        let finale: Vec<FaultKind> = plan
            .iter()
            .filter(|f| f.at >= 170.0)
            .map(|f| f.site.kind())
            .collect();
        for kind in FaultKind::ALL {
            assert!(finale.contains(&kind), "{kind:?} missing from the finale");
        }
    }

    #[test]
    fn every_fault_can_fail_before_landing() {
        // Otherwise it would not matter whether the player fixed it.
        let level = level_one();
        for f in level.roll(&mut GameRng::from_seed(0)) {
            assert!(f.at + f.clock < level.duration_secs, "{f:?}");
        }
    }

    #[test]
    fn one_roll_of_tape_covers_every_breach_with_room_to_waste() {
        let breaches = level_one()
            .slots
            .iter()
            .filter(|s| s.choices[0].kind == FaultKind::HullBreach)
            .count() as f32;
        let needed = breaches * SEAL_SECS;
        assert!(needed < TAPE_CAPACITY, "needs {needed}s of tape");
        assert!(TAPE_CAPACITY - needed >= 5.0, "less than 5s of slack");
    }
}
