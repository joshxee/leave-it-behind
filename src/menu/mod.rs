//! Menus and pausing: the title card, the main menu, how to play, settings,
//! the pause menu, the end-of-flight screen and confirm dialogs.
//!
//! [`Menu`] is a stack of screens; the top one is drawn (`view`) from
//! [`content`]. Keyboard (arrows or WASD, Enter or Space, Esc) and mouse
//! (hover, click) share one highlight. All input becomes [`MenuAction`]
//! messages, applied in one place ([`apply_actions`]).
//!
//! Pausing: Esc or P during a flight, or the window losing focus (a
//! setting), enters [`Pause::Paused`]; gameplay stops (`running`) and the
//! pause menu opens. Resuming clears any input held from before.

mod screens;
mod view;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::WindowFocused;

pub use screens::{
    Content, Ctx, EndInfo, Item, Style, content, damage_line, escape_action, progress_line,
};
pub use view::{MENU_Z, MenuRoot, MenuRow, StepArrow};

use crate::diagnostics::Diagnostics;
use crate::level::{CurrentLevel, Journey, LastRun, Progress, RunStats};
use crate::player::PlayerIntent;
use crate::settings::{SettingKey, Settings};
use crate::tools::ToolBelt;
use crate::ui::{Notices, game_font};
use crate::{AppState, GameSet, Pause, RunSet, palette, running};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    /// "Click or press any key", once per launch before the first flight.
    Title,
    Main,
    HowToPlay,
    Settings,
    Pause,
    /// Landed or lost: the flight's score and what next.
    End,
    Confirm(Confirm),
}

impl Screen {
    pub fn as_str(self) -> &'static str {
        match self {
            Screen::Title => "Title",
            Screen::Main => "Main",
            Screen::HowToPlay => "HowToPlay",
            Screen::Settings => "Settings",
            Screen::Pause => "Pause",
            Screen::End => "End",
            Screen::Confirm(Confirm::Restart) => "ConfirmRestart",
            Screen::Confirm(Confirm::MainMenu) => "ConfirmMainMenu",
            Screen::Confirm(Confirm::Quit) => "ConfirmQuit",
            Screen::Confirm(Confirm::ResetProgress) => "ConfirmResetProgress",
        }
    }
}

/// Actions that throw something away ask first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Confirm {
    Restart,
    MainMenu,
    Quit,
    ResetProgress,
}

/// Everything a menu can do. Written by the input systems (and tests),
/// applied by [`apply_actions`].
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// From the title card to the main menu.
    Continue,
    /// A fresh flight: Play, Fly again, or a confirmed Restart.
    Play,
    Open(Screen),
    Back,
    Resume,
    ToMainMenu,
    /// Desktop only.
    Quit,
    ResetSettings,
    ResetProgress,
    /// A setting one step down (`false`) or up.
    Step(SettingKey, bool),
    /// A setting's next value, wrapping.
    Cycle(SettingKey),
}

/// One screen on the stack and its highlighted row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub screen: Screen,
    pub focus: usize,
}

/// The screens on show, bottom to top. Empty during a flight.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct Menu {
    pub stack: Vec<Entry>,
    /// The title card shows only before the first flight of a launch.
    pub title_done: bool,
}

impl Menu {
    pub fn top(&self) -> Option<Entry> {
        self.stack.last().copied()
    }

    pub fn screen(&self) -> Option<Screen> {
        self.top().map(|e| e.screen)
    }

    pub fn open(&mut self, screen: Screen) {
        self.stack.push(Entry { screen, focus: 0 });
    }

    pub fn show_only(&mut self, screen: Screen) {
        self.stack = vec![Entry { screen, focus: 0 }];
    }

    /// Back to the screen underneath (the bottom screen stays).
    pub fn back(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn close(&mut self) {
        self.stack.clear();
    }

    fn set_focus(&mut self, focus: usize) {
        if let Some(top) = self.stack.last_mut() {
            top.focus = focus;
        }
    }
}

/// What the screens need to know, as one system parameter.
#[derive(SystemParam)]
pub struct MenuCtx<'w> {
    state: Res<'w, State<AppState>>,
    settings: Res<'w, Settings>,
    progress: Res<'w, Progress>,
    last_run: Res<'w, LastRun>,
    level: Res<'w, CurrentLevel>,
    journey: Res<'w, Journey>,
    stats: Res<'w, RunStats>,
    diag: Res<'w, Diagnostics>,
    belt: Res<'w, ToolBelt>,
}

impl MenuCtx<'_> {
    pub fn get(&self) -> Ctx {
        let level = self.progress.level(&self.level.0.id).cloned();
        let ended = matches!(self.state.get(), AppState::Landed | AppState::Lost);
        let end = self
            .last_run
            .record
            .filter(|_| ended)
            .map(|record| EndInfo {
                record,
                new_best: self.last_run.new_best,
                best: level.as_ref().and_then(|l| l.best),
                failure: self.stats.failure.map(|site| site.kind()),
                fixed: self.stats.fixed,
                started: self.stats.started,
                diag_uses: self.diag.uses,
                tape_left: self.belt.tape_left,
                duration: self.journey.duration,
            });
        Ctx {
            web: cfg!(target_arch = "wasm32"),
            settings: self.settings.clone(),
            level_name: self.level.0.name.clone(),
            level,
            time_left: self.journey.remaining(),
            end,
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Whether what the screens show may have changed.
    fn changed(&self) -> bool {
        self.settings.is_changed() || self.progress.is_changed() || self.last_run.is_changed()
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Menu>()
            .add_message::<MenuAction>()
            // Registered by WindowPlugin; headless test apps have none.
            .add_message::<WindowFocused>()
            .add_systems(OnEnter(AppState::Menu), open_menu)
            .add_systems(
                OnEnter(AppState::Playing),
                start_flight.in_set(RunSet::Cleanup),
            )
            .add_systems(OnEnter(Pause::Paused), open_pause)
            .add_systems(OnExit(Pause::Paused), resume_flight)
            .add_systems(OnEnter(AppState::Landed), open_end)
            .add_systems(OnEnter(AppState::Lost), open_end)
            .add_systems(
                Update,
                (
                    (pause_keys, pause_on_focus_loss).run_if(running),
                    (menu_keys, menu_mouse).chain().run_if(menu_open),
                )
                    .in_set(GameSet::Input),
            )
            .add_systems(Update, apply_actions.in_set(GameSet::Act))
            .add_systems(
                Update,
                (draw_menu, highlight_focus)
                    .chain()
                    .in_set(GameSet::Present),
            );
        #[cfg(feature = "smoke")]
        app.add_message::<bevy::dev_tools::ci_testing::CiTestingCustomEvent>()
            .add_systems(Update, smoke_play.in_set(GameSet::Input));
    }
}

fn menu_open(menu: Res<Menu>) -> bool {
    !menu.stack.is_empty()
}

fn open_menu(mut menu: ResMut<Menu>) {
    let screen = if menu.title_done {
        Screen::Main
    } else {
        Screen::Title
    };
    menu.show_only(screen);
}

fn start_flight(mut menu: ResMut<Menu>) {
    menu.title_done = true;
    menu.close();
}

fn open_pause(mut menu: ResMut<Menu>) {
    // A scenario may have stacked screens already.
    if menu.stack.is_empty() {
        menu.open(Screen::Pause);
    }
}

fn resume_flight(mut menu: ResMut<Menu>, mut intent: ResMut<PlayerIntent>) {
    menu.close();
    // Nothing held or pressed before or during the pause carries over.
    *intent = PlayerIntent {
        aim: intent.aim,
        ..default()
    };
}

fn open_end(mut menu: ResMut<Menu>) {
    menu.show_only(Screen::End);
}

fn pause_keys(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<Pause>>) {
    if keys.any_just_pressed([KeyCode::Escape, KeyCode::KeyP]) {
        next.set(Pause::Paused);
    }
}

/// Alt-tab, or a click outside the game's frame on the web.
fn pause_on_focus_loss(
    mut focus: MessageReader<WindowFocused>,
    settings: Res<Settings>,
    mut next: ResMut<NextState<Pause>>,
) {
    let latest = focus.read().last().map(|f| f.focused);
    if latest == Some(false) && settings.pause_unfocused {
        next.set(Pause::Paused);
    }
}

/// Keys that do one thing on one screen.
fn hotkey(screen: Screen, keys: &ButtonInput<KeyCode>) -> Option<MenuAction> {
    match screen {
        Screen::Pause if keys.just_pressed(KeyCode::KeyP) => Some(MenuAction::Resume),
        Screen::End if keys.just_pressed(KeyCode::KeyR) => Some(MenuAction::Play),
        _ => None,
    }
}

fn menu_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut menu: ResMut<Menu>,
    ctx: MenuCtx,
    mut actions: MessageWriter<MenuAction>,
) {
    let Some(top) = menu.top() else {
        return;
    };
    if top.screen == Screen::Title {
        if keys.get_just_pressed().next().is_some() {
            actions.write(MenuAction::Continue);
        }
        return;
    }
    let pressed = |codes: &[KeyCode]| codes.iter().any(|&k| keys.just_pressed(k));
    if pressed(&[KeyCode::Escape]) {
        if let Some(action) = escape_action(top.screen) {
            actions.write(action);
        }
        return;
    }
    if let Some(action) = hotkey(top.screen, &keys) {
        actions.write(action);
        return;
    }
    let items = content(top.screen, &ctx.get()).items;
    let n = items.len();
    if n == 0 {
        return;
    }
    // Several keys can land in one frame (slow frames, fast fingers): take
    // them all, in the order a player means them: move, change, activate.
    let mut focus = top.focus.min(n - 1);
    if pressed(&[KeyCode::ArrowUp, KeyCode::KeyW]) {
        focus = (focus + n - 1) % n;
    }
    if pressed(&[KeyCode::ArrowDown, KeyCode::KeyS]) {
        focus = (focus + 1) % n;
    }
    if focus != top.focus {
        menu.set_focus(focus);
    }
    if let Item::Setting(key) = items[focus] {
        if pressed(&[KeyCode::ArrowLeft, KeyCode::KeyA]) {
            actions.write(MenuAction::Step(key, false));
        }
        if pressed(&[KeyCode::ArrowRight, KeyCode::KeyD]) {
            actions.write(MenuAction::Step(key, true));
        }
    }
    if pressed(&[KeyCode::Enter, KeyCode::NumpadEnter, KeyCode::Space]) {
        actions.write(items[focus].activate());
    }
}

/// A press then a release over the same node is a click (true that frame).
fn clicked(interaction: &Interaction, pressed: &mut bool) -> bool {
    match interaction {
        Interaction::Pressed => {
            *pressed = true;
            false
        }
        Interaction::Hovered => std::mem::take(pressed),
        Interaction::None => {
            *pressed = false;
            false
        }
    }
}

enum Click {
    Row(usize),
    Arrow(StepArrow),
}

/// Hovering moves the highlight (only when the hover starts, so a resting
/// cursor never fights the keyboard); clicking activates.
fn menu_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    mut menu: ResMut<Menu>,
    ctx: MenuCtx,
    mut rows: Query<(&MenuRow, Ref<Interaction>, &mut view::Clicky), Without<StepArrow>>,
    mut arrows: Query<(&StepArrow, Ref<Interaction>, &mut view::Clicky), Without<MenuRow>>,
    mut actions: MessageWriter<MenuAction>,
) {
    let Some(top) = menu.top() else {
        return;
    };
    if top.screen == Screen::Title {
        if mouse.just_released(MouseButton::Left) {
            actions.write(MenuAction::Continue);
        }
        return;
    }
    let mut hover = None;
    let mut click = None;
    for (row, interaction, mut clicky) in &mut rows {
        if clicked(&interaction, &mut clicky.0) {
            click = Some(Click::Row(row.0));
        }
        if interaction.is_changed() && *interaction != Interaction::None {
            hover = Some(row.0);
        }
    }
    for (arrow, interaction, mut clicky) in &mut arrows {
        if clicked(&interaction, &mut clicky.0) {
            click = Some(Click::Arrow(*arrow));
        }
        if interaction.is_changed() && *interaction != Interaction::None {
            hover = Some(arrow.row);
        }
    }
    if let Some(row) = hover.filter(|&r| r != top.focus) {
        menu.set_focus(row);
    }
    match click {
        Some(Click::Row(row)) => {
            if let Some(item) = content(top.screen, &ctx.get()).items.get(row) {
                actions.write(item.activate());
            }
        }
        Some(Click::Arrow(arrow)) => {
            actions.write(MenuAction::Step(arrow.key, arrow.up));
        }
        None => {}
    }
}

/// The one place menu actions change the game.
pub fn apply_actions(
    mut actions: MessageReader<MenuAction>,
    mut menu: ResMut<Menu>,
    mut settings: ResMut<Settings>,
    mut progress: ResMut<Progress>,
    mut notices: ResMut<Notices>,
    mut next_state: ResMut<NextState<AppState>>,
    mut next_pause: ResMut<NextState<Pause>>,
    mut exit: MessageWriter<AppExit>,
) {
    for action in actions.read() {
        match *action {
            MenuAction::Continue => {
                menu.title_done = true;
                menu.show_only(Screen::Main);
            }
            MenuAction::Play => {
                // Entering Playing again keeps a paused sub-state: unpause too.
                next_state.set(AppState::Playing);
                next_pause.set(Pause::Running);
            }
            MenuAction::Open(screen) => menu.open(screen),
            MenuAction::Back => menu.back(),
            MenuAction::Resume => next_pause.set(Pause::Running),
            MenuAction::ToMainMenu => next_state.set(AppState::Menu),
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
            MenuAction::ResetSettings => {
                *settings = Settings::default();
                notices.push("Settings reset.");
            }
            MenuAction::ResetProgress => {
                *progress = Progress::default();
                menu.back();
                notices.push("Progress reset.");
            }
            MenuAction::Step(key, up) => settings.step(key, up),
            MenuAction::Cycle(key) => settings.cycle(key),
        }
    }
}

/// Rebuilds the menu when the top screen, or what it shows, changes.
fn draw_menu(
    mut commands: Commands,
    menu: Res<Menu>,
    ctx: MenuCtx,
    roots: Query<Entity, With<MenuRoot>>,
    asset_server: Option<Res<AssetServer>>,
    mut shown: Local<Option<(Screen, usize)>>,
) {
    let key = menu.top().map(|top| (top.screen, menu.stack.len()));
    if key == *shown && !ctx.changed() {
        return;
    }
    *shown = key;
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some(top) = menu.top() else {
        return;
    };
    // Menus cover the ship; in a flight it shows through, dimmed.
    let backdrop = match ctx.state.get() {
        AppState::Boot | AppState::Menu => palette::VOID,
        AppState::Playing | AppState::Landed | AppState::Lost => palette::UI_PANEL.with_alpha(0.85),
    };
    let snapshot = ctx.get();
    let font = |size| game_font(asset_server.as_deref(), size);
    view::spawn_screen(
        &mut commands,
        &content(top.screen, &snapshot),
        &snapshot.settings,
        top.focus,
        backdrop,
        &font,
    );
}

fn highlight_focus(
    menu: Res<Menu>,
    mut rows: Query<(&MenuRow, &mut BackgroundColor, &mut BorderColor)>,
    mut texts: Query<(&view::RowText, &mut TextColor)>,
) {
    if let Some(top) = menu.top() {
        view::highlight(top.focus, &mut rows, &mut texts);
    }
}

/// `ci/smoke.ron` sends `Custom("play")` to fly past the title screen.
#[cfg(feature = "smoke")]
fn smoke_play(
    mut events: MessageReader<bevy::dev_tools::ci_testing::CiTestingCustomEvent>,
    mut actions: MessageWriter<MenuAction>,
) {
    for event in events.read() {
        if event.0 == "play" {
            actions.write(MenuAction::Play);
        }
    }
}
