//! How well a flight went, and the best flight per level (saved by `save`).
//!
//! A flight is scored by its damage: how long each fault ran before it was
//! fixed. Breaches lose oxygen, drift takes the ship off course, loose bolts
//! overheat the engine. Landing beats not landing; between landings the
//! least damage wins; between losses, the longest flight.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::faults::FaultKind;

/// Seconds each kind of fault ran before it was fixed (or the flight ended).
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Damage {
    /// Hull breaches: oxygen lost.
    pub oxygen: f32,
    /// Trajectory drift: time off course.
    pub course: f32,
    /// Loose bolts: time the engine overheated.
    pub engine: f32,
}

impl Damage {
    pub fn add(&mut self, kind: FaultKind, secs: f32) {
        match kind {
            FaultKind::HullBreach => self.oxygen += secs,
            FaultKind::TrajectoryDrift => self.course += secs,
            FaultKind::LooseBolts => self.engine += secs,
        }
    }

    pub fn total(&self) -> f32 {
        self.oxygen + self.course + self.engine
    }
}

/// Damage as the end screen shows it: whole seconds.
pub fn damage_secs(secs: f32) -> u32 {
    secs.max(0.0).round() as u32
}

/// Flight time as the end screen shows it (`format_clock` rounds up).
pub fn flight_secs(secs: f32) -> u32 {
    secs.max(0.0).ceil() as u32
}

/// One finished flight.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub landed: bool,
    pub damage: Damage,
    /// Seconds flown before touching down or failing.
    pub survived: f32,
}

impl RunRecord {
    /// Whether this flight is better than `other`. Compared in the whole
    /// seconds the end screen shows, so a new best is always visibly better.
    pub fn beats(&self, other: &RunRecord) -> bool {
        match (self.landed, other.landed) {
            (true, false) => true,
            (false, true) => false,
            (true, true) => damage_secs(self.damage.total()) < damage_secs(other.damage.total()),
            (false, false) => flight_secs(self.survived) > flight_secs(other.survived),
        }
    }
}

/// Everything saved about one level.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LevelProgress {
    /// Flights finished, landed or lost.
    pub flights: u32,
    pub landings: u32,
    pub best: Option<RunRecord>,
}

/// Progress across levels, by level id. Saved whenever it changes.
#[derive(Resource, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Progress {
    pub levels: BTreeMap<String, LevelProgress>,
}

impl Progress {
    pub fn level(&self, id: &str) -> Option<&LevelProgress> {
        self.levels.get(id)
    }

    /// Counts a finished flight. Returns whether it is the level's new best.
    pub fn record(&mut self, level: &str, run: RunRecord) -> bool {
        let entry = self.levels.entry(level.to_string()).or_default();
        entry.flights += 1;
        entry.landings += u32::from(run.landed);
        let new_best = entry.best.is_none_or(|best| run.beats(&best));
        if new_best {
            entry.best = Some(run);
        }
        new_best
    }
}

/// The flight that just ended, for the end screen.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct LastRun {
    pub record: Option<RunRecord>,
    pub new_best: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn landed(total: f32) -> RunRecord {
        RunRecord {
            landed: true,
            damage: Damage {
                oxygen: total,
                ..default()
            },
            survived: 240.0,
        }
    }

    fn lost(survived: f32) -> RunRecord {
        RunRecord {
            landed: false,
            damage: Damage::default(),
            survived,
        }
    }

    #[test]
    fn damage_adds_up_by_kind() {
        let mut d = Damage::default();
        d.add(FaultKind::HullBreach, 4.0);
        d.add(FaultKind::TrajectoryDrift, 2.5);
        d.add(FaultKind::LooseBolts, 1.0);
        d.add(FaultKind::HullBreach, 1.0);
        assert_eq!((d.oxygen, d.course, d.engine), (5.0, 2.5, 1.0));
        assert_eq!(d.total(), 8.5);
    }

    #[test]
    fn landing_beats_losing_and_less_damage_wins() {
        assert!(landed(90.0).beats(&lost(239.0)));
        assert!(!lost(239.0).beats(&landed(90.0)));
        assert!(landed(20.0).beats(&landed(30.0)));
        assert!(!landed(30.0).beats(&landed(20.0)));
        assert!(lost(120.0).beats(&lost(60.0)));
        assert!(!lost(60.0).beats(&lost(120.0)));
    }

    #[test]
    fn ties_in_shown_seconds_are_not_a_new_best() {
        assert!(!landed(20.4).beats(&landed(20.2)));
        assert!(!lost(60.2).beats(&lost(60.9)));
    }

    #[test]
    fn recording_keeps_the_best_and_counts_flights() {
        let mut p = Progress::default();
        assert!(p.record("one", lost(60.0)));
        assert!(p.record("one", landed(40.0)));
        assert!(!p.record("one", landed(55.0)));
        assert!(!p.record("one", lost(200.0)));
        assert!(p.record("one", landed(12.0)));
        let one = p.level("one").unwrap();
        assert_eq!((one.flights, one.landings), (5, 3));
        assert_eq!(one.best, Some(landed(12.0)));
        assert!(p.level("two").is_none());
    }
}
