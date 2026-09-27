//! The campaign: five levels flown in order, an upgrade picked between each
//! (`upgrades`). Level one (`one.rs`) teaches the three fixes; the rest are
//! randomized within their windows (which fault, where, exactly when, how
//! long its clock), so no two flights are the same. From level two the
//! faults start at launch, and each level asks for more at once on shorter
//! clocks. A level lands once every fault is fixed, and every flight is over
//! within three minutes when each fault is fixed within 20 s:
//!
//! | Level | Faults | At once | Clocks | Over by |
//! |---|---|---|---|---|
//! | 1 | 4 | 1 | 55-80 s | 1:50 |
//! | 2 | 7 | 2 | 58-68 s | 2:22 |
//! | 3 | 10 | 3 | 52-62 s | 2:34 |
//! | 4 | 14 | 3 (waves) | 48-56 s | 2:34 |
//! | 5 | 16 | 4 (waves) | 45-54 s | 2:30 |
//!
//! Each slot names the kinds it may be, so every level mixes all three and
//! never asks for more tape than one roll holds. Levels four and five come
//! in waves (`wave`) about half a minute apart: the gaps between them are
//! what let the oxygen and the engine recover (`LevelDef::validate`).

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

/// A wave: one slot per entry of `kinds`, each 2 s wide, 4 s apart from
/// `from`. Clocks come from `clocks` by the slot's first kind.
fn wave(
    from: f32,
    kinds: &[&[FaultKind]],
    clocks: impl Fn(FaultKind) -> (f32, f32),
) -> Vec<FaultSlot> {
    kinds
        .iter()
        .enumerate()
        .map(|(i, kinds)| {
            let at = from + 4.0 * i as f32;
            slot(at, at + 2.0, kinds, clocks(kinds[0]))
        })
        .collect()
}

fn level_def(id: &str, number: usize, max_overlap: usize, slots: Vec<FaultSlot>) -> LevelDef {
    LevelDef {
        id: id.into(),
        name: format!("Level {number}"),
        seed: None,
        envelope: Envelope {
            max_overlap,
            response_secs: 20.0,
        },
        slots,
        coaching: false,
    }
}

/// Faults from launch, and now and then two at once.
pub fn level_two() -> LevelDef {
    level_def(
        "two",
        2,
        2,
        vec![
            slot(2.0, 4.0, &[Bolts], (60.0, 68.0)),
            slot(14.0, 18.0, &[Breach], (60.0, 68.0)),
            slot(40.0, 44.0, &[Drift], (58.0, 66.0)),
            slot(54.0, 58.0, &[Bolts, Breach], (58.0, 66.0)),
            slot(80.0, 84.0, &[Breach, Drift], (58.0, 66.0)),
            slot(92.0, 96.0, &[Bolts], (58.0, 66.0)),
            slot(118.0, 122.0, &[Drift, Bolts, Breach], (58.0, 64.0)),
        ],
    )
}

/// All three at once, three times over.
pub fn level_three() -> LevelDef {
    level_def(
        "three",
        3,
        3,
        vec![
            // All three at once, from launch.
            slot(1.0, 3.0, &[Breach], (55.0, 62.0)),
            slot(6.0, 10.0, &[Bolts], (55.0, 62.0)),
            slot(16.0, 20.0, &[Drift], (55.0, 62.0)),
            // Three again.
            slot(36.0, 40.0, &[Bolts, Breach], (52.0, 60.0)),
            slot(48.0, 52.0, &[Breach], (52.0, 60.0)),
            slot(56.0, 60.0, &[Drift], (52.0, 60.0)),
            // And again.
            slot(80.0, 84.0, &[Bolts], (52.0, 60.0)),
            slot(90.0, 94.0, &[Breach, Drift], (52.0, 60.0)),
            slot(100.0, 104.0, &[Drift, Bolts], (52.0, 60.0)),
            slot(128.0, 134.0, &[Breach], (52.0, 58.0)),
        ],
    )
}

/// Waves of three, one of each kind, every half minute.
pub fn level_four() -> LevelDef {
    let clocks = |kind: FaultKind| match kind {
        Drift => (48.0, 54.0),
        _ => (50.0, 56.0),
    };
    level_def(
        "four",
        4,
        3,
        [
            wave(0.0, &[&[Bolts], &[Breach], &[Drift]], clocks),
            wave(30.0, &[&[Drift], &[Bolts], &[Breach]], clocks),
            wave(62.0, &[&[Breach], &[Drift], &[Bolts]], clocks),
            wave(96.0, &[&[Bolts], &[Breach, Drift], &[Drift]], clocks),
            wave(128.0, &[&[Drift, Bolts], &[Breach]], clocks),
        ]
        .concat(),
    )
}

/// The last flight: the shortest clocks, and waves of up to four at once.
pub fn level_five() -> LevelDef {
    let clocks = |kind: FaultKind| match kind {
        Drift => (45.0, 50.0),
        _ => (50.0, 54.0),
    };
    level_def(
        "five",
        5,
        4,
        [
            wave(0.0, &[&[Breach], &[Bolts], &[Drift], &[Drift]], clocks),
            wave(30.0, &[&[Bolts], &[Breach], &[Drift]], clocks),
            wave(60.0, &[&[Drift], &[Bolts], &[Breach], &[Drift]], clocks),
            wave(92.0, &[&[Bolts], &[Breach], &[Drift]], clocks),
            wave(124.0, &[&[Breach, Bolts], &[Drift]], clocks),
        ]
        .concat(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameRng;
    use crate::faults::breach::SEAL_SECS;
    use crate::level::def::MAX_FLIGHT_SECS;
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
    fn every_level_is_over_within_three_minutes() {
        for level in campaign() {
            assert!(level.worst_case_secs() <= MAX_FLIGHT_SECS, "{}", level.id);
        }
    }

    #[test]
    fn faults_start_at_launch_after_level_one() {
        for level in campaign().iter().skip(1) {
            let first = level
                .slots
                .iter()
                .map(|s| s.window.from)
                .fold(f32::MAX, f32::min);
            assert!(first <= 2.0, "{} waits {first}s", level.id);
        }
    }

    #[test]
    fn difficulty_ramps_up_steeply() {
        let levels = campaign();
        let overlaps: Vec<usize> = levels.iter().map(LevelDef::worst_case_overlap).collect();
        assert_eq!(overlaps, vec![1, 2, 3, 3, 4]);
        let counts: Vec<usize> = levels.iter().map(|l| l.slots.len()).collect();
        assert_eq!(counts, vec![4, 7, 10, 14, 16]);
        // Clocks only get shorter: the longest from level to level, and the
        // shortest from level two on (level one's last fault is a short one).
        let shortest = |level: &LevelDef| {
            level
                .slots
                .iter()
                .map(|s| s.clock.from)
                .fold(f32::MAX, f32::min)
        };
        let longest = |level: &LevelDef| level.slots.iter().map(|s| s.clock.to).fold(0.0, f32::max);
        for pair in levels.windows(2) {
            assert!(longest(&pair[1]) < longest(&pair[0]), "{}", pair[1].id);
        }
        for pair in levels[1..].windows(2) {
            assert!(shortest(&pair[1]) < shortest(&pair[0]), "{}", pair[1].id);
        }
        // Level five really does ask for four at once: its first wave.
        let plan = level_five().roll(&mut GameRng::from_seed(0));
        assert_eq!(peak_overlap(&plan, level_five().envelope.response_secs), 4);
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
