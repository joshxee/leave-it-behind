//! The campaign: five levels flown in order, an upgrade picked between each
//! (`upgrades`). Level one (`one.rs`) teaches the three fixes; the rest are
//! randomized within their windows (which fault, where, exactly when, how
//! long its clock), so no two flights are the same, and ramp up:
//!
//! | Level | Flight | Faults | At once | Clocks |
//! |---|---|---|---|---|
//! | 1 | 2:30 | 4 | 1 | 55-80 s |
//! | 2 | 3:30 | 6 | 1 | 55-75 s |
//! | 3 | 3:45 | 8 | 2 | 55-70 s |
//! | 4 | 4:00 | 11 | 3 (once) | 55-68 s |
//! | 5 | 4:30 | 14 | 3 (twice) | 55-65 s |
//!
//! Each slot names the kinds it may be, so every level mixes all three and
//! never asks for more tape than one roll holds.

use super::def::{Envelope, FaultSlot, LevelDef, TimeWindow};
use super::one::level_one;
use crate::faults::FaultKind::{
    self, HullBreach as Breach, LooseBolts as Bolts, TrajectoryDrift as Drift,
};

/// Levels in the campaign.
pub const LEVEL_COUNT: usize = 5;

/// Each level's id (`LevelDef::id`), in the order they are flown. Saved
/// progress is keyed by these: never change a released one.
pub const LEVEL_IDS: [&str; LEVEL_COUNT] = ["one", "two", "three", "four", "five"];

/// Every level, in the order they are flown.
pub fn campaign() -> [LevelDef; LEVEL_COUNT] {
    [
        level_one(),
        level_two(),
        level_three(),
        level_four(),
        level_five(),
    ]
}

/// Level `number` (1 to [`LEVEL_COUNT`]).
pub fn level(number: usize) -> Option<LevelDef> {
    Some(match number {
        1 => level_one(),
        2 => level_two(),
        3 => level_three(),
        4 => level_four(),
        5 => level_five(),
        _ => return None,
    })
}

/// Where the level with `id` comes in the campaign (1-based).
pub fn number_of(id: &str) -> Option<usize> {
    LEVEL_IDS.iter().position(|&l| l == id).map(|i| i + 1)
}

/// The level flown after the one with `id`, if there is one.
pub fn next_after(id: &str) -> Option<LevelDef> {
    number_of(id).and_then(|n| level(n + 1))
}

fn slot(from: f32, to: f32, kinds: &[FaultKind], clock: (f32, f32)) -> FaultSlot {
    FaultSlot::any(
        TimeWindow::new(from, to),
        kinds,
        TimeWindow::new(clock.0, clock.1),
    )
}

fn level_def(
    id: &str,
    number: usize,
    duration: f32,
    max_overlap: usize,
    slots: Vec<FaultSlot>,
) -> LevelDef {
    LevelDef {
        id: id.into(),
        name: format!("Level {number}"),
        duration_secs: duration,
        seed: None,
        envelope: Envelope {
            max_overlap,
            response_secs: 20.0,
        },
        slots,
        coaching: false,
    }
}

/// A longer flight, still one fault at a time.
pub fn level_two() -> LevelDef {
    level_def(
        "two",
        2,
        210.0,
        1,
        vec![
            slot(10.0, 16.0, &[Bolts], (65.0, 75.0)),
            slot(36.0, 42.0, &[Breach], (60.0, 72.0)),
            slot(62.0, 68.0, &[Drift], (60.0, 72.0)),
            slot(88.0, 94.0, &[Bolts, Breach], (60.0, 72.0)),
            slot(114.0, 120.0, &[Drift, Bolts], (60.0, 72.0)),
            slot(140.0, 144.0, &[Breach, Drift], (55.0, 62.0)),
        ],
    )
}

/// Two at once, from the second minute.
pub fn level_three() -> LevelDef {
    level_def(
        "three",
        3,
        225.0,
        2,
        vec![
            slot(8.0, 12.0, &[Breach], (60.0, 70.0)),
            slot(32.0, 38.0, &[Bolts], (60.0, 70.0)),
            // Pairs.
            slot(60.0, 64.0, &[Drift], (60.0, 70.0)),
            slot(66.0, 70.0, &[Breach, Bolts], (60.0, 70.0)),
            slot(100.0, 104.0, &[Bolts], (58.0, 68.0)),
            slot(106.0, 110.0, &[Breach], (58.0, 68.0)),
            slot(140.0, 146.0, &[Drift], (55.0, 65.0)),
            slot(146.0, 150.0, &[Bolts, Breach], (55.0, 60.0)),
        ],
    )
}

/// Pairs all flight long, and once all three at once.
pub fn level_four() -> LevelDef {
    level_def(
        "four",
        4,
        240.0,
        3,
        vec![
            slot(8.0, 12.0, &[Bolts], (60.0, 68.0)),
            slot(14.0, 18.0, &[Breach], (60.0, 68.0)),
            slot(44.0, 50.0, &[Drift], (58.0, 68.0)),
            slot(70.0, 74.0, &[Bolts], (58.0, 68.0)),
            slot(74.0, 78.0, &[Breach], (58.0, 68.0)),
            slot(100.0, 104.0, &[Drift, Bolts], (55.0, 65.0)),
            slot(106.0, 110.0, &[Breach], (55.0, 65.0)),
            // All three at once.
            slot(136.0, 140.0, &[Bolts], (55.0, 65.0)),
            slot(140.0, 144.0, &[Drift], (55.0, 65.0)),
            slot(144.0, 148.0, &[Breach], (55.0, 62.0)),
            slot(172.0, 176.0, &[Bolts, Drift], (55.0, 60.0)),
        ],
    )
}

/// The last flight: short clocks, pairs throughout, all three at once twice.
pub fn level_five() -> LevelDef {
    level_def(
        "five",
        5,
        270.0,
        3,
        vec![
            slot(8.0, 12.0, &[Bolts], (58.0, 65.0)),
            slot(12.0, 16.0, &[Breach], (58.0, 65.0)),
            slot(40.0, 44.0, &[Drift], (55.0, 65.0)),
            slot(44.0, 48.0, &[Bolts], (55.0, 65.0)),
            // All three at once.
            slot(72.0, 76.0, &[Breach], (55.0, 65.0)),
            slot(76.0, 80.0, &[Bolts], (55.0, 65.0)),
            slot(80.0, 84.0, &[Drift], (55.0, 65.0)),
            slot(112.0, 116.0, &[Bolts], (55.0, 65.0)),
            slot(116.0, 120.0, &[Breach, Drift], (55.0, 65.0)),
            // All three at once, again.
            slot(150.0, 154.0, &[Drift], (55.0, 65.0)),
            slot(154.0, 158.0, &[Bolts], (55.0, 65.0)),
            slot(158.0, 162.0, &[Breach], (55.0, 65.0)),
            slot(192.0, 196.0, &[Bolts, Breach], (55.0, 65.0)),
            slot(196.0, 200.0, &[Drift], (55.0, 65.0)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameRng;
    use crate::faults::breach::SEAL_SECS;
    use crate::level::def::peak_overlap;
    use crate::tools::TAPE_CAPACITY;

    #[test]
    fn five_valid_levels_with_unique_ids_and_names() {
        let levels = campaign();
        for (i, level) in levels.iter().enumerate() {
            assert_eq!(level.validate(), Ok(()), "{}", level.id);
            assert_eq!(level.id, LEVEL_IDS[i]);
            assert_eq!(level.name, format!("Level {}", i + 1));
            assert_eq!(number_of(&level.id), Some(i + 1));
            assert_eq!(level.coaching, i == 0, "only level one coaches");
        }
        let mut ids: Vec<&str> = levels.iter().map(|l| l.id.as_str()).collect();
        ids.dedup();
        assert_eq!(ids.len(), LEVEL_COUNT);
    }

    #[test]
    fn levels_follow_one_another_and_the_last_has_no_next() {
        assert_eq!(level(1).unwrap().id, "one");
        assert_eq!(next_after("one").unwrap().id, "two");
        assert_eq!(next_after("four").unwrap().id, "five");
        assert!(next_after("five").is_none());
        assert!(level(0).is_none() && level(6).is_none());
        assert!(number_of("nope").is_none());
    }

    #[test]
    fn the_first_two_levels_are_short_and_one_at_a_time() {
        assert_eq!(level_one().duration_secs, 150.0);
        assert_eq!(level_two().duration_secs, 210.0);
        assert_eq!(level_two().worst_case_overlap(), 1);
    }

    #[test]
    fn difficulty_ramps_up() {
        let levels = campaign();
        let overlaps: Vec<usize> = levels.iter().map(LevelDef::worst_case_overlap).collect();
        assert_eq!(overlaps, vec![1, 1, 2, 3, 3]);
        for pair in levels.windows(2) {
            assert!(pair[1].duration_secs >= pair[0].duration_secs);
            assert!(pair[1].slots.len() >= pair[0].slots.len());
        }
        // Level five asks for all three at once more often than level four.
        let triples = |level: &LevelDef| {
            let plan = level.roll(&mut GameRng::from_seed(0));
            let r = level.envelope.response_secs;
            plan.iter()
                .filter(|f| {
                    let around: Vec<_> = plan
                        .iter()
                        .copied()
                        .filter(|g| (g.at - f.at).abs() < 12.0)
                        .collect();
                    peak_overlap(&around, r) == 3
                })
                .count()
        };
        assert!(triples(&level_five()) > triples(&level_four()));
    }

    #[test]
    fn every_fault_can_fail_before_landing() {
        // Otherwise it would not matter whether the player fixed it.
        for level in campaign() {
            for (i, s) in level.slots.iter().enumerate() {
                assert!(
                    s.window.to + s.clock.to < level.duration_secs,
                    "{} slot {i}",
                    level.id
                );
            }
        }
    }

    #[test]
    fn every_level_has_every_kind() {
        for level in campaign() {
            for kind in FaultKind::ALL {
                // A slot that can only be this kind: it always comes up.
                assert!(
                    level
                        .slots
                        .iter()
                        .any(|s| s.choices.iter().all(|c| c.kind == kind)),
                    "{} may have no {kind:?}",
                    level.id
                );
            }
        }
    }

    #[test]
    fn one_roll_of_tape_covers_every_breach_with_room_to_waste() {
        for level in campaign() {
            let breaches = level.slots.iter().filter(|s| s.can_be(Breach)).count() as f32;
            let needed = breaches * SEAL_SECS;
            assert!(
                TAPE_CAPACITY - needed >= 5.0,
                "{} can need {needed}s of tape",
                level.id
            );
        }
    }
}
