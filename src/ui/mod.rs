//! HUD: the countdown to arrival (top), the current room (top left), the
//! level (under the room), the ship's vitals (top right: oxygen, engine
//! heat, time to impact), the
//! tool belt and a context prompt (bottom), and a controls hint at launch.
//! Shown during a flight and behind the end screen, hidden on the menus.
//! It never shows where a fault is: that is the diagnostic screen's job.
//! Also the notice line ([`Notices`]), shown above everything.
//!
//! Uses the bundled Super Indie font when an `AssetServer` exists; headless
//! tests fall back to the default font.

mod vitals;

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::diagnostics::{DiagView, Diagnostics};
use crate::faults::drift::Nav;
use crate::level::{CurrentLevel, Journey, LEVEL_COUNT, format_clock};
use crate::player::{Focus, InteractKind};
use crate::settings::Settings;
use crate::ship::CurrentRoom;
use crate::tools::{Tool, ToolBelt, ToolState};
use crate::{AppState, GameSet, palette};

pub use vitals::{Gauge, GaugeFill, GaugeLabel, GaugeReading, GaugeValue, gauge};

pub const FONT_PATH: &str = "fonts/super-indie-font/SuperIndie-GOp7O.ttf";
/// The controls hint shows for this long after launch.
pub const HINT_SECS: f32 = 20.0;
pub const CONTROLS_HINT: &str =
    "WASD move   Mouse aim   Click use tool   1 / 2 or wheel switch tool   E interact   Esc pause";
/// Seconds a notice stays up.
pub const NOTICE_SECS: f32 = 4.0;
/// Above the HUD (10), diagnostics (20) and menus (40).
const NOTICE_Z: i32 = 60;

#[derive(Component, Debug)]
pub struct TimerText;

#[derive(Component, Debug)]
pub struct RoomText;

#[derive(Component, Debug)]
pub struct LevelText;

#[derive(Component, Debug)]
pub struct PromptText;

#[derive(Component, Debug)]
pub struct BeltSlot(pub Tool);

#[derive(Component, Debug)]
struct HintText;

/// A top-level HUD node, hidden outside a flight.
#[derive(Component, Debug)]
pub struct Hud;

#[derive(Component, Debug)]
struct NoticeBox;

#[derive(Component, Debug)]
pub struct NoticeText;

/// Short messages for the player ("Could not save settings."), shown one at
/// a time at the bottom of the screen for [`NOTICE_SECS`] each.
#[derive(Resource, Debug, Default)]
pub struct Notices {
    queue: VecDeque<String>,
    /// The notice on screen and its seconds left.
    showing: Option<(String, f32)>,
}

impl Notices {
    pub fn push(&mut self, text: impl Into<String>) {
        self.queue.push_back(text.into());
    }

    /// The notice on screen, if any.
    pub fn current(&self) -> Option<&str> {
        self.showing.as_ref().map(|(text, _)| text.as_str())
    }

    /// Advances the display by `dt` seconds.
    pub fn tick(&mut self, dt: f32) {
        if let Some((_, left)) = &mut self.showing {
            *left -= dt;
            if *left <= 0.0 {
                self.showing = None;
            }
        }
        if self.showing.is_none() {
            self.showing = self.queue.pop_front().map(|text| (text, NOTICE_SECS));
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Notices>()
            .add_systems(
                Startup,
                (spawn_hud, vitals::spawn_vitals, spawn_notice_line).in_set(GameSet::Input),
            )
            .add_systems(
                Update,
                (
                    show_hud,
                    update_hud,
                    vitals::update_vitals,
                    update_prompt,
                    show_notices,
                )
                    .in_set(GameSet::Present),
            );
    }
}

/// The game font, or the default font when there is no asset server.
pub fn game_font(asset_server: Option<&AssetServer>, size: f32) -> TextFont {
    let mut font = TextFont::from_font_size(size);
    if let Some(server) = asset_server {
        font.font = server.load(FONT_PATH).into();
    }
    font
}

/// "LEVEL 2 OF 5", or the level's own name outside the campaign (`number` 0).
pub fn level_label(number: usize, name: &str) -> String {
    if number == 0 {
        name.to_uppercase()
    } else {
        format!("LEVEL {number} OF {LEVEL_COUNT}")
    }
}

pub fn timer_label(remaining: f32) -> String {
    format!("ARRIVAL IN {}", format_clock(remaining))
}

pub fn belt_label(tool: Tool, tape_left: f32) -> String {
    match tool {
        Tool::Wrench => "[1] WRENCH".into(),
        Tool::Tape => format!("[2] TAPE {}s", tape_left.ceil() as u32),
    }
}

/// Inputs to the context prompt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PromptInputs {
    pub focus: Option<InteractKind>,
    pub at_helm: bool,
    pub diag_open: bool,
    pub held: Tool,
    pub snapped: bool,
    pub tape_left: f32,
}

/// What the bottom prompt says, most specific first.
pub fn prompt(i: &PromptInputs) -> &'static str {
    if i.at_helm {
        "WASD  hold the ship inside the square      E  leave the helm"
    } else if i.diag_open {
        "E  close diagnostics"
    } else if i.focus == Some(InteractKind::Helm) {
        "E  take the helm"
    } else if i.focus == Some(InteractKind::Diagnostics) {
        "E  run diagnostics"
    } else if i.held == Tool::Tape && i.tape_left <= 0.0 {
        "The tape roll is empty"
    } else if i.held == Tool::Wrench && i.snapped {
        "Click  turn the bolt"
    } else {
        ""
    }
}

fn spawn_hud(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    let font = |size| game_font(asset_server.as_deref(), size);
    commands.spawn((
        Hud,
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(10),
        children![(
            TimerText,
            Text::new(timer_label(0.0)),
            font(34.0),
            TextColor(palette::UI_TEXT)
        )],
    ));
    commands.spawn((
        Hud,
        RoomText,
        Text::new(""),
        font(20.0),
        TextColor(palette::UI_DIM),
        Node {
            position_type: PositionType::Absolute,
            top: px(18),
            left: px(18),
            ..default()
        },
        GlobalZIndex(10),
    ));
    commands.spawn((
        Hud,
        LevelText,
        Text::new(""),
        font(16.0),
        TextColor(palette::UI_DIM),
        Node {
            position_type: PositionType::Absolute,
            // Under the room name: the vitals panel has the top right.
            top: px(44),
            left: px(18),
            ..default()
        },
        GlobalZIndex(10),
    ));
    commands.spawn((
        Hud,
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            width: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(6),
            ..default()
        },
        GlobalZIndex(10),
        children![
            (
                PromptText,
                Text::new(""),
                font(20.0),
                TextColor(palette::UI_ACCENT),
                // Readable over the room's bottom wall.
                Node {
                    padding: UiRect::axes(px(10), px(2)),
                    ..default()
                },
                BackgroundColor(palette::UI_PANEL.with_alpha(0.8)),
                Visibility::Hidden,
            ),
            (
                Node {
                    column_gap: px(28),
                    ..default()
                },
                children![
                    (
                        BeltSlot(Tool::Wrench),
                        Text::new(""),
                        font(20.0),
                        TextColor(palette::UI_DIM)
                    ),
                    (
                        BeltSlot(Tool::Tape),
                        Text::new(""),
                        font(20.0),
                        TextColor(palette::UI_DIM)
                    ),
                ],
            ),
            (
                HintText,
                Text::new(CONTROLS_HINT),
                font(15.0),
                TextColor(palette::UI_DIM)
            ),
        ],
    ));
}

fn spawn_notice_line(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(64),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(NOTICE_Z),
        children![(
            NoticeBox,
            Node {
                padding: UiRect::axes(px(16), px(8)),
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(palette::UI_PANEL),
            BorderColor::all(palette::UI_ACCENT),
            Visibility::Hidden,
            children![(
                NoticeText,
                Text::new(""),
                game_font(asset_server.as_deref(), 18.0),
                TextColor(palette::UI_TEXT),
            )],
        )],
    ));
}

/// The HUD belongs to a flight: hidden on the title and menu screens.
fn show_hud(state: Res<State<AppState>>, mut huds: Query<&mut Visibility, With<Hud>>) {
    let show = matches!(
        state.get(),
        AppState::Playing | AppState::Landed | AppState::Lost
    );
    let visibility = if show {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut v in &mut huds {
        v.set_if_neq(visibility);
    }
}

/// Real time, so notices time out while paused (or frozen in e2e) too.
fn show_notices(
    time: Res<Time<Real>>,
    mut notices: ResMut<Notices>,
    mut boxes: Query<&mut Visibility, With<NoticeBox>>,
    mut texts: Query<&mut Text, With<NoticeText>>,
) {
    notices.tick(time.delta_secs());
    let current = notices.current().unwrap_or_default();
    for mut v in &mut boxes {
        v.set_if_neq(if current.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    for mut t in &mut texts {
        if t.0 != current {
            t.0 = current.to_string();
        }
    }
}

fn update_hud(
    journey: Res<Journey>,
    level: Res<CurrentLevel>,
    room: Res<CurrentRoom>,
    belt: Res<ToolBelt>,
    settings: Res<Settings>,
    state: Res<State<AppState>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TimerText>>,
        Query<&mut Text, With<RoomText>>,
        Query<(&mut Text, &mut TextColor, &BeltSlot)>,
        Query<&mut Text, With<LevelText>>,
    )>,
    mut hints: Query<&mut Visibility, With<HintText>>,
) {
    let timer = match state.get() {
        AppState::Landed => "LANDED".to_string(),
        _ => timer_label(journey.remaining()),
    };
    for mut t in &mut texts.p0() {
        if t.0 != timer {
            t.0 = timer.clone();
        }
    }
    let room_name = room.0.name().to_uppercase();
    for mut t in &mut texts.p1() {
        if t.0 != room_name {
            t.0 = room_name.clone();
        }
    }
    let level_name = level_label(level.number(), &level.0.name);
    for mut t in &mut texts.p3() {
        if t.0 != level_name {
            t.0 = level_name.clone();
        }
    }
    for (mut t, mut color, slot) in &mut texts.p2() {
        let label = belt_label(slot.0, belt.tape_left);
        if t.0 != label {
            t.0 = label;
        }
        color.0 = if slot.0 == belt.held {
            palette::UI_ACCENT
        } else {
            palette::UI_DIM
        };
    }
    let show_hint =
        settings.controls_hint && *state.get() == AppState::Playing && journey.elapsed < HINT_SECS;
    for mut v in &mut hints {
        *v = if show_hint {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_prompt(
    focus: Res<Focus>,
    nav: Res<Nav>,
    diag: Res<Diagnostics>,
    belt: Res<ToolBelt>,
    tools: Res<ToolState>,
    state: Res<State<AppState>>,
    mut prompts: Query<(&mut Text, &mut Visibility), With<PromptText>>,
) {
    let text = if *state.get() == AppState::Playing {
        prompt(&PromptInputs {
            focus: focus.0,
            at_helm: nav.engaged,
            diag_open: diag.view != DiagView::Closed,
            held: belt.held,
            snapped: tools.snap.is_some(),
            tape_left: belt.tape_left,
        })
    } else {
        ""
    };
    for (mut t, mut visibility) in &mut prompts {
        if t.0 != text {
            t.0 = text.to_string();
        }
        let want = if text.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != want {
            *visibility = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> PromptInputs {
        PromptInputs {
            focus: None,
            at_helm: false,
            diag_open: false,
            held: Tool::Wrench,
            snapped: false,
            tape_left: 10.0,
        }
    }

    #[test]
    fn helm_prompt_wins_over_everything() {
        let i = PromptInputs {
            at_helm: true,
            focus: Some(InteractKind::Helm),
            snapped: true,
            ..inputs()
        };
        assert!(prompt(&i).contains("leave the helm"));
    }

    #[test]
    fn consoles_and_tools_have_prompts() {
        assert_eq!(prompt(&inputs()), "");
        let near_console = PromptInputs {
            focus: Some(InteractKind::Diagnostics),
            ..inputs()
        };
        assert_eq!(prompt(&near_console), "E  run diagnostics");
        let snapped = PromptInputs {
            snapped: true,
            ..inputs()
        };
        assert_eq!(prompt(&snapped), "Click  turn the bolt");
        let empty = PromptInputs {
            held: Tool::Tape,
            tape_left: 0.0,
            ..inputs()
        };
        assert_eq!(prompt(&empty), "The tape roll is empty");
    }

    #[test]
    fn notices_show_one_at_a_time_then_clear() {
        let mut n = Notices::default();
        n.push("first");
        n.push("second");
        n.tick(0.0);
        assert_eq!(n.current(), Some("first"));
        n.tick(NOTICE_SECS - 0.1);
        assert_eq!(n.current(), Some("first"));
        n.tick(0.2);
        assert_eq!(n.current(), Some("second"));
        n.tick(NOTICE_SECS + 0.1);
        assert_eq!(n.current(), None);
    }

    #[test]
    fn labels() {
        assert_eq!(timer_label(125.5), "ARRIVAL IN 2:06");
        assert_eq!(level_label(2, "Level 2"), "LEVEL 2 OF 5");
        assert_eq!(level_label(0, "Test flight"), "TEST FLIGHT");
        assert_eq!(belt_label(Tool::Tape, 19.2), "[2] TAPE 20s");
        assert_eq!(belt_label(Tool::Wrench, 0.0), "[1] WRENCH");
    }
}
