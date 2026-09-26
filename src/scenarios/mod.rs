//! Scenario fixtures: functions that put the world into an exact game
//! situation, so tests can start there instead of playing through to it.
//!
//! The same functions are used by native tests
//! (`tests/integration/common.rs::test_app_with`), by the web build under the
//! `e2e` feature (`?scenario=<name>`), and by `e2e/capture.mjs --scenario`.
//!
//! Adding one: add a variant, give it a name in [`Scenario::name`], add it to
//! [`Scenario::ALL`], and write its setup function below.

use bevy::prelude::*;

use crate::GameSet;
use crate::scoring::Score;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scenario {
    /// Fresh game, nothing changed.
    Default,
    /// Score is one point below `SCORE_THRESHOLD`.
    ScoreNine,
}

impl Scenario {
    pub const ALL: &[Scenario] = &[Scenario::Default, Scenario::ScoreNine];

    pub fn name(self) -> &'static str {
        match self {
            Scenario::Default => "default",
            Scenario::ScoreNine => "score_nine",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|s| s.name() == name)
    }

    /// Runs after all `Startup` spawning, so it can edit spawned entities.
    pub fn apply(self, world: &mut World) {
        match self {
            Scenario::Default => {}
            Scenario::ScoreNine => score_nine(world),
        }
    }
}

fn score_nine(world: &mut World) {
    world.insert_resource(Score(9));
}

/// Scenario to apply at startup. Insert before the first `app.update()`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveScenario(pub Scenario);

pub struct ScenarioPlugin;

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "e2e")]
        if let Some(scenario) = requested_scenario() {
            app.insert_resource(ActiveScenario(scenario));
        }
        app.add_systems(Startup, apply_active_scenario.in_set(GameSet::Simulate));
    }
}

fn apply_active_scenario(world: &mut World) {
    if let Some(ActiveScenario(scenario)) = world.get_resource::<ActiveScenario>().copied() {
        info!("applying scenario {}", scenario.name());
        scenario.apply(world);
    }
}

/// `?scenario=<name>` on the web, `SCENARIO=<name>` natively.
#[cfg(feature = "e2e")]
fn requested_scenario() -> Option<Scenario> {
    #[cfg(target_arch = "wasm32")]
    let name = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .and_then(|q| web_sys::UrlSearchParams::new_with_str(&q).ok())
        .and_then(|p| p.get("scenario"));
    #[cfg(not(target_arch = "wasm32"))]
    let name = std::env::var("SCENARIO").ok();

    let name = name?;
    let scenario = Scenario::from_name(&name);
    if scenario.is_none() {
        error!("unknown scenario {name:?}");
    }
    scenario
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for &s in Scenario::ALL {
            assert_eq!(Scenario::from_name(s.name()), Some(s));
        }
    }
}
