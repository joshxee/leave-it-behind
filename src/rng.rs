//! The single source of randomness. Gameplay code never calls `rand::rng()`
//! or `thread_rng()`; it takes `ResMut<GameRng>` instead so tests and e2e
//! runs can fix the seed.

use bevy::prelude::*;
use rand::{RngExt, SeedableRng, rngs::SmallRng};

#[derive(Resource)]
pub struct GameRng(SmallRng);

impl GameRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(SmallRng::seed_from_u64(seed))
    }

    pub fn from_entropy() -> Self {
        Self::from_seed(getrandom::u64().unwrap_or(0x5eed))
    }

    /// Uniform float in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        self.0.random::<f32>()
    }

    /// Uniform float in `[lo, hi)`. Returns `lo` without drawing when the
    /// range is empty, so pinned values never consume randomness.
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        if hi <= lo {
            return lo;
        }
        lo + (hi - lo) * self.unit()
    }

    /// Index picked with probability proportional to `weights`. Returns 0
    /// without drawing when there is at most one option.
    pub fn weighted(&mut self, weights: &[f32]) -> usize {
        if weights.len() <= 1 {
            return 0;
        }
        let total: f32 = weights.iter().map(|w| w.max(0.0)).sum();
        let mut roll = self.unit() * total;
        for (i, w) in weights.iter().enumerate() {
            roll -= w.max(0.0);
            if roll < 0.0 {
                return i;
            }
        }
        weights.len() - 1
    }
}

pub struct RngPlugin;

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        // TestDeterminismPlugin may already have inserted a seeded one.
        if !app.world().contains_resource::<GameRng>() {
            app.insert_resource(GameRng::from_entropy());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_range_and_single_option_draw_nothing() {
        let mut a = GameRng::from_seed(7);
        let mut b = GameRng::from_seed(7);
        assert_eq!(a.range(3.0, 3.0), 3.0);
        assert_eq!(a.weighted(&[5.0]), 0);
        // `a` drew nothing, so both streams are still in step.
        assert_eq!(a.unit(), b.unit());
    }

    #[test]
    fn weighted_never_picks_zero_weight() {
        let mut rng = GameRng::from_seed(1);
        for _ in 0..1000 {
            assert_ne!(rng.weighted(&[1.0, 0.0, 2.0]), 1);
        }
    }

    #[test]
    fn range_stays_in_bounds() {
        let mut rng = GameRng::from_seed(2);
        for _ in 0..1000 {
            let v = rng.range(45.0, 60.0);
            assert!((45.0..60.0).contains(&v));
        }
    }
}
