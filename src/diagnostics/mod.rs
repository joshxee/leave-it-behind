//! The diagnostic screen in the engineer's quarters. E at the console runs a
//! short scan, then shows a minimap of the whole ship that pinpoints every
//! active fault (each loose bolt, each breach, the helm) with its time left.
//! It is the only place fault locations are shown. E again, or walking
//! away from the console, closes it. Time keeps running while it is open.

use bevy::prelude::*;

use crate::faults::bolts::Bolt;
use crate::faults::{Fault, FaultKind};
use crate::player::{Focus, InteractKind, InteractPressed, Interactable, Player};
use crate::shapes::{at, rect};
use crate::ship::RoomId;
use crate::ship::layout::{self, HALF_HEIGHT, WALL};
use crate::{AppState, GameSet, RunSet, palette};

/// Seconds between opening the screen and the faults appearing.
pub const SCAN_SECS: f32 = 1.0;
/// E works within this distance of the console's front edge.
pub const CONSOLE_RANGE: f32 = 90.0;

/// Minimap size in UI pixels (the whole ship, walls included).
const MAP_WIDTH: f32 = 1100.0;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum DiagView {
    #[default]
    Closed,
    Scanning {
        left: f32,
    },
    Open,
}

impl DiagView {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagView::Closed => "Closed",
            DiagView::Scanning { .. } => "Scanning",
            DiagView::Open => "Open",
        }
    }
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Diagnostics {
    pub view: DiagView,
    /// Times the screen was opened this run.
    pub uses: u32,
}

/// One pinpointed fault location on the minimap.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub kind: FaultKind,
    pub room: RoomId,
    pub label: &'static str,
    pub remaining: f32,
    /// Exact positions (every loose bolt of a panel, the breach, the helm).
    pub points: Vec<Vec2>,
}

/// What the screen shows for the current faults, most urgent first.
pub fn readings<'a>(
    faults: impl IntoIterator<Item = &'a Fault>,
    loose_bolts: &[Bolt],
) -> Vec<Reading> {
    let mut out: Vec<Reading> = faults
        .into_iter()
        .map(|f| {
            let points = match f.kind() {
                FaultKind::LooseBolts => loose_bolts
                    .iter()
                    .filter(|b| b.loose && b.panel == f.site)
                    .map(|b| b.pos)
                    .collect(),
                _ => vec![f.site.pos()],
            };
            Reading {
                kind: f.kind(),
                room: f.site.room(),
                label: f.site.label(),
                remaining: f.remaining(),
                points,
            }
        })
        .collect();
    out.sort_by(|a, b| a.remaining.total_cmp(&b.remaining));
    out
}

/// One line of the fault list.
pub fn reading_line(r: &Reading) -> String {
    format!(
        "{} - {}, {} - {}s left",
        r.kind.name().to_uppercase(),
        r.room.name(),
        r.label,
        r.remaining.ceil() as u32
    )
}

/// World position to minimap pixels (origin top left).
pub fn to_map(p: Vec2) -> Vec2 {
    let half = Vec2::new(layout::ship_length() / 2.0 + WALL, HALF_HEIGHT + WALL);
    let scale = MAP_WIDTH / (half.x * 2.0);
    Vec2::new((p.x + half.x) * scale, (half.y - p.y) * scale)
}

fn map_height() -> f32 {
    to_map(Vec2::new(0.0, -HALF_HEIGHT - WALL)).y
}

#[derive(Component)]
struct DiagOverlay;
#[derive(Component)]
struct DiagStatus;
#[derive(Component)]
struct DiagMap;
#[derive(Component)]
struct DiagMarker;
#[derive(Component)]
struct DiagPlayerDot;
#[derive(Component)]
struct DiagList;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Diagnostics>()
            .add_systems(
                Startup,
                (spawn_console, spawn_overlay).in_set(GameSet::Input),
            )
            .add_systems(OnEnter(AppState::Playing), reset.in_set(RunSet::Spawn))
            .add_systems(
                FixedUpdate,
                use_console
                    .in_set(GameSet::Act)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                scan.in_set(GameSet::Simulate)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, draw_overlay.in_set(GameSet::Present));
    }
}

fn spawn_console(mut commands: Commands) {
    let console = layout::diag_console();
    let screen = Rect::from_center_size(
        console.center() + Vec2::new(0.0, 6.0),
        Vec2::new(110.0, 22.0),
    );
    commands.spawn((
        rect(screen.size(), palette::DIAG_SCREEN),
        at(screen.center(), 1.3),
    ));
    commands.spawn((
        Interactable {
            kind: InteractKind::Diagnostics,
            range: CONSOLE_RANGE,
        },
        Transform::from_translation(Vec2::new(console.center().x, console.min.y).extend(0.0)),
    ));
}

fn reset(mut diag: ResMut<Diagnostics>) {
    *diag = Diagnostics::default();
}

fn use_console(
    mut presses: MessageReader<InteractPressed>,
    focus: Res<Focus>,
    mut diag: ResMut<Diagnostics>,
) {
    for press in presses.read() {
        if diag.view != DiagView::Closed {
            diag.view = DiagView::Closed;
        } else if press.target == Some(InteractKind::Diagnostics) {
            diag.view = DiagView::Scanning { left: SCAN_SECS };
            diag.uses += 1;
        }
    }
    // Walking away from the console closes it.
    if diag.view != DiagView::Closed && focus.0 != Some(InteractKind::Diagnostics) {
        diag.view = DiagView::Closed;
    }
}

fn scan(time: Res<Time>, mut diag: ResMut<Diagnostics>) {
    if let DiagView::Scanning { left } = diag.view {
        let left = left - time.delta_secs();
        diag.view = if left <= 1e-4 {
            DiagView::Open
        } else {
            DiagView::Scanning { left }
        };
    }
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}

fn spawn_overlay(mut commands: Commands) {
    let root = commands
        .spawn((
            DiagOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(20),
            Visibility::Hidden,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(16)),
                row_gap: px(10),
                border: UiRect::all(px(2)),
                ..default()
            },
            BackgroundColor(palette::UI_PANEL),
            BorderColor::all(palette::DIAG_SCREEN),
            ChildOf(root),
        ))
        .id();
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(24),
            ..default()
        },
        ChildOf(panel),
        children![
            text("SHIP DIAGNOSTICS", 22.0, palette::GOOD),
            (DiagStatus, text("", 22.0, palette::UI_TEXT)),
        ],
    ));
    let map = commands
        .spawn((
            DiagMap,
            Node {
                width: px(MAP_WIDTH),
                height: px(map_height()),
                ..default()
            },
            BackgroundColor(palette::WALL),
            ChildOf(panel),
        ))
        .id();
    for room in RoomId::ALL {
        let r = room.interior();
        let top_left = to_map(Vec2::new(r.min.x, r.max.y));
        let size = to_map(Vec2::new(r.max.x, r.min.y)) - top_left;
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(top_left.x),
                top: px(top_left.y),
                width: px(size.x),
                height: px(size.y),
                padding: UiRect::all(px(4)),
                ..default()
            },
            BackgroundColor(palette::FLOORS[room.index()]),
            ChildOf(map),
            children![text(room.name().to_uppercase(), 12.0, palette::UI_DIM)],
        ));
    }
    commands.spawn((
        DiagPlayerDot,
        Node {
            position_type: PositionType::Absolute,
            width: px(10),
            height: px(10),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(palette::PLAYER),
        GlobalZIndex(22),
        ChildOf(map),
    ));
    commands.spawn((DiagList, text("", 16.0, palette::UI_TEXT), ChildOf(panel)));
    commands.spawn((text("E  close", 14.0, palette::UI_DIM), ChildOf(panel)));
}

fn draw_overlay(
    mut commands: Commands,
    diag: Res<Diagnostics>,
    time: Res<Time>,
    faults: Query<&Fault>,
    bolts: Query<&Bolt>,
    players: Query<&Transform, With<Player>>,
    mut overlay: Query<&mut Visibility, With<DiagOverlay>>,
    mut status: Query<&mut Text, (With<DiagStatus>, Without<DiagList>)>,
    mut list: Query<&mut Text, (With<DiagList>, Without<DiagStatus>)>,
    mut dot: Query<&mut Node, With<DiagPlayerDot>>,
    map: Query<Entity, With<DiagMap>>,
    markers: Query<Entity, With<DiagMarker>>,
    state: Res<State<AppState>>,
) {
    let open = diag.view != DiagView::Closed && *state.get() == AppState::Playing;
    for mut v in &mut overlay {
        *v = if open {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for marker in &markers {
        commands.entity(marker).despawn();
    }
    if !open {
        return;
    }
    if let (Some(player), Ok(mut node)) = (players.iter().next(), dot.single_mut()) {
        let p = to_map(player.translation.truncate());
        node.left = px(p.x - 5.0);
        node.top = px(p.y - 5.0);
    }
    let (status_text, lines) = match diag.view {
        DiagView::Scanning { left } => {
            let dots = ".".repeat(1 + ((SCAN_SECS - left) * 6.0) as usize % 4);
            (format!("SCANNING{dots}"), String::new())
        }
        _ => {
            let bolts: Vec<Bolt> = bolts.iter().copied().collect();
            let readings = readings(faults, &bolts);
            let Ok(map) = map.single() else {
                return;
            };
            let pulse = 0.55 + 0.45 * (time.elapsed_secs() * 6.0).sin().abs();
            for point in readings.iter().flat_map(|r| r.points.iter()) {
                let p = to_map(*point);
                commands.spawn((
                    DiagMarker,
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(p.x - 7.0),
                        top: px(p.y - 7.0),
                        width: px(14),
                        height: px(14),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(palette::ALERT.with_alpha(pulse)),
                    GlobalZIndex(21),
                    ChildOf(map),
                ));
            }
            let status = match readings.len() {
                0 => "ALL SYSTEMS NOMINAL".to_string(),
                1 => "1 ACTIVE FAULT".to_string(),
                n => format!("{n} ACTIVE FAULTS"),
            };
            (
                status,
                readings
                    .iter()
                    .map(reading_line)
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
    };
    for mut t in &mut status {
        t.0 = status_text.clone();
    }
    for mut t in &mut list {
        t.0 = lines.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faults::Site;

    #[test]
    fn map_covers_the_whole_ship() {
        let half = layout::ship_length() / 2.0 + WALL;
        assert_eq!(to_map(Vec2::new(-half, HALF_HEIGHT + WALL)), Vec2::ZERO);
        let bottom_right = to_map(Vec2::new(half, -HALF_HEIGHT - WALL));
        assert!((bottom_right.x - MAP_WIDTH).abs() < 1e-3);
        assert!((bottom_right.y - map_height()).abs() < 1e-3);
    }

    #[test]
    fn readings_pinpoint_each_loose_bolt_and_sort_by_urgency() {
        let mut bolts_fault = Fault::new(Site::PortEngineInner, 50.0);
        bolts_fault.elapsed = 10.0;
        let mut breach = Fault::new(Site::AirlockPortAft, 45.0);
        breach.elapsed = 30.0;
        let panel = Site::PortEngineInner.bolts().unwrap();
        let bolts = [
            Bolt {
                panel: Site::PortEngineInner,
                pos: panel[0],
                loose: true,
            },
            Bolt {
                panel: Site::PortEngineInner,
                pos: panel[1],
                loose: false,
            },
            Bolt {
                panel: Site::PortEngineOuter,
                pos: Vec2::ZERO,
                loose: true,
            },
        ];
        let r = readings([&bolts_fault, &breach], &bolts);
        assert_eq!(r[0].kind, FaultKind::HullBreach, "15s left beats 40s left");
        assert_eq!(r[0].points, vec![Site::AirlockPortAft.pos()]);
        assert_eq!(r[1].points, vec![panel[0]]);
        assert_eq!(r[1].room, RoomId::Engine);
        assert_eq!(
            reading_line(&r[0]),
            "HULL BREACH - Airlock, upper wall, rear - 15s left"
        );
    }
}
