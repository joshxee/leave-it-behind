//! Draws the top screen of the menu stack: a full-screen root holding the
//! title, text lines, focusable rows and footer. Rebuilt whenever the screen
//! or what it shows changes; the focus highlight is updated every frame.

use bevy::prelude::*;

use super::screens::{Content, Item, Style};
use crate::palette;
use crate::settings::{SettingKey, Settings};

/// Above the HUD (10) and the diagnostic screen (20); below notices (60).
pub const MENU_Z: i32 = 40;
const ROW_WIDTH: f32 = 470.0;
const ROW_HEIGHT: f32 = 42.0;

#[derive(Component, Debug)]
pub struct MenuRoot;

/// A focusable row: its index in the screen's items.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuRow(pub usize);

/// The `<` and `>` on a setting row.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepArrow {
    pub row: usize,
    pub key: SettingKey,
    pub up: bool,
}

/// Text that takes the highlight of row `.0`.
#[derive(Component, Debug, Clone, Copy)]
pub struct RowText(pub usize);

/// Set when the mouse button went down on this node: a release over it is
/// a click. (Bevy's `Interaction` only reports the press.)
#[derive(Component, Debug, Default)]
pub struct Clicky(pub bool);

pub fn spawn_title(
    commands: &mut Commands,
    assets: Option<&AssetServer>,
    font: &impl Fn(f32) -> TextFont,
) {
    let root = commands
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(palette::VOID),
            GlobalZIndex(MENU_Z),
        ))
        .id();
    if let Some(assets) = assets {
        commands.spawn((
            ImageNode::new(assets.load("cover/engineer-emergency-cover-square.png")),
            Node {
                width: Val::VMin(100.0),
                height: Val::VMin(100.0),
                flex_shrink: 0.0,
                ..default()
            },
            ChildOf(root),
        ));
    }
    let caption = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(0),
                width: percent(100),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(palette::VOID.with_alpha(0.82)),
            ChildOf(root),
        ))
        .id();
    commands.spawn((
        Text::new(super::screens::GAME_TITLE),
        font(64.0),
        TextColor(palette::UI_TITLE),
        TextLayout::justify(Justify::Center),
        Node {
            max_width: percent(100),
            ..default()
        },
        ChildOf(caption),
    ));
    commands.spawn((
        Text::new("CLICK OR PRESS ANY KEY"),
        font(22.0),
        TextColor(palette::UI_ACCENT),
        TextLayout::justify(Justify::Center),
        Node {
            max_width: percent(100),
            ..default()
        },
        ChildOf(caption),
    ));
}

pub fn spawn_screen(
    commands: &mut Commands,
    content: &Content,
    settings: &Settings,
    focus: usize,
    backdrop: Color,
    font: &impl Fn(f32) -> TextFont,
) {
    let root = commands
        .spawn((
            MenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: px(10),
                ..default()
            },
            BackgroundColor(backdrop),
            GlobalZIndex(MENU_Z),
        ))
        .id();
    let title_size = if content.big_title { 84.0 } else { 56.0 };
    if !content.title.is_empty() {
        commands.spawn((
            Text::new(content.title.clone()),
            font(title_size),
            TextColor(palette::UI_TITLE),
            Node {
                margin: UiRect::bottom(px(8)),
                ..default()
            },
            ChildOf(root),
        ));
    }
    for (text, style) in &content.lines {
        let (size, color, top) = match style {
            Style::Body => (20.0, palette::UI_TEXT, 0.0),
            Style::Dim => (18.0, palette::UI_DIM, 0.0),
            Style::Accent => (24.0, palette::UI_ACCENT, 6.0),
            Style::Heading => (20.0, palette::UI_ACCENT, 10.0),
        };
        commands.spawn((
            Text::new(text.clone()),
            font(size),
            TextColor(color),
            TextLayout::justify(Justify::Center),
            Node {
                margin: UiRect::top(px(top)),
                max_width: percent(90),
                ..default()
            },
            ChildOf(root),
        ));
    }
    if !content.items.is_empty() {
        let list = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    margin: UiRect::vertical(px(12)),
                    ..default()
                },
                ChildOf(root),
            ))
            .id();
        for (i, item) in content.items.iter().enumerate() {
            spawn_row(commands, list, i, item, settings, i == focus, font);
        }
    }
    for text in &content.footer {
        commands.spawn((
            Text::new(text.clone()),
            font(16.0),
            TextColor(palette::UI_DIM),
            ChildOf(root),
        ));
    }
}

fn spawn_row(
    commands: &mut Commands,
    list: Entity,
    index: usize,
    item: &Item,
    settings: &Settings,
    focused: bool,
    font: &impl Fn(f32) -> TextFont,
) {
    let (text_color, bg, border) = row_colors(focused);
    let row = commands
        .spawn((
            MenuRow(index),
            Button,
            Clicky::default(),
            Node {
                width: px(ROW_WIDTH),
                height: px(ROW_HEIGHT),
                padding: UiRect::horizontal(px(18)),
                border: UiRect::all(px(2)),
                justify_content: match item {
                    Item::Button { .. } => JustifyContent::Center,
                    Item::Setting(_) => JustifyContent::SpaceBetween,
                },
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::all(border),
            ChildOf(list),
        ))
        .id();
    let label = match item {
        Item::Button { label, .. } => label.clone(),
        Item::Setting(key) => key.label().to_string(),
    };
    commands.spawn((
        RowText(index),
        Text::new(label),
        font(22.0),
        TextColor(text_color),
        ChildOf(row),
    ));
    let Item::Setting(key) = item else {
        return;
    };
    let values = commands
        .spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                ..default()
            },
            ChildOf(row),
        ))
        .id();
    let arrow = |commands: &mut Commands, up: bool| {
        commands.spawn((
            StepArrow {
                row: index,
                key: *key,
                up,
            },
            Button,
            Clicky::default(),
            Node {
                padding: UiRect::horizontal(px(6)),
                ..default()
            },
            ChildOf(values),
            children![(
                RowText(index),
                Text::new(if up { ">" } else { "<" }),
                font(22.0),
                TextColor(text_color),
            )],
        ));
    };
    arrow(commands, false);
    commands.spawn((
        RowText(index),
        Text::new(settings.value_label(*key)),
        font(22.0),
        TextColor(text_color),
        TextLayout::justify(Justify::Center),
        Node {
            width: px(64),
            ..default()
        },
        ChildOf(values),
    ));
    arrow(commands, true);
}

/// Text, background and border of a row.
fn row_colors(focused: bool) -> (Color, Color, Color) {
    if focused {
        // The paler cyan for text: it reads on the blue fill.
        (palette::UI_TITLE, palette::UI_ROW_FOCUS, palette::UI_ACCENT)
    } else {
        (palette::UI_TEXT, palette::UI_ROW, palette::UI_DIM)
    }
}

/// Moves the highlight to the focused row.
pub fn highlight(
    focus: usize,
    rows: &mut Query<(&MenuRow, &mut BackgroundColor, &mut BorderColor)>,
    texts: &mut Query<(&RowText, &mut TextColor)>,
) {
    for (row, mut bg, mut border) in rows.iter_mut() {
        let (_, fill, edge) = row_colors(row.0 == focus);
        bg.set_if_neq(BackgroundColor(fill));
        border.set_if_neq(BorderColor::all(edge));
    }
    for (text, mut color) in texts.iter_mut() {
        let (fg, _, _) = row_colors(text.0 == focus);
        color.set_if_neq(TextColor(fg));
    }
}
