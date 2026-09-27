//! Scenario fixtures: functions that put the world into an exact game
//! situation, so tests can start there instead of playing through to it.
//!
//! The same functions are used by native tests
//! (`tests/integration/common.rs::test_app_with`), by the web build under the
//! `e2e` feature (`?scenario=<name>`), and by `e2e/tools/capture.mjs --scenario`.
//! A scenario is applied on top of every fresh run (launch and each restart),
//! after the run's player, tools and fault plan exist. Any scenario skips the
//! title screen; the plain boot (no scenario) is the title and menu situation.
//! Scenarios skip level one's coaching too, except the ones made for it
//! ([`Scenario::coached`]).
//!
//! Adding one: add a variant, give it a name in [`Scenario::name`], add it to
//! [`Scenario::ALL`], and write its setup function below.

use bevy::prelude::*;

use crate::art::engineer::{contact, facing};
use crate::coach::{Tip, TipsSeen};
use crate::faults::{Site, Vitals, fault_bundle};
use crate::level::{CurrentLevel, FaultPlan, RunStats, campaign::level};
use crate::menu::{Menu, Screen};
use crate::player::{Facing, Player};
use crate::settings::Settings;
use crate::ship::layout;
use crate::tools::{Tool, ToolBelt};
use crate::upgrades::{Upgrade, Upgrades};
use crate::{AppState, Pause, RunSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scenario {
    /// Level one from launch, as a returning player gets it (no coaching).
    Default,
    /// No faults will ever start: free movement and tool use.
    Quiet,
    /// Loose bolts on the port engine's spine side; the engineer stands in
    /// front of the first bolt, facing it, wrench in hand.
    Bolts,
    /// A breach on the airlock's port wall; the engineer stands in front of
    /// it, facing it, tape in hand.
    Breach,
    /// Trajectory drift; the engineer stands at the helm seat.
    Drift,
    /// Bolts and a breach active; the engineer stands at the diagnostic console.
    Diagnostics,
    /// All three kinds at once; the engineer is in the quarters.
    Scramble,
    /// Level one with every fault fixed: the final approach, landing three
    /// seconds after launch.
    Landing,
    /// The `breach` scenario with two seconds of oxygen left.
    BreachCritical,
    /// The `breach` scenario with the air still at 40% from an earlier one:
    /// the new breach empties it in 40% of its clock.
    SecondBreach,
    /// The `breach` scenario with one second of tape left.
    TapeLow,
    /// Level one from launch, paused: the pause menu is open.
    Paused,
    /// Level one, paused, with the settings screen open over the pause menu.
    Settings,
    /// Level one as a first-time player gets it: every coaching tip still
    /// to come, so the launch waits for the pre-flight check at the console.
    FirstFlight,
    /// A first-time player's first fault: the pre-flight check done, loose
    /// bolts on the port engine with their tip up, the engineer in front of
    /// them with the wrench (as in `bolts`).
    FirstBolts,
    /// The last level from launch, with an upgrade picked before each level
    /// (run faster, faster wrench, wider tape, run faster).
    LevelFive,
    /// The `level_five` scenario with every fault fixed, three seconds from
    /// touchdown: landing ends the campaign.
    FinalLanding,
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
        Scenario::SecondBreach,
        Scenario::TapeLow,
        Scenario::Paused,
        Scenario::Settings,
        Scenario::FirstFlight,
        Scenario::FirstBolts,
        Scenario::LevelFive,
        Scenario::FinalLanding,
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
            Scenario::SecondBreach => "second_breach",
            Scenario::TapeLow => "tape_low",
            Scenario::Paused => "paused",
            Scenario::Settings => "settings",
            Scenario::FirstFlight => "first_flight",
            Scenario::FirstBolts => "first_bolts",
            Scenario::LevelFive => "level_five",
            Scenario::FinalLanding => "final_landing",
        }
    }

    /// Whether the flight coaches as a player's would. Only the scenarios
    /// made for the coaching do; the rest play as a returning player.
    pub fn coached(self) -> bool {
        matches!(self, Scenario::FirstFlight | Scenario::FirstBolts)
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
            Scenario::SecondBreach => second_breach(world),
            Scenario::TapeLow => tape_low(world),
            Scenario::Paused => paused(world),
            Scenario::Settings => settings(world),
            Scenario::FirstFlight => first_flight(world),
            Scenario::FirstBolts => first_bolts(world),
            Scenario::LevelFive => level_five(world),
            Scenario::FinalLanding => final_landing(world),
        }
    }
}

/// Out from a surface along `normal`, far enough that the tool held
/// facing it rests `short` units off the surface.
fn stand_off(point: Vec2, normal: Vec2, wrench: bool, short: f32) -> Vec2 {
    let dir16 = facing(-normal, 16).unwrap_or(0);
    let reach = contact(wrench, dir16).dot(-normal);
    point + normal * (reach + short)
}

/// Where the `bolts` scenario puts the engineer: in front of the first bolt
/// of the port engine's spine-side panel, the wrench's bite a few units
/// short of it (the snap pulls it on).
pub fn bolts_stand() -> Vec2 {
    let first = Site::PortEngineInner.bolts().expect("a bolt panel")[0];
    stand_off(first, Site::PortEngineInner.normal(), true, 4.0)
}

/// Where the `breach` scenario puts the engineer: in front of the breach,
/// the tape roll's edge pressed to the wall.
pub fn breach_stand() -> Vec2 {
    stand_off(
        Site::AirlockPortAft.pos(),
        Site::AirlockPortAft.normal(),
        false,
        -4.0,
    )
}

/// Where the `diagnostics` scenario puts the engineer: at the console.
pub fn console_stand() -> Vec2 {
    layout::console_point() - Vec2::Y * 30.0
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

/// The `breach` scenario's clock: a full tank lasts this long.
pub const BREACH_CLOCK: f32 = 50.0;

fn breach(world: &mut World) {
    quiet(world);
    start(world, Site::AirlockPortAft, BREACH_CLOCK);
    place_player(world, breach_stand(), -Site::AirlockPortAft.normal());
    hold(world, Tool::Tape);
}

fn drift(world: &mut World) {
    quiet(world);
    start(world, Site::Helm, 60.0);
    place_player(world, layout::helm_seat(), layout::ship().helm().facing);
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
    start(world, Site::AirlockHatchStarboard, 45.0);
    start(world, Site::Helm, 50.0);
}

/// Every planned fault counted as started and fixed, so the flight is
/// cleared on its first tick and lands after the final approach.
fn landing(world: &mut World) {
    let planned = std::mem::take(&mut world.resource_mut::<FaultPlan>().pending).len() as u32;
    let mut stats = world.resource_mut::<RunStats>();
    stats.started = planned;
    stats.fixed = planned;
}

fn breach_critical(world: &mut World) {
    breach(world);
    world.resource_mut::<Vitals>().oxygen.spent = 1.0 - 2.0 / BREACH_CLOCK;
}

fn second_breach(world: &mut World) {
    breach(world);
    world.resource_mut::<Vitals>().oxygen.spent = 0.6;
}

fn tape_low(world: &mut World) {
    breach(world);
    world.resource_mut::<ToolBelt>().tape_left = 1.0;
}

fn paused(world: &mut World) {
    world.resource_mut::<NextState<Pause>>().set(Pause::Paused);
}

fn settings(world: &mut World) {
    let mut menu = world.resource_mut::<Menu>();
    menu.open(Screen::Pause);
    menu.open(Screen::Settings);
    paused(world);
}

fn tips_seen(world: &mut World, seen: TipsSeen) {
    world.resource_mut::<Settings>().tips_seen = seen;
}

fn first_flight(world: &mut World) {
    tips_seen(world, TipsSeen::default());
}

fn first_bolts(world: &mut World) {
    bolts(world);
    let mut seen = TipsSeen::default();
    seen.mark(Tip::Preflight);
    tips_seen(world, seen);
}

/// Flies campaign level `number` instead of the one the run started with:
/// its fault plan replaces the one already rolled.
fn fly_level(world: &mut World, number: usize) {
    let def = level(number).expect("a campaign level");
    let plan = world.resource_scope(|_, mut rng: Mut<crate::GameRng>| def.roll(&mut rng));
    world.resource_mut::<FaultPlan>().pending = plan;
    world.insert_resource(CurrentLevel(def));
}

fn level_five(world: &mut World) {
    fly_level(world, 5);
    let mut upgrades = Upgrades::default();
    for (level, upgrade) in (2..).zip([
        Upgrade::RunFaster,
        Upgrade::FasterWrench,
        Upgrade::WiderTape,
        Upgrade::RunFaster,
    ]) {
        upgrades.choose(level, upgrade);
    }
    world.insert_resource(upgrades);
}

fn final_landing(world: &mut World) {
    level_five(world);
    landing(world);
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

pub fn apply_active_scenario(world: &mut World) {
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
    use crate::player::PLAYER_RADIUS;
    use crate::ship::layout::{colliders, resolve_circle, wall_contact, walls};
    use crate::tools::{SNAP_RADIUS, TAPE_CONTACT, tool_tip};

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
            assert_eq!(resolve_circle(stand, PLAYER_RADIUS, &colliders), stand);
        }
        let bolt = Site::PortEngineInner.bolts().unwrap()[0];
        let aim = -Site::PortEngineInner.normal();
        let tip = tool_tip(Tool::Wrench, bolts_stand(), aim, false);
        assert!(tip.distance(bolt) < SNAP_RADIUS);
        assert!(tip.distance(bolt) > 1.0, "the snap has something to pull");
        let aim = -Site::AirlockPortAft.normal();
        let tape_tip = tool_tip(Tool::Tape, breach_stand(), aim, false);
        let contact = wall_contact(breach_stand(), tape_tip, TAPE_CONTACT, &walls())
            .expect("tape reaches the wall");
        assert!(contact.point.distance(Site::AirlockPortAft.pos()) < BREACH_RADIUS);
    }
}
