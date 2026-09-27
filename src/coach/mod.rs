//! Coaching: level one's first flight walks a new player through the game.
//! It never pauses anything; it only points the way.
//!
//! - Pre-flight check: the flight waits to launch (the countdown and the
//!   fault schedule hold, see [`Journey::launched`]) until the engineer has
//!   looked at the diagnostic screen. Then it launches as a normal flight,
//!   with a short note on what that screen is for.
//! - Tips: the first fault of each kind brings up a tip on how to fix it,
//!   shown until a fault of that kind is fixed.
//!
//! Each tip shows once ([`TipsSeen`], saved with the settings): a tip still
//! on screen when the flight ends counts as seen. The TIPS setting reads ON
//! while any tip is still to come; switching it on brings them all back.
//! Scenarios skip the coaching, except the ones made for it.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::diagnostics::{DiagView, Diagnostics};
use crate::faults::bolts::Bolt;
use crate::faults::{Fault, FaultFixed, FaultKind, Site, resolve_faults};
use crate::level::{CurrentLevel, Journey};
use crate::player::{InteractKind, InteractPressed};
use crate::scenarios::apply_active_scenario;
use crate::settings::Settings;
use crate::shapes::Shapes;
use crate::ship::layout;
use crate::tools::{TapeLaid, ToolState, WrenchTightened};
use crate::ui::game_font;
use crate::{ActiveScenario, AppState, GameSet, RunSet, palette, running};

/// Shown for this long once the pre-flight check launches the flight.
pub const LAUNCH_NOTE_SECS: f32 = 8.0;
pub const LAUNCH_NOTE: &str =
    "Cleared for launch. When the alarm goes off, check this console to find the fault.";
/// Above the HUD (10) and the diagnostic screen (20), below menus (40).
const COACH_Z: i32 = 25;
/// Just under the countdown, clear of the diagnostic screen's panel.
const PANEL_TOP: f32 = 56.0;
const PANEL_MAX_WIDTH: f32 = 900.0;

/// One piece of coaching, shown once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tip {
    /// Before launch: look at the diagnostic screen.
    Preflight,
    LooseBolts,
    HullBreach,
    TrajectoryDrift,
}

impl Tip {
    pub const ALL: [Tip; 4] = [
        Tip::Preflight,
        Tip::LooseBolts,
        Tip::HullBreach,
        Tip::TrajectoryDrift,
    ];

    /// The tip for the first fault of `kind`.
    pub fn for_fault(kind: FaultKind) -> Tip {
        match kind {
            FaultKind::LooseBolts => Tip::LooseBolts,
            FaultKind::HullBreach => Tip::HullBreach,
            FaultKind::TrajectoryDrift => Tip::TrajectoryDrift,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Tip::Preflight => "Pre-flight check: walk to the diagnostic console and press E.",
            Tip::LooseBolts => "Use the wrench [1] to screw the bolts back into the engine.",
            Tip::HullBreach => "The hull has been breached. Use the tape [2] to seal the hole.",
            Tip::TrajectoryDrift => {
                "Take your seat at the cockpit and realign the ship back on course."
            }
        }
    }
}

/// The tips a player has been shown. Saved with the settings; missing
/// fields (a tip added later) read as not seen.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TipsSeen {
    pub preflight: bool,
    pub loose_bolts: bool,
    pub hull_breach: bool,
    pub trajectory_drift: bool,
}

impl TipsSeen {
    /// Every tip seen: nothing left to coach.
    pub const EVERY: TipsSeen = TipsSeen {
        preflight: true,
        loose_bolts: true,
        hull_breach: true,
        trajectory_drift: true,
    };

    pub fn has(&self, tip: Tip) -> bool {
        match tip {
            Tip::Preflight => self.preflight,
            Tip::LooseBolts => self.loose_bolts,
            Tip::HullBreach => self.hull_breach,
            Tip::TrajectoryDrift => self.trajectory_drift,
        }
    }

    pub fn mark(&mut self, tip: Tip) {
        let seen = match tip {
            Tip::Preflight => &mut self.preflight,
            Tip::LooseBolts => &mut self.loose_bolts,
            Tip::HullBreach => &mut self.hull_breach,
            Tip::TrajectoryDrift => &mut self.trajectory_drift,
        };
        *seen = true;
    }

    /// Whether every tip has been shown.
    pub fn all(&self) -> bool {
        Tip::ALL.into_iter().all(|tip| self.has(tip))
    }
}

/// This flight's coaching.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Coach {
    /// The flight may coach: its level asks for it and no scenario skips it.
    /// What it shows still depends on the tips seen.
    pub active: bool,
    /// Seconds left of the launch note.
    pub launch_note: f32,
}

/// What the coaching panel says, top to bottom: the pre-flight check while
/// the launch waits for it, the launch note, then one tip per kind of fault
/// with an unseen tip, oldest fault first.
pub fn lines<'a>(
    coach: &Coach,
    journey: &Journey,
    seen: &TipsSeen,
    faults: impl IntoIterator<Item = &'a Fault>,
) -> Vec<&'static str> {
    if !coach.active {
        return Vec::new();
    }
    let mut out = Vec::new();
    if !journey.launched {
        out.push(Tip::Preflight.text());
    }
    if coach.launch_note > 0.0 {
        out.push(LAUNCH_NOTE);
    }
    // Each kind with its oldest unfixed fault's age.
    let mut kinds: Vec<(FaultKind, f32)> = Vec::new();
    for fault in faults {
        if fault.is_repaired() || seen.has(Tip::for_fault(fault.kind())) {
            continue;
        }
        match kinds.iter_mut().find(|(kind, _)| *kind == fault.kind()) {
            Some((_, age)) => *age = age.max(fault.elapsed),
            None => kinds.push((fault.kind(), fault.elapsed)),
        }
    }
    let rank = |kind: FaultKind| FaultKind::ALL.iter().position(|&k| k == kind);
    kinds.sort_by(|a, b| b.1.total_cmp(&a.1).then(rank(a.0).cmp(&rank(b.0))));
    out.extend(
        kinds
            .into_iter()
            .map(|(kind, _)| Tip::for_fault(kind).text()),
    );
    out
}

/// The coaching panel (hidden when there is nothing to say).
#[derive(Component, Debug)]
pub struct CoachPanel;

#[derive(Component, Debug)]
pub struct CoachText;

pub struct CoachPlugin;

impl Plugin for CoachPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Coach>()
            .add_systems(Startup, spawn_panel.in_set(GameSet::Input))
            .add_systems(
                OnEnter(AppState::Playing),
                start_coaching
                    .in_set(RunSet::Scenario)
                    .after(apply_active_scenario),
            )
            .add_systems(
                FixedUpdate,
                (
                    (preflight_check, tick_launch_note).chain(),
                    note_fixes.after(resolve_faults),
                )
                    .in_set(GameSet::Resolve)
                    .run_if(running),
            )
            .add_systems(OnExit(AppState::Playing), finish_coaching)
            .add_systems(Startup, spawn_pings.in_set(GameSet::Input))
            .add_systems(
                FixedUpdate,
                note_ping_interactions
                    .in_set(GameSet::Resolve)
                    .after(resolve_faults),
            )
            .add_systems(Update, (draw_panel, draw_pings).in_set(GameSet::Present));
    }
}

/// Decides whether this flight coaches, and holds the launch for the
/// pre-flight check if it has not been done. Runs after the scenario, which
/// may set the tips up.
fn start_coaching(
    scenario: Option<Res<ActiveScenario>>,
    level: Res<CurrentLevel>,
    settings: Res<Settings>,
    mut coach: ResMut<Coach>,
    mut journey: ResMut<Journey>,
) {
    let active = level.0.coaching && scenario.is_none_or(|s| s.0.coached());
    *coach = Coach {
        active,
        launch_note: 0.0,
    };
    if active && !settings.tips_seen.has(Tip::Preflight) {
        journey.launched = false;
    }
}

/// Launches the flight once the diagnostic screen is up (or the tips were
/// switched off meanwhile).
fn preflight_check(
    diag: Res<Diagnostics>,
    mut settings: ResMut<Settings>,
    mut coach: ResMut<Coach>,
    mut journey: ResMut<Journey>,
) {
    if journey.launched {
        return;
    }
    if diag.view == DiagView::Open {
        settings.tips_seen.mark(Tip::Preflight);
        coach.launch_note = LAUNCH_NOTE_SECS;
        journey.launched = true;
    } else if !coach.active || settings.tips_seen.has(Tip::Preflight) {
        journey.launched = true;
    }
}

fn tick_launch_note(time: Res<Time>, mut coach: ResMut<Coach>) {
    if coach.launch_note > 0.0 {
        coach.launch_note = (coach.launch_note - time.delta_secs()).max(0.0);
    }
}

/// Fixing a fault retires its kind's tip.
fn note_fixes(
    mut fixed: MessageReader<FaultFixed>,
    coach: Res<Coach>,
    mut settings: ResMut<Settings>,
) {
    for msg in fixed.read() {
        let tip = Tip::for_fault(msg.site.kind());
        if coach.active && !settings.tips_seen.has(tip) {
            settings.tips_seen.mark(tip);
        }
    }
}

/// Leaving a flight (landed, lost, restarted or abandoned) retires the tips
/// on screen: they were shown.
fn finish_coaching(coach: Res<Coach>, faults: Query<&Fault>, mut settings: ResMut<Settings>) {
    if !coach.active {
        return;
    }
    for fault in &faults {
        let tip = Tip::for_fault(fault.kind());
        if !settings.tips_seen.has(tip) {
            settings.tips_seen.mark(tip);
        }
    }
}

fn spawn_panel(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(PANEL_TOP),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(COACH_Z),
        children![(
            CoachPanel,
            Node {
                max_width: px(PANEL_MAX_WIDTH),
                padding: UiRect::axes(px(16), px(6)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(palette::UI_PANEL),
            BorderColor::all(palette::UI_ACCENT),
            Visibility::Hidden,
            children![(
                CoachText,
                Text::new(""),
                game_font(asset_server.as_deref(), 20.0),
                TextColor(palette::UI_TEXT),
                TextLayout::justify(Justify::Center),
            )],
        )],
    ));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum CoachPing {
    Diagnostic,
    Bolt,
    Breach,
    Chair,
}

fn spawn_pings(mut commands: Commands, shapes: Res<Shapes>) {
    let bolt = Site::PortEngineInner.bolts().expect("first bolt target")[0];
    for (kind, pos) in [
        (
            CoachPing::Diagnostic,
            layout::ship().center(layout::ship().console_cell()),
        ),
        (CoachPing::Bolt, bolt),
        (CoachPing::Breach, Site::AirlockPortAft.pos()),
        (CoachPing::Chair, layout::helm_seat()),
    ] {
        commands.spawn((
            kind,
            shapes.ring(24.0, Color::srgb_u8(45, 155, 255)),
            Transform::from_translation(pos.extend(8.0)),
            Visibility::Hidden,
        ));
    }
}

fn note_ping_interactions(
    mut presses: MessageReader<InteractPressed>,
    mut tightened: MessageReader<WrenchTightened>,
    mut tape: MessageReader<TapeLaid>,
    bolts: Query<&Bolt>,
    tools: Res<ToolState>,
    mut settings: ResMut<Settings>,
) {
    for press in presses.read() {
        match press.target {
            Some(InteractKind::Diagnostics) => settings.pings_seen.diagnostic = true,
            Some(InteractKind::Helm) => settings.pings_seen.cockpit_chair = true,
            None => {}
        }
    }
    for msg in tightened.read() {
        if bolts.get(msg.target).is_ok_and(|bolt| {
            bolt.panel == Site::PortEngineInner
                && bolt.pos == Site::PortEngineInner.bolts().unwrap()[0]
        }) {
            settings.pings_seen.first_bolt = true;
        }
    }
    if let Some(turn) = tools.turn
        && bolts.get(turn.target).is_ok_and(|bolt| {
            bolt.panel == Site::PortEngineInner
                && bolt.pos == Site::PortEngineInner.bolts().unwrap()[0]
        })
    {
        settings.pings_seen.first_bolt = true;
    }
    for msg in tape.read() {
        if msg.point.distance(Site::AirlockPortAft.pos()) <= crate::faults::breach::BREACH_RADIUS {
            settings.pings_seen.first_breach = true;
        }
    }
}

fn draw_pings(
    time: Res<Time>,
    coach: Res<Coach>,
    journey: Res<Journey>,
    settings: Res<Settings>,
    faults: Query<&Fault>,
    mut pings: Query<(&CoachPing, &mut Visibility, &mut Transform)>,
) {
    let has = |site: Site| {
        faults
            .iter()
            .any(|fault| fault.site == site && !fault.is_repaired())
    };
    let t = time.elapsed_secs();
    for (ping, mut visibility, mut transform) in &mut pings {
        let show = coach.active
            && match ping {
                CoachPing::Diagnostic => !journey.launched && !settings.pings_seen.diagnostic,
                CoachPing::Bolt => has(Site::PortEngineInner) && !settings.pings_seen.first_bolt,
                CoachPing::Breach => has(Site::AirlockPortAft) && !settings.pings_seen.first_breach,
                CoachPing::Chair => has(Site::Helm) && !settings.pings_seen.cockpit_chair,
            };
        visibility.set_if_neq(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        let pulse = (t * 3.5).sin() * 0.12;
        transform.scale = Vec3::splat(1.0 + pulse);
    }
}

/// Shown during a flight only; the end screen and the menus hide it.
fn draw_panel(
    state: Res<State<AppState>>,
    coach: Res<Coach>,
    journey: Res<Journey>,
    settings: Res<Settings>,
    faults: Query<&Fault>,
    mut panels: Query<&mut Visibility, With<CoachPanel>>,
    mut texts: Query<&mut Text, With<CoachText>>,
) {
    let text = if *state.get() == AppState::Playing {
        lines(&coach, &journey, &settings.tips_seen, faults).join("\n")
    } else {
        String::new()
    };
    for mut visibility in &mut panels {
        visibility.set_if_neq(if text.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for mut t in &mut texts {
        if t.0 != text {
            t.0 = text.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faults::Site;
    use crate::tools::Tool;

    fn fault(site: Site, elapsed: f32) -> Fault {
        Fault {
            elapsed,
            ..Fault::new(site, 80.0)
        }
    }

    fn launched() -> Journey {
        Journey::new(270.0)
    }

    const ON: Coach = Coach {
        active: true,
        launch_note: 0.0,
    };

    #[test]
    fn the_preflight_check_shows_until_launch() {
        let mut held = launched();
        held.launched = false;
        let none = TipsSeen::default();
        assert_eq!(lines(&ON, &held, &none, []), [Tip::Preflight.text()]);
        let noted = Coach {
            launch_note: 1.0,
            ..ON
        };
        assert_eq!(lines(&noted, &launched(), &none, []), [LAUNCH_NOTE]);
        assert!(lines(&ON, &launched(), &none, []).is_empty());
    }

    #[test]
    fn one_tip_per_kind_oldest_first_until_seen() {
        let faults = [
            fault(Site::Helm, 3.0),
            fault(Site::PortEngineInner, 20.0),
            fault(Site::StarboardEngineOuter, 1.0),
            fault(Site::AirlockPortAft, 9.0),
        ];
        let none = TipsSeen::default();
        assert_eq!(
            lines(&ON, &launched(), &none, &faults),
            [
                Tip::LooseBolts.text(),
                Tip::HullBreach.text(),
                Tip::TrajectoryDrift.text()
            ]
        );
        let mut seen = TipsSeen::default();
        seen.mark(Tip::HullBreach);
        assert_eq!(
            lines(&ON, &launched(), &seen, &faults),
            [Tip::LooseBolts.text(), Tip::TrajectoryDrift.text()]
        );
        let mut fixed = fault(Site::Helm, 5.0);
        fixed.repair = 1.0;
        assert!(lines(&ON, &launched(), &none, [&fixed]).is_empty());
    }

    #[test]
    fn an_inactive_coach_says_nothing() {
        let mut held = launched();
        held.launched = false;
        let faults = [fault(Site::Helm, 1.0)];
        let off = Coach {
            active: false,
            launch_note: 5.0,
        };
        assert!(lines(&off, &held, &TipsSeen::default(), &faults).is_empty());
    }

    #[test]
    fn tips_are_marked_one_by_one() {
        let mut seen = TipsSeen::default();
        for (i, tip) in Tip::ALL.into_iter().enumerate() {
            assert!(!seen.has(tip) && !seen.all(), "{tip:?}");
            seen.mark(tip);
            assert!(seen.has(tip));
            assert_eq!(seen.all(), i == Tip::ALL.len() - 1);
        }
        assert_eq!(seen, TipsSeen::EVERY);
        for kind in FaultKind::ALL {
            assert_ne!(Tip::for_fault(kind), Tip::Preflight);
        }
    }

    #[test]
    fn texts_are_ascii_and_name_the_right_keys() {
        for text in Tip::ALL.map(Tip::text).into_iter().chain([LAUNCH_NOTE]) {
            assert!(text.is_ascii(), "{text}");
            assert!(text.len() <= 90, "too long for one line: {text}");
        }
        let key = |tool: Tool| format!("[{}]", tool.slot() + 1);
        assert!(Tip::LooseBolts.text().contains(&key(Tool::Wrench)));
        assert!(Tip::HullBreach.text().contains(&key(Tool::Tape)));
    }
}
