//! HUD: the countdown to arrival (top), the current room (top left), the
//! tool belt and a context prompt (bottom), a controls hint at launch, and
//! the landed / lost screen. It never shows where a fault is: that is the
//! diagnostic screen's job. Uses the bundled Super Indie font when an
//! `AssetServer` exists; headless tests fall back to the default font.

use bevy::prelude::*;

use crate::diagnostics::{DiagView, Diagnostics};
use crate::faults::drift::Nav;
use crate::level::{CurrentLevel, Journey, RunStats, format_clock};
use crate::player::{Focus, InteractKind};
use crate::ship::CurrentRoom;
use crate::tools::{Tool, ToolBelt, ToolState};
use crate::{AppState, GameSet, palette};

pub const FONT_PATH: &str = "fonts/super-indie-font/SuperIndie-GOp7O.ttf";
/// The controls hint shows for this long after launch.
pub const HINT_SECS: f32 = 20.0;
pub const CONTROLS_HINT: &str =
    "WASD move   Mouse aim   Click use tool   1 / 2 or wheel switch tool   E interact";

#[derive(Component, Debug)]
pub struct TimerText;

#[derive(Component, Debug)]
pub struct RoomText;

#[derive(Component, Debug)]
pub struct PromptText;

#[derive(Component, Debug)]
pub struct BeltSlot(pub Tool);

#[derive(Component, Debug)]
struct HintText;

#[derive(Component, Debug)]
pub struct EndScreen;

#[derive(Component, Debug)]
struct EndTitle;

#[derive(Component, Debug)]
struct EndBody;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_hud, spawn_end_screen).in_set(GameSet::Input),
        )
        .add_systems(
            Update,
            (update_hud, update_prompt, update_end_screen).in_set(GameSet::Present),
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
        "WASD  steer the marker into the centre band      E  leave the helm"
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
                TextColor(palette::UI_ACCENT)
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

fn spawn_end_screen(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    let font = |size| game_font(asset_server.as_deref(), size);
    commands.spawn((
        EndScreen,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(18),
            ..default()
        },
        BackgroundColor(palette::UI_PANEL.with_alpha(0.75)),
        GlobalZIndex(30),
        Visibility::Hidden,
        children![
            (
                EndTitle,
                Text::new(""),
                font(56.0),
                TextColor(palette::UI_TEXT)
            ),
            (
                EndBody,
                Text::new(""),
                font(22.0),
                TextColor(palette::UI_TEXT),
                TextLayout::justify(Justify::Center),
            ),
            (
                Text::new("Press R to fly again"),
                font(22.0),
                TextColor(palette::UI_ACCENT)
            ),
        ],
    ));
}

fn update_hud(
    journey: Res<Journey>,
    room: Res<CurrentRoom>,
    belt: Res<ToolBelt>,
    state: Res<State<AppState>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TimerText>>,
        Query<&mut Text, With<RoomText>>,
        Query<(&mut Text, &mut TextColor, &BeltSlot)>,
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
    let show_hint = *state.get() == AppState::Playing && journey.elapsed < HINT_SECS;
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
    mut prompts: Query<&mut Text, With<PromptText>>,
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
    for mut t in &mut prompts {
        if t.0 != text {
            t.0 = text.to_string();
        }
    }
}

fn update_end_screen(
    state: Res<State<AppState>>,
    stats: Res<RunStats>,
    journey: Res<Journey>,
    level: Res<CurrentLevel>,
    diag: Res<Diagnostics>,
    belt: Res<ToolBelt>,
    mut screens: Query<&mut Visibility, With<EndScreen>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<EndTitle>>,
        Query<&mut Text, With<EndBody>>,
    )>,
) {
    let (title, body) = match state.get() {
        AppState::Landed => (
            "TOUCHDOWN".to_string(),
            format!(
                "You kept the ship together all the way down.\n\
                 Faults fixed: {} of {}    Diagnostics used: {}    Tape left: {}s",
                stats.fixed,
                stats.started,
                diag.uses,
                belt.tape_left.ceil() as u32
            ),
        ),
        AppState::Lost => {
            let kind = stats.failure.map(|s| s.kind());
            (
                kind.map_or("LOST", |k| k.failure_title()).to_uppercase(),
                format!(
                    "{}\nSurvived {} of {}.    Faults fixed: {}",
                    kind.map_or("", |k| k.failure_text()),
                    format_clock(journey.elapsed.min(journey.duration)),
                    format_clock(level.0.duration_secs),
                    stats.fixed
                ),
            )
        }
        _ => (String::new(), String::new()),
    };
    let show = !title.is_empty();
    for mut v in &mut screens {
        *v = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for mut t in &mut texts.p0() {
        if t.0 != title {
            t.0 = title.clone();
        }
    }
    for mut t in &mut texts.p1() {
        if t.0 != body {
            t.0 = body.clone();
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
    fn labels() {
        assert_eq!(timer_label(125.5), "ARRIVAL IN 2:06");
        assert_eq!(belt_label(Tool::Tape, 19.2), "[2] TAPE 20s");
        assert_eq!(belt_label(Tool::Wrench, 0.0), "[1] WRENCH");
    }
}
