//! Scenario fixtures: functions that put the world into an exact game
//! situation, so tests can start there instead of playing through to it.
//!
//! The same functions are used by native tests
//! (`tests/integration/common.rs::test_app_with`), by the web build under the
//! `e2e` feature (`?scenario=<name>`), and by `e2e/tools/capture.mjs --scenario`.
//! A scenario is applied on top of every fresh run (launch and each restart),
//! after the run's player, tools and fault plan exist.
//!
//! Adding one: add a variant, give it a name in [`Scenario::name`], add it to
//! [`Scenario::ALL`], and write its setup function below.

use bevy::prelude::*;

use crate::faults::{Fault, Site, fault_bundle};
use crate::level::{FaultPlan, Journey};
use crate::player::{Facing, Player};
use crate::ship::layout;
use crate::tools::{Tool, ToolBelt};
use crate::{AppState, RunSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scenario {
    /// Level one from launch, as a player gets it.
    Default,
    /// No faults will ever start: free movement and tool use.
    Quiet,
    /// Loose bolts on the upper engine's spine side; the engineer stands
    /// below the first bolt, facing it, wrench in hand.
    Bolts,
    /// A breach on the airlock's upper wall; the engineer stands below it,
    /// facing it, tape in hand.
    Breach,
    /// Trajectory drift; the engineer stands at the helm seat.
    Drift,
    /// Bolts and a breach active; the engineer stands at the diagnostic console.
    Diagnostics,
    /// All three kinds at once; the engineer is in the quarters.
    Scramble,
    /// Level one with three seconds to arrival and nothing broken.
    Landing,
    /// The `breach` scenario with two seconds left on the breach's clock.
    BreachCritical,
    /// The `breach` scenario with one second of tape left.
    TapeLow,
}

impl Scenario {
    pub const ALL: &[Scenario] = &[
        Scenario::Default,
        Scenario::Quiet,
        Scenario::Bolts,
        Scenario::Breach,
        Scenario::Drift,
        Scenario::Diagnostics,
        Scenario::Scramble,
        Scenario::Landing,
        Scenario::BreachCritical,
        Scenario::TapeLow,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Scenario::Default => "default",
            Scenario::Quiet => "quiet",
            Scenario::Bolts => "bolts",
            Scenario::Breach => "breach",
            Scenario::Drift => "drift",
            Scenario::Diagnostics => "diagnostics",
            Scenario::Scramble => "scramble",
            Scenario::Landing => "landing",
            Scenario::BreachCritical => "breach_critical",
            Scenario::TapeLow => "tape_low",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|s| s.name() == name)
    }

    /// Runs after the run's player, tools and fault plan are spawned.
    pub fn apply(self, world: &mut World) {
        match self {
            Scenario::Default => {}
            Scenario::Quiet => quiet(world),
            Scenario::Bolts => bolts(world),
            Scenario::Breach => breach(world),
            Scenario::Drift => drift(world),
            Scenario::Diagnostics => diagnostics(world),
            Scenario::Scramble => scramble(world),
            Scenario::Landing => landing(world),
            Scenario::BreachCritical => breach_critical(world),
            Scenario::TapeLow => tape_low(world),
        }
    }
}

/// Where the `bolts` scenario puts the engineer: below the first bolt of
/// the upper engine's spine-side panel, the wrench head resting on it.
pub fn bolts_stand() -> Vec2 {
    let first = Site::PortEngineInner.bolts().expect("a bolt panel")[0];
    first + Site::PortEngineInner.normal() * 44.0
}

/// Where the `breach` scenario puts the engineer: just below the breach.
pub fn breach_stand() -> Vec2 {
    Site::AirlockPortAft.pos() + Site::AirlockPortAft.normal() * 22.0
}

/// Where the `diagnostics` scenario puts the engineer: at the console.
pub fn console_stand() -> Vec2 {
    let console = layout::diag_console();
    Vec2::new(console.center().x, console.min.y - 30.0)
}

fn place_player(world: &mut World, pos: Vec2, facing: Vec2) {
    let mut q = world.query_filtered::<(&mut Transform, &mut Facing), With<Player>>();
    for (mut transform, mut f) in q.iter_mut(world) {
        transform.translation = pos.extend(transform.translation.z);
        f.0 = facing;
    }
}

fn hold(world: &mut World, tool: Tool) {
    world.resource_mut::<ToolBelt>().held = tool;
}

fn start(world: &mut World, site: Site, clock: f32) {
    world.spawn(fault_bundle(site, clock));
}

fn quiet(world: &mut World) {
    world.resource_mut::<FaultPlan>().pending.clear();
}

fn bolts(world: &mut World) {
    quiet(world);
    start(world, Site::PortEngineInner, 55.0);
    place_player(world, bolts_stand(), -Site::PortEngineInner.normal());
    hold(world, Tool::Wrench);
}

fn breach(world: &mut World) {
    quiet(world);
    start(world, Site::AirlockPortAft, 50.0);
    place_player(world, breach_stand(), -Site::AirlockPortAft.normal());
    hold(world, Tool::Tape);
}

fn drift(world: &mut World) {
    quiet(world);
    start(world, Site::Helm, 60.0);
    place_player(world, layout::helm_seat(), Vec2::X);
}

fn diagnostics(world: &mut World) {
    quiet(world);
    start(world, Site::PortEngineInner, 55.0);
    start(world, Site::AirlockPortAft, 50.0);
    place_player(world, console_stand(), Vec2::Y);
}

fn scramble(world: &mut World) {
    quiet(world);
    start(world, Site::StarboardEngineOuter, 50.0);
    start(world, Site::AirlockHatchLower, 45.0);
    start(world, Site::Helm, 50.0);
}

fn landing(world: &mut World) {
    quiet(world);
    let mut journey = world.resource_mut::<Journey>();
    journey.elapsed = journey.duration - 3.0;
}

fn breach_critical(world: &mut World) {
    breach(world);
    let mut q = world.query::<&mut Fault>();
    for mut fault in q.iter_mut(world) {
        fault.elapsed = fault.clock - 2.0;
    }
}

fn tape_low(world: &mut World) {
    breach(world);
    world.resource_mut::<ToolBelt>().tape_left = 1.0;
}

/// Scenario to apply to every run. Insert before the first `app.update()`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ActiveScenario(pub Scenario);

pub struct ScenarioPlugin;

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "e2e")]
        if let Some(scenario) = requested_scenario() {
            app.insert_resource(ActiveScenario(scenario));
        }
        app.add_systems(
            OnEnter(AppState::Playing),
            apply_active_scenario.in_set(RunSet::Scenario),
        );
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
    use crate::faults::breach::BREACH_RADIUS;
    use crate::ship::layout::{colliders, resolve_circle, wall_contact, walls};
    use crate::tools::{SNAP_RADIUS, TAPE_CONTACT, TAPE_REACH, WRENCH_REACH};

    #[test]
    fn names_round_trip() {
        for &s in Scenario::ALL {
            assert_eq!(Scenario::from_name(s.name()), Some(s));
        }
    }

    #[test]
    fn stands_are_free_and_in_reach() {
        let colliders = colliders();
        for stand in [
            bolts_stand(),
            breach_stand(),
            console_stand(),
            layout::helm_seat(),
        ] {
            assert_eq!(resolve_circle(stand, 18.0, &colliders), stand);
        }
        let bolt = Site::PortEngineInner.bolts().unwrap()[0];
        let tip = bolts_stand() - Site::PortEngineInner.normal() * WRENCH_REACH;
        assert!(tip.distance(bolt) < SNAP_RADIUS);
        let tape_tip = breach_stand() - Site::AirlockPortAft.normal() * TAPE_REACH;
        let contact =
            wall_contact(tape_tip, TAPE_CONTACT, &walls()).expect("tape reaches the wall");
        assert!(contact.point.distance(Site::AirlockPortAft.pos()) < BREACH_RADIUS);
    }
}
