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
