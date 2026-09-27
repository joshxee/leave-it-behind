//! Upgrades: between levels the engineer picks one of three, kept for the
//! rest of the campaign. Picking the same one again stacks it. This is where
//! a roguelike choice of upgrades will grow; for now it is always the same
//! three.
//!
//! - [`Upgrade::RunFaster`]: walk faster (`player::move_player`).
//! - [`Upgrade::FasterWrench`]: each bolt turns in less time (`tools::use_wrench`).
//! - [`Upgrade::WiderTape`]: tape covers more: it counts toward a breach from
//!   further away and seals it faster (`faults::breach`), in wider strips.
//!
//! One upgrade is picked on the way into each of levels two to five.
//! Flying a level again and picking again replaces the pick for that level,
//! so replaying never stacks up extra upgrades. Upgrades are not saved: a
//! new game (PLAY on the main menu) starts without any.

use bevy::prelude::*;

use crate::level::LEVEL_COUNT;
use crate::tools::WRENCH_TURN_SECS;

/// Walking speed added per pick (of the base speed).
pub const SPEED_PER_PICK: f32 = 0.15;
/// A wrench turn takes this much of its time per pick.
pub const WRENCH_TIME_PER_PICK: f32 = 0.75;
/// Tape reach and sealing speed added per pick.
pub const TAPE_PER_PICK: f32 = 0.5;
/// The most picks a campaign can hold: one before each level after the first.
pub const MAX_PICKS: usize = LEVEL_COUNT - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Upgrade {
    RunFaster,
    FasterWrench,
    WiderTape,
}

impl Upgrade {
    /// In the order the upgrade screen offers them.
    pub const ALL: [Upgrade; 3] = [
        Upgrade::RunFaster,
        Upgrade::FasterWrench,
        Upgrade::WiderTape,
    ];

    /// Stable identifier (test bridge).
    pub fn as_str(self) -> &'static str {
        match self {
            Upgrade::RunFaster => "RunFaster",
            Upgrade::FasterWrench => "FasterWrench",
            Upgrade::WiderTape => "WiderTape",
        }
    }

    /// The upgrade screen's button.
    pub fn label(self) -> &'static str {
        match self {
            Upgrade::RunFaster => "RUN FASTER",
            Upgrade::FasterWrench => "FASTER WRENCH",
            Upgrade::WiderTape => "WIDER TAPE",
        }
    }

    /// What one pick does.
    pub fn describe(self) -> &'static str {
        match self {
            Upgrade::RunFaster => "Run faster: move 15% faster.",
            Upgrade::FasterWrench => "Faster wrench: each bolt turns in 25% less time.",
            Upgrade::WiderTape => "Wider tape: seals a hole from further away, 50% faster.",
        }
    }
}

/// The upgrades picked this campaign, in order: `picks[0]` was picked on the
/// way into level two, `picks[1]` into level three, and so on.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Upgrades {
    picks: Vec<Upgrade>,
}

impl Upgrades {
    pub fn picks(&self) -> &[Upgrade] {
        &self.picks
    }

    /// Times `upgrade` has been picked.
    pub fn count(&self, upgrade: Upgrade) -> usize {
        self.picks.iter().filter(|&&u| u == upgrade).count()
    }

    /// Picks `upgrade` on the way into level `level` (2 to [`LEVEL_COUNT`]).
    /// Anything picked for that level or later (a replayed level) is dropped
    /// first.
    pub fn choose(&mut self, level: usize, upgrade: Upgrade) {
        let before = level.clamp(2, LEVEL_COUNT) - 2;
        self.picks.truncate(before);
        self.picks.push(upgrade);
    }

    /// Walking speed, as a multiple of the base speed.
    pub fn speed_factor(&self) -> f32 {
        1.0 + SPEED_PER_PICK * self.count(Upgrade::RunFaster) as f32
    }

    /// Seconds one wrench turn takes.
    pub fn wrench_secs(&self) -> f32 {
        WRENCH_TURN_SECS * WRENCH_TIME_PER_PICK.powi(self.count(Upgrade::FasterWrench) as i32)
    }

    /// Tape reach and sealing speed, as a multiple of the base.
    pub fn tape_factor(&self) -> f32 {
        1.0 + TAPE_PER_PICK * self.count(Upgrade::WiderTape) as f32
    }

    /// Every pick of one upgrade (tests, scenarios).
    pub fn all_of(upgrade: Upgrade, picks: usize) -> Self {
        Self {
            picks: vec![upgrade; picks.min(MAX_PICKS)],
        }
    }
}

pub struct UpgradesPlugin;

impl Plugin for UpgradesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Upgrades>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_picked_changes_nothing() {
        let u = Upgrades::default();
        assert_eq!(u.speed_factor(), 1.0);
        assert_eq!(u.wrench_secs(), WRENCH_TURN_SECS);
        assert_eq!(u.tape_factor(), 1.0);
    }

    #[test]
    fn picks_stack() {
        let mut u = Upgrades::default();
        u.choose(2, Upgrade::RunFaster);
        u.choose(3, Upgrade::RunFaster);
        u.choose(4, Upgrade::FasterWrench);
        u.choose(5, Upgrade::WiderTape);
        assert_eq!(u.count(Upgrade::RunFaster), 2);
        assert!((u.speed_factor() - 1.3).abs() < 1e-6);
        assert!((u.wrench_secs() - WRENCH_TURN_SECS * 0.75).abs() < 1e-6);
        assert_eq!(u.tape_factor(), 1.5);
    }

    #[test]
    fn picking_again_for_a_replayed_level_replaces_the_pick() {
        let mut u = Upgrades::default();
        u.choose(2, Upgrade::RunFaster);
        u.choose(3, Upgrade::WiderTape);
        // Level two flown again, then on into level three with another pick.
        u.choose(3, Upgrade::FasterWrench);
        assert_eq!(u.picks(), &[Upgrade::RunFaster, Upgrade::FasterWrench]);
        // Level one flown again from a new pick for level two.
        u.choose(2, Upgrade::WiderTape);
        assert_eq!(u.picks(), &[Upgrade::WiderTape]);
    }

    #[test]
    fn a_full_campaign_holds_one_pick_per_level_after_the_first() {
        let mut u = Upgrades::default();
        for level in 2..=LEVEL_COUNT {
            u.choose(level, Upgrade::RunFaster);
        }
        assert_eq!(u.picks().len(), MAX_PICKS);
        assert_eq!(Upgrades::all_of(Upgrade::RunFaster, 9), u);
    }

    #[test]
    fn upgrade_text_is_ascii() {
        for u in Upgrade::ALL {
            assert!(u.label().is_ascii() && u.describe().is_ascii(), "{u:?}");
        }
    }
}
