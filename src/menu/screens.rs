//! What each screen says and offers, as plain data: [`content`] turns a
//! [`Screen`] and a [`Ctx`] snapshot of the game into a title, text lines and
//! focusable items. `view` draws it; `mod.rs` acts on it.

use crate::faults::FaultKind;
use crate::level::progress::damage_secs;
use crate::level::{Damage, LevelProgress, RunRecord, format_clock};
use crate::settings::{SettingKey, Settings};

use super::{Confirm, MenuAction, Screen};

/// Everything the screens show about the game, gathered once per use.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctx {
    /// Browser build: no Quit, no fullscreen or vsync settings.
    pub web: bool,
    pub settings: Settings,
    pub level_name: String,
    pub level: Option<LevelProgress>,
    /// Seconds to arrival (the pause screen shows it).
    pub time_left: f32,
    pub end: Option<EndInfo>,
}

/// The flight that just ended.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EndInfo {
    pub record: RunRecord,
    pub new_best: bool,
    /// The level's best, this flight included.
    pub best: Option<RunRecord>,
    pub failure: Option<FaultKind>,
    pub fixed: u32,
    pub started: u32,
    pub diag_uses: u32,
    pub tape_left: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Body,
    Dim,
    Accent,
    Heading,
}

/// A focusable row.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Button { label: String, action: MenuAction },
    Setting(SettingKey),
}

impl Item {
    fn button(label: &str, action: MenuAction) -> Self {
        Item::Button {
            label: label.to_string(),
            action,
        }
    }

    /// What Enter, Space or a click does.
    pub fn activate(&self) -> MenuAction {
        match self {
            Item::Button { action, .. } => *action,
            Item::Setting(key) => MenuAction::Cycle(*key),
        }
    }

    /// The row as text (the e2e bridge publishes it).
    pub fn label(&self, settings: &Settings) -> String {
        match self {
            Item::Button { label, .. } => label.clone(),
            Item::Setting(key) => format!("{} {}", key.label(), settings.value_label(*key)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    pub title: String,
    pub big_title: bool,
    pub lines: Vec<(String, Style)>,
    pub items: Vec<Item>,
    /// Dim lines under the items (key hints, progress).
    pub footer: Vec<String>,
}

impl Content {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            big_title: false,
            lines: Vec::new(),
            items: Vec::new(),
            footer: Vec::new(),
        }
    }

    fn line(mut self, text: impl Into<String>, style: Style) -> Self {
        self.lines.push((text.into(), style));
        self
    }

    fn item(mut self, label: &str, action: MenuAction) -> Self {
        self.items.push(Item::button(label, action));
        self
    }

    fn footer(mut self, text: impl Into<String>) -> Self {
        self.footer.push(text.into());
        self
    }
}

pub const GAME_TITLE: &str = "JOURNEY 999";

pub const ENGINEER_LOGS: &[&str] = &[
    "One more journey. Then I'm home.",
    "I just need to survive one more night.",
    "Please don't fail on me this time.",
    "I can still hear home through the static.",
    "A little more tape. A little further.",
    "They'll leave a light on. They have to.",
    "Hold together. We're almost there.",
    "One more journey. That's what I said last time.",
];

pub fn content(screen: Screen, ctx: &Ctx) -> Content {
    match screen {
        Screen::Title => {
            // On the web, keys reach the game only once its frame has focus: a
            // click always works.
            let mut c = Content::new(GAME_TITLE).line("CLICK OR PRESS ANY KEY", Style::Accent);
            c.big_title = true;
            c
        }
        Screen::Main => main_menu(ctx),
        Screen::Story(index) => {
            Content::new("").line(ENGINEER_LOGS[index % ENGINEER_LOGS.len()], Style::Body)
        }
        Screen::HowToPlay => how_to_play(),
        Screen::Settings => settings(ctx),
        Screen::Pause => pause(ctx),
        Screen::End => end(ctx),
        Screen::Confirm(kind) => confirm(kind),
    }
}

/// Esc: back out of a screen (or leave it the natural way).
pub fn escape_action(screen: Screen) -> Option<MenuAction> {
    match screen {
        Screen::Title => Some(MenuAction::Continue),
        Screen::Story(_) => Some(MenuAction::Launch),
        Screen::Main => None,
        Screen::HowToPlay | Screen::Settings | Screen::Confirm(_) => Some(MenuAction::Back),
        Screen::Pause => Some(MenuAction::Resume),
        Screen::End => Some(MenuAction::ToMainMenu),
    }
}

fn main_menu(ctx: &Ctx) -> Content {
    let mut c = Content::new(GAME_TITLE)
        .item("PLAY", MenuAction::Play)
        .item("HOW TO PLAY", MenuAction::Open(Screen::HowToPlay))
        .item("SETTINGS", MenuAction::Open(Screen::Settings));
    if !ctx.web {
        c = c.item("QUIT", MenuAction::Quit);
    }
    if let Some(line) = progress_line(&ctx.level_name, ctx.level.as_ref()) {
        c = c.footer(line);
    }
    c
}

/// The main menu's summary of a level, once it has been flown.
pub fn progress_line(level_name: &str, level: Option<&LevelProgress>) -> Option<String> {
    let level = level.filter(|l| l.flights > 0)?;
    let best = level.best?;
    let best = if best.landed {
        format!("LANDED, {}s DAMAGE", damage_secs(best.damage.total()))
    } else {
        format!("SURVIVED {}", format_clock(best.survived))
    };
    let flights = match level.flights {
        1 => "1 FLIGHT".to_string(),
        n => format!("{n} FLIGHTS"),
    };
    Some(format!(
        "{}  -  BEST: {best}  -  {flights}",
        level_name.to_uppercase()
    ))
}

/// "landed, 34s damage" or "survived 2:31".
fn best_summary(best: &RunRecord) -> String {
    if best.landed {
        format!("landed, {}s damage", damage_secs(best.damage.total()))
    } else {
        format!("survived {}", format_clock(best.survived))
    }
}

fn how_to_play() -> Content {
    Content::new("HOW TO PLAY")
        .line(
            "Keep the ship together until it lands. Faults break out around the ship,",
            Style::Body,
        )
        .line(
            "and each one ends the flight if it runs for too long.",
            Style::Body,
        )
        .line("CONTROLS", Style::Heading)
        .line(
            "WASD move    Mouse aim    Click use the tool    1 / 2 or wheel switch tool",
            Style::Body,
        )
        .line(
            "E use the helm or the console    Esc or P pause",
            Style::Body,
        )
        .line("FAULTS", Style::Heading)
        .line(
            "Loose bolts (engine room): turn each loose bolt with the wrench.",
            Style::Body,
        )
        .line(
            "Hull breach (airlock, main hull): hold the tape over the hole.",
            Style::Body,
        )
        .line(
            "Trajectory drift (cockpit): take the helm and steer into the centre band.",
            Style::Body,
        )
        .line(
            "The console in your quarters shows where every fault is. A red tint and",
            Style::Dim,
        )
        .line(
            "shaking mean a fault is getting worse. The sooner it is fixed, the better.",
            Style::Dim,
        )
        .item("BACK", MenuAction::Back)
}

fn settings(ctx: &Ctx) -> Content {
    let mut c = Content::new("SETTINGS");
    for key in SettingKey::ALL {
        if key.available(ctx.web) {
            c.items.push(Item::Setting(key));
        }
    }
    c.item("RESET SETTINGS", MenuAction::ResetSettings)
        .item(
            "RESET PROGRESS",
            MenuAction::Open(Screen::Confirm(Confirm::ResetProgress)),
        )
        .item("BACK", MenuAction::Back)
        .footer("Up / Down choose    Left / Right change    Esc back")
}

fn pause(ctx: &Ctx) -> Content {
    let mut c = Content::new("PAUSED")
        .line(
            format!("ARRIVAL IN {}", format_clock(ctx.time_left)),
            Style::Dim,
        )
        .item("RESUME", MenuAction::Resume)
        .item(
            "RESTART",
            MenuAction::Open(Screen::Confirm(Confirm::Restart)),
        )
        .item("HOW TO PLAY", MenuAction::Open(Screen::HowToPlay))
        .item("SETTINGS", MenuAction::Open(Screen::Settings))
        .item(
            "MAIN MENU",
            MenuAction::Open(Screen::Confirm(Confirm::MainMenu)),
        );
    if !ctx.web {
        c = c.item("QUIT", MenuAction::Open(Screen::Confirm(Confirm::Quit)));
    }
    c.footer("Esc or P  resume")
}

fn confirm(kind: Confirm) -> Content {
    let (title, text, label, action) = match kind {
        Confirm::Restart => (
            "RESTART THE FLIGHT?",
            "This flight will be lost.",
            "RESTART",
            MenuAction::Play,
        ),
        Confirm::MainMenu => (
            "LEAVE THE FLIGHT?",
            "This flight will be lost.",
            "MAIN MENU",
            MenuAction::ToMainMenu,
        ),
        Confirm::Quit => (
            "QUIT THE GAME?",
            "This flight will be lost.",
            "QUIT",
            MenuAction::Quit,
        ),
        Confirm::ResetProgress => (
            "RESET PROGRESS?",
            "Your best flights will be erased. Settings are kept.",
            "RESET",
            MenuAction::ResetProgress,
        ),
    };
    // Back first: Enter on a confirm screen never destroys anything.
    Content::new(title)
        .line(text, Style::Body)
        .item("BACK", MenuAction::Back)
        .item(label, action)
}

/// "Oxygen lost 12s    Off course 3s    Engine overheating 0s".
pub fn damage_line(d: &Damage) -> String {
    format!(
        "Oxygen lost {}s    Off course {}s    Engine overheating {}s",
        damage_secs(d.oxygen),
        damage_secs(d.course),
        damage_secs(d.engine)
    )
}

fn end(ctx: &Ctx) -> Content {
    let Some(e) = ctx.end else {
        return Content::new("").item("MAIN MENU", MenuAction::ToMainMenu);
    };
    let r = e.record;
    let mut c = if r.landed {
        Content::new("TOUCHDOWN")
            .line("You kept the ship together all the way down.", Style::Body)
            .line(damage_line(&r.damage), Style::Body)
            .line(
                format!(
                    "Damage {}s    Faults fixed {} of {}    Diagnostics used {}    Tape left {}s",
                    damage_secs(r.damage.total()),
                    e.fixed,
                    e.started,
                    e.diag_uses,
                    e.tape_left.ceil() as u32
                ),
                Style::Dim,
            )
    } else {
        let kind = e.failure;
        Content::new(&kind.map_or("LOST", |k| k.failure_title()).to_uppercase())
            .line(kind.map_or("", |k| k.failure_text()), Style::Body)
            .line(
                format!(
                    "Survived {} of {}    Faults fixed {}",
                    format_clock(r.survived),
                    format_clock(e.duration),
                    e.fixed
                ),
                Style::Body,
            )
            .line(damage_line(&r.damage), Style::Dim)
    };
    c = match (e.new_best, e.best) {
        (true, _) => c.line("NEW BEST", Style::Accent),
        (false, Some(best)) => c.line(format!("Best: {}", best_summary(&best)), Style::Dim),
        (false, None) => c,
    };
    c.item("FLY AGAIN (R)", MenuAction::Play)
        .item("MAIN MENU", MenuAction::ToMainMenu)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(web: bool) -> Ctx {
        Ctx {
            web,
            settings: Settings::default(),
            level_name: "Level 1".into(),
            level: None,
            time_left: 151.0,
            end: None,
        }
    }

    fn labels(c: &Content) -> Vec<String> {
        c.items
            .iter()
            .map(|i| i.label(&Settings::default()))
            .collect()
    }

    #[test]
    fn engineer_logs_are_single_lines_and_wrap() {
        for index in 0..ENGINEER_LOGS.len() {
            let log = content(Screen::Story(index), &ctx(false));
            assert!(log.title.is_empty());
            assert!(log.items.is_empty());
            assert!(log.footer.is_empty());
            assert_eq!(log.lines.len(), 1);
            assert!(log.lines[0].0.is_ascii());
            assert!(!log.lines[0].0.contains('\n'));
        }
        assert_eq!(
            content(Screen::Story(ENGINEER_LOGS.len()), &ctx(false)),
            content(Screen::Story(0), &ctx(false))
        );
    }

    #[test]
    fn the_web_build_has_no_quit() {
        for screen in [Screen::Main, Screen::Pause] {
            assert!(labels(&content(screen, &ctx(false))).contains(&"QUIT".into()));
            assert!(!labels(&content(screen, &ctx(true))).contains(&"QUIT".into()));
        }
        let web_settings = labels(&content(Screen::Settings, &ctx(true)));
        assert!(!web_settings.iter().any(|l| l.starts_with("FULLSCREEN")));
    }

    #[test]
    fn menus_lead_with_the_obvious_choice() {
        assert_eq!(
            content(Screen::Main, &ctx(true)).items[0].activate(),
            MenuAction::Play
        );
        let pause = content(Screen::Pause, &ctx(true));
        assert_eq!(pause.items[0].activate(), MenuAction::Resume);
        assert_eq!(pause.lines[0].0, "ARRIVAL IN 2:31");
        for kind in [
            Confirm::Restart,
            Confirm::MainMenu,
            Confirm::Quit,
            Confirm::ResetProgress,
        ] {
            let c = content(Screen::Confirm(kind), &ctx(false));
            assert_eq!(c.items[0].activate(), MenuAction::Back, "{kind:?}");
        }
    }

    #[test]
    fn settings_rows_show_their_values_and_cycle() {
        let c = content(Screen::Settings, &ctx(false));
        assert_eq!(c.items[0].label(&Settings::default()), "SCREEN SHAKE 100%");
        assert_eq!(c.items[0].activate(), MenuAction::Cycle(SettingKey::Shake));
        assert_eq!(c.items.last().unwrap().activate(), MenuAction::Back);
    }

    #[test]
    fn escape_backs_out_of_every_screen_but_the_main_menu() {
        assert_eq!(escape_action(Screen::Main), None);
        assert_eq!(escape_action(Screen::Pause), Some(MenuAction::Resume));
        assert_eq!(
            escape_action(Screen::Confirm(Confirm::Quit)),
            Some(MenuAction::Back)
        );
        assert_eq!(escape_action(Screen::End), Some(MenuAction::ToMainMenu));
    }

    fn record(landed: bool, oxygen: f32, survived: f32) -> RunRecord {
        RunRecord {
            landed,
            damage: Damage {
                oxygen,
                course: 2.0,
                engine: 0.4,
            },
            survived,
        }
    }

    #[test]
    fn the_end_screen_scores_the_flight() {
        let landed = record(true, 11.6, 240.0);
        let mut c = ctx(false);
        c.end = Some(EndInfo {
            record: landed,
            new_best: true,
            best: Some(landed),
            failure: None,
            fixed: 10,
            started: 10,
            diag_uses: 2,
            tape_left: 7.2,
            duration: 240.0,
        });
        let shown = content(Screen::End, &c);
        assert_eq!(shown.title, "TOUCHDOWN");
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.contains(&"Oxygen lost 12s    Off course 2s    Engine overheating 0s"));
        assert!(text.iter().any(|t| t.starts_with("Damage 14s")));
        assert!(text.contains(&"NEW BEST"));
        assert_eq!(shown.items[0].activate(), MenuAction::Play);

        let lost = record(false, 45.0, 150.5);
        c.end = Some(EndInfo {
            record: lost,
            new_best: false,
            best: Some(landed),
            failure: Some(FaultKind::HullBreach),
            fixed: 3,
            ..c.end.unwrap()
        });
        let shown = content(Screen::End, &c);
        assert_eq!(shown.title, "OXYGEN DEPLETED");
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.contains(&"Survived 2:31 of 4:00    Faults fixed 3"));
        assert!(text.contains(&"Best: landed, 14s damage"));
    }

    #[test]
    fn the_main_menu_sums_up_a_flown_level() {
        assert_eq!(progress_line("Level 1", None), None);
        let flown = LevelProgress {
            flights: 3,
            landings: 1,
            best: Some(record(true, 20.0, 240.0)),
        };
        assert_eq!(
            progress_line("Level 1", Some(&flown)).unwrap(),
            "LEVEL 1  -  BEST: LANDED, 22s DAMAGE  -  3 FLIGHTS"
        );
    }
}
