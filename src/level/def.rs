//! The level format: a flight of fixed length plus a sequence of fault
//! slots. Each slot has a timing window (when it may start), weighted
//! choices (which fault, optionally at which site) and a failure-clock
//! window. Rolling a level turns every slot into one concrete
//! [`PlannedFault`]. A level whose windows are all zero-width, with one
//! choice per slot that pins its site, is fully deterministic: rolling it
//! draws no randomness at all.
//!
//! The [`Envelope`] bounds difficulty: assuming the engineer handles each
//! fault within `response_secs`, no rolled schedule may ever ask for more than
//! `max_overlap` faults at once. `validate` checks the worst case over every
//! possible roll, so randomized levels stay inside the same envelope.

use crate::GameRng;
use crate::faults::{CLOCK_LIMITS, FaultKind, Site};

/// Inclusive range of seconds. `from == to` pins the value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeWindow {
    pub from: f32,
    pub to: f32,
}

impl TimeWindow {
    pub const fn new(from: f32, to: f32) -> Self {
        Self { from, to }
    }

    pub const fn at(t: f32) -> Self {
        Self { from: t, to: t }
    }

    pub fn is_pinned(&self) -> bool {
        self.from == self.to
    }

    fn roll(&self, rng: &mut GameRng) -> f32 {
        rng.range(self.from, self.to)
    }
}

/// One weighted option for a slot. Without a site, any site of the kind is
/// picked, each as likely as the others.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Choice {
    pub weight: f32,
    pub kind: FaultKind,
    pub site: Option<Site>,
}

impl Choice {
    pub fn pinned(site: Site) -> Self {
        Self {
            weight: 1.0,
            kind: site.kind(),
            site: Some(site),
        }
    }

    pub const fn any(kind: FaultKind, weight: f32) -> Self {
        Self {
            weight,
            kind,
            site: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FaultSlot {
    /// Seconds after launch the fault may start.
    pub window: TimeWindow,
    pub choices: Vec<Choice>,
    /// Failure clock in seconds.
    pub clock: TimeWindow,
}

impl FaultSlot {
    /// A fully pinned slot: this site, at this time, with this clock.
    pub fn pinned(at: f32, site: Site, clock: f32) -> Self {
        Self {
            window: TimeWindow::at(at),
            choices: vec![Choice::pinned(site)],
            clock: TimeWindow::at(clock),
        }
    }

    /// Any site of one of `kinds` (equally likely), starting somewhere in
    /// `window` with a clock somewhere in `clock`.
    pub fn any(window: TimeWindow, kinds: &[FaultKind], clock: TimeWindow) -> Self {
        Self {
            window,
            choices: kinds.iter().map(|&kind| Choice::any(kind, 1.0)).collect(),
            clock,
        }
    }

    /// Whether this slot can start a fault of `kind`.
    pub fn can_be(&self, kind: FaultKind) -> bool {
        self.choices.iter().any(|c| c.kind == kind)
    }

    pub fn is_pinned(&self) -> bool {
        self.window.is_pinned()
            && self.clock.is_pinned()
            && matches!(self.choices.as_slice(), [Choice { site: Some(_), .. }])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    /// Most faults that may need handling at the same time.
    pub max_overlap: usize,
    /// Assumed time to reach and fix one fault.
    pub response_secs: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelDef {
    /// Stable key for saved progress. Never change it once released.
    pub id: String,
    pub name: String,
    /// Seconds from launch to landing.
    pub duration_secs: f32,
    /// Reseeds `GameRng` at the start of every run, making everything
    /// random in the run (drift headings too) repeatable.
    pub seed: Option<u64>,
    pub envelope: Envelope,
    pub slots: Vec<FaultSlot>,
    /// First-time players are coached through it: a pre-flight check at
    /// the diagnostic screen, then a tip for each kind's first fault (`coach`).
    pub coaching: bool,
}

/// One fault a rolled level will start.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlannedFault {
    /// Seconds after launch.
    pub at: f32,
    pub site: Site,
    /// Failure clock in seconds.
    pub clock: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LevelError {
    Duration(f32),
    NoChoices { slot: usize },
    Weight { slot: usize },
    SiteKind { slot: usize },
    Window { slot: usize },
    Clock { slot: usize },
    Envelope { worst: usize, max: usize },
}

/// Bounds on a level's length, in seconds.
pub const DURATION_LIMITS: (f32, f32) = (60.0, 600.0);

impl LevelDef {
    pub fn is_pinned(&self) -> bool {
        self.slots.iter().all(FaultSlot::is_pinned)
    }

    /// Every problem with the definition, or `Ok` if there are none.
    pub fn validate(&self) -> Result<(), Vec<LevelError>> {
        let mut errors = Vec::new();
        if !(DURATION_LIMITS.0..=DURATION_LIMITS.1).contains(&self.duration_secs) {
            errors.push(LevelError::Duration(self.duration_secs));
        }
        for (slot, s) in self.slots.iter().enumerate() {
            if s.choices.is_empty() {
                errors.push(LevelError::NoChoices { slot });
            }
            if s.choices
                .iter()
                .any(|c| c.weight <= 0.0 || !c.weight.is_finite())
            {
                errors.push(LevelError::Weight { slot });
            }
            if s.choices
                .iter()
                .any(|c| c.site.is_some_and(|site| site.kind() != c.kind))
            {
                errors.push(LevelError::SiteKind { slot });
            }
            if s.window.from < 0.0
                || s.window.from > s.window.to
                || s.window.to >= self.duration_secs
            {
                errors.push(LevelError::Window { slot });
            }
            if s.clock.from < CLOCK_LIMITS.0
                || s.clock.from > s.clock.to
                || s.clock.to > CLOCK_LIMITS.1
            {
                errors.push(LevelError::Clock { slot });
            }
        }
        let worst = self.worst_case_overlap();
        if worst > self.envelope.max_overlap {
            errors.push(LevelError::Envelope {
                worst,
                max: self.envelope.max_overlap,
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Most slots that could need handling at once over every possible roll:
    /// slot `i` is busy somewhere in `[window.from, window.to + response]`.
    pub fn worst_case_overlap(&self) -> usize {
        let spans = self
            .slots
            .iter()
            .map(|s| (s.window.from, s.window.to + self.envelope.response_secs));
        max_overlap(spans)
    }

    /// Turns every slot into a concrete fault, sorted by start time. A
    /// pinned level draws nothing from `rng`.
    pub fn roll(&self, rng: &mut GameRng) -> Vec<PlannedFault> {
        let mut plan: Vec<PlannedFault> = self
            .slots
            .iter()
            .filter(|s| !s.choices.is_empty())
            .map(|s| {
                let at = s.window.roll(rng);
                let weights: Vec<f32> = s.choices.iter().map(|c| c.weight).collect();
                let choice = s.choices[rng.weighted(&weights)];
                let site = choice.site.unwrap_or_else(|| {
                    let sites = choice.kind.sites();
                    sites[rng.pick(sites.len())]
                });
                PlannedFault {
                    at,
                    site,
                    clock: s.clock.roll(rng),
                }
            })
            .collect();
        plan.sort_by(|a, b| a.at.total_cmp(&b.at));
        plan
    }
}

/// Most faults in `plan` busy at the same moment, each busy for
/// `response_secs` after it starts.
pub fn peak_overlap(plan: &[PlannedFault], response_secs: f32) -> usize {
    max_overlap(plan.iter().map(|p| (p.at, p.at + response_secs)))
}

/// Largest number of half-open spans `[start, end)` covering one point.
fn max_overlap(spans: impl Iterator<Item = (f32, f32)>) -> usize {
    let mut edges: Vec<(f32, i32)> = spans.flat_map(|(a, b)| [(a, 1), (b, -1)]).collect();
    // Ends sort before starts at the same instant: back-to-back is no overlap.
    edges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let (mut now, mut peak) = (0i32, 0i32);
    for (_, delta) in edges {
        now += delta;
        peak = peak.max(now);
    }
    peak as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_level() -> LevelDef {
        LevelDef {
            id: "test".into(),
            name: "test".into(),
            duration_secs: 200.0,
            seed: None,
            envelope: Envelope {
                max_overlap: 3,
                response_secs: 20.0,
            },
            slots: vec![
                FaultSlot {
                    window: TimeWindow::new(10.0, 30.0),
                    choices: vec![
                        Choice::any(FaultKind::HullBreach, 2.0),
                        Choice::any(FaultKind::LooseBolts, 1.0),
                    ],
                    clock: TimeWindow::new(55.0, 70.0),
                },
                FaultSlot {
                    window: TimeWindow::new(60.0, 90.0),
                    choices: vec![Choice::any(FaultKind::TrajectoryDrift, 1.0)],
                    clock: TimeWindow::new(60.0, 65.0),
                },
            ],
            coaching: false,
        }
    }

    #[test]
    fn rolls_stay_inside_their_windows() {
        let level = random_level();
        assert!(level.validate().is_ok());
        for seed in 0..200 {
            let plan = level.roll(&mut GameRng::from_seed(seed));
            assert_eq!(plan.len(), 2);
            assert!((10.0..=30.0).contains(&plan[0].at));
            assert!(matches!(
                plan[0].site.kind(),
                FaultKind::HullBreach | FaultKind::LooseBolts
            ));
            assert!((55.0..=70.0).contains(&plan[0].clock));
            assert_eq!(plan[1].site, Site::Helm);
            assert!((60.0..=90.0).contains(&plan[1].at));
        }
    }

    #[test]
    fn same_seed_same_plan_different_seed_different_plan() {
        let level = random_level();
        let a = level.roll(&mut GameRng::from_seed(1));
        assert_eq!(a, level.roll(&mut GameRng::from_seed(1)));
        assert!((2..50).any(|s| level.roll(&mut GameRng::from_seed(s)) != a));
    }

    #[test]
    fn weights_bias_the_pick() {
        let level = random_level();
        let breaches = (0..2000)
            .filter(|&s| {
                level.roll(&mut GameRng::from_seed(s))[0].site.kind() == FaultKind::HullBreach
            })
            .count();
        // Weight 2 vs 1: about two thirds.
        assert!((1150..1500).contains(&breaches), "{breaches}");
    }

    #[test]
    fn unpinned_breaches_reach_every_breach_site() {
        let level = random_level();
        let mut seen = std::collections::HashSet::new();
        for s in 0..2000 {
            let site = level.roll(&mut GameRng::from_seed(s))[0].site;
            if site.kind() == FaultKind::HullBreach {
                seen.insert(site);
            }
        }
        assert_eq!(seen.len(), crate::faults::sites::BREACH_SITES.len());
    }

    #[test]
    fn any_slots_offer_each_kind_equally() {
        let slot = FaultSlot::any(
            TimeWindow::new(5.0, 9.0),
            &[FaultKind::LooseBolts, FaultKind::HullBreach],
            TimeWindow::at(60.0),
        );
        assert!(slot.can_be(FaultKind::HullBreach));
        assert!(!slot.can_be(FaultKind::TrajectoryDrift));
        assert!(!slot.is_pinned());
        assert!(
            slot.choices
                .iter()
                .all(|c| c.weight == 1.0 && c.site.is_none())
        );
    }

    #[test]
    fn pinned_slots_draw_no_randomness() {
        let level = LevelDef {
            slots: vec![FaultSlot::pinned(5.0, Site::Helm, 50.0)],
            ..random_level()
        };
        assert!(level.is_pinned());
        let mut rng = GameRng::from_seed(9);
        let plan = level.roll(&mut rng);
        assert_eq!(
            plan,
            vec![PlannedFault {
                at: 5.0,
                site: Site::Helm,
                clock: 50.0
            }]
        );
        assert_eq!(rng.unit(), GameRng::from_seed(9).unit());
    }

    #[test]
    fn validation_reports_every_problem() {
        let mut level = random_level();
        level.duration_secs = 25.0;
        level.slots[0].clock = TimeWindow::new(30.0, 80.0);
        level.slots[1].choices = vec![Choice {
            weight: 0.0,
            kind: FaultKind::LooseBolts,
            site: Some(Site::Helm),
        }];
        let errors = level.validate().unwrap_err();
        assert!(errors.contains(&LevelError::Duration(25.0)));
        assert!(errors.contains(&LevelError::Clock { slot: 0 }));
        assert!(errors.contains(&LevelError::Weight { slot: 1 }));
        assert!(errors.contains(&LevelError::SiteKind { slot: 1 }));
        assert!(errors.contains(&LevelError::Window { slot: 1 }));
    }

    #[test]
    fn envelope_caps_worst_case_overlap() {
        let mut level = random_level();
        level.envelope.max_overlap = 1;
        // Slot 0 can run until 30 + 20 = 50; slot 1 starts at 60: no overlap.
        assert_eq!(level.worst_case_overlap(), 1);
        assert!(level.validate().is_ok());
        level.slots[1].window = TimeWindow::new(40.0, 90.0);
        assert_eq!(level.worst_case_overlap(), 2);
        assert_eq!(
            level.validate().unwrap_err(),
            vec![LevelError::Envelope { worst: 2, max: 1 }]
        );
    }

    #[test]
    fn back_to_back_faults_do_not_overlap() {
        let plan = [
            PlannedFault {
                at: 0.0,
                site: Site::Helm,
                clock: 50.0,
            },
            PlannedFault {
                at: 20.0,
                site: Site::HullPortAft,
                clock: 50.0,
            },
        ];
        assert_eq!(peak_overlap(&plan, 20.0), 1);
        assert_eq!(peak_overlap(&plan, 20.5), 2);
    }
}
