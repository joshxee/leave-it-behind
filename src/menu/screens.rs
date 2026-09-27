//! What each screen says and offers, as plain data: [`content`] turns a
//! [`Screen`] and a [`Ctx`] snapshot of the game into a title, text lines and
//! focusable items. `view` draws it; `mod.rs` acts on it.

use crate::faults::FaultKind;
use crate::level::progress::damage_secs;
use crate::level::{Damage, LEVEL_COUNT, LevelProgress, RunRecord, format_clock};
use crate::settings::{SettingKey, Settings};
use crate::ui::repairs_label;
use crate::upgrades::{Upgrade, Upgrades};

use super::{Confirm, MenuAction, Screen};

/// Everything the screens show about the game, gathered once per use.
#[derive(Debug, Clone, PartialEq)]
pub struct Ctx {
    /// Browser build: no Quit, no fullscreen or vsync settings.
    pub web: bool,
    pub settings: Settings,
    pub level_name: String,
    /// The level's place in the campaign (1-based).
    pub level_number: usize,
    pub level: Option<LevelProgress>,
    /// The level after this one: its name and how many faults it has. None
    /// on the last.
    pub next_level: Option<(String, usize)>,
    pub upgrades: Upgrades,
    /// Campaign levels landed at least once, and flights flown in all.
    pub levels_landed: usize,
    pub flights: u32,
    /// Faults fixed and the flight's total (the pause screen shows them).
    pub repairs: (u32, u32),
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
    /// Faults the flight had to fix to land.
    pub total: u32,
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

pub const GAME_TITLE: &str = "LEAVE IT BEHIND";

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
        Screen::HowToPlay => how_to_play(),
        Screen::Settings => settings(ctx),
        Screen::Pause => pause(ctx),
        Screen::End => end(ctx),
        Screen::Upgrade => upgrade(ctx),
        Screen::Confirm(kind) => confirm(kind),
    }
}

/// Esc: back out of a screen (or leave it the natural way).
pub fn escape_action(screen: Screen) -> Option<MenuAction> {
    match screen {
        Screen::Title => Some(MenuAction::Continue),
        Screen::Main => None,
        Screen::HowToPlay | Screen::Settings | Screen::Upgrade | Screen::Confirm(_) => {
            Some(MenuAction::Back)
        }
        Screen::Pause => Some(MenuAction::Resume),
        Screen::End => Some(MenuAction::ToMainMenu),
    }
}

fn main_menu(ctx: &Ctx) -> Content {
    let mut c = Content::new(GAME_TITLE)
        .item("PLAY", MenuAction::NewGame)
        .item("HOW TO PLAY", MenuAction::Open(Screen::HowToPlay))
        .item("SETTINGS", MenuAction::Open(Screen::Settings));
    if !ctx.web {
        c = c.item("QUIT", MenuAction::Quit);
    }
    if let Some(line) = progress_line(ctx.levels_landed, ctx.flights) {
        c = c.footer(line);
    }
    c
}

/// The main menu's summary of the campaign, once a flight has been flown.
pub fn progress_line(levels_landed: usize, flights: u32) -> Option<String> {
    let flights = match flights {
        0 => return None,
        1 => "1 FLIGHT".to_string(),
        n => format!("{n} FLIGHTS"),
    };
    Some(format!(
        "LEVELS LANDED: {levels_landed} OF {LEVEL_COUNT}  -  {flights}"
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
            "Fix every fault and the ship lands. Faults break out around the ship,",
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
            "Hull breach (any hull wall but the quarters): hold the tape over the hole.",
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
            format!(
                "{}    {}",
                ctx.level_name.to_uppercase(),
                repairs_label(ctx.repairs.0, ctx.repairs.1)
            )
            .trim_end(),
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
            .line(
                format!(
                    "{} of {LEVEL_COUNT}: you kept the ship together all the way down.",
                    ctx.level_name
                ),
                Style::Body,
            )
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
                    "{}    Survived {}    Faults fixed {} of {}",
                    ctx.level_name,
                    format_clock(r.survived),
                    e.fixed,
                    e.total
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
    if r.landed && ctx.next_level.is_some() {
        // On to the next level, by way of an upgrade.
        c = c.item("CONTINUE", MenuAction::Open(Screen::Upgrade));
    } else if r.landed {
        c = c.line(
            format!("ALL {LEVEL_COUNT} LEVELS LANDED. THE SHIP MADE IT HOME."),
            Style::Accent,
        );
    }
    c.item("FLY AGAIN (R)", MenuAction::Play)
        .item("MAIN MENU", MenuAction::ToMainMenu)
}

/// Between levels: one upgrade to keep, then the next level.
fn upgrade(ctx: &Ctx) -> Content {
    let mut c = Content::new("CHOOSE AN UPGRADE").line(
        "Pick one. It stays with you for the rest of the flights.",
        Style::Body,
    );
    for u in Upgrade::ALL {
        let taken = match ctx.upgrades.count(u) {
            0 => String::new(),
            n => format!("  (taken {n})"),
        };
        c = c.line(format!("{}{taken}", u.describe()), Style::Dim);
    }
    for u in Upgrade::ALL {
        c = c.item(u.label(), MenuAction::NextLevel(u));
    }
    if let Some((name, faults)) = &ctx.next_level {
        c = c.footer(format!("NEXT: {}, {faults} FAULTS", name.to_uppercase()));
    }
    c.footer("Esc back")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(web: bool) -> Ctx {
        Ctx {
            web,
            settings: Settings::default(),
            level_name: "Level 1".into(),
            level_number: 1,
            level: None,
            next_level: Some(("Level 2".into(), 7)),
            upgrades: Upgrades::default(),
            levels_landed: 0,
            flights: 0,
            repairs: (1, 4),
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
            MenuAction::NewGame
        );
        let pause = content(Screen::Pause, &ctx(true));
        assert_eq!(pause.items[0].activate(), MenuAction::Resume);
        assert_eq!(pause.lines[0].0, "LEVEL 1    REPAIRS 1 OF 4");
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
        assert_eq!(escape_action(Screen::Upgrade), Some(MenuAction::Back));
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

    fn ended(record: RunRecord) -> EndInfo {
        EndInfo {
            record,
            new_best: true,
            best: Some(record),
            failure: None,
            fixed: 10,
            started: 10,
            diag_uses: 2,
            tape_left: 7.2,
            total: 10,
        }
    }

    #[test]
    fn the_end_screen_scores_the_flight() {
        let landed = record(true, 11.6, 240.0);
        let mut c = ctx(false);
        c.end = Some(ended(landed));
        let shown = content(Screen::End, &c);
        assert_eq!(shown.title, "TOUCHDOWN");
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.contains(&"Level 1 of 5: you kept the ship together all the way down."));
        assert!(text.contains(&"Oxygen lost 12s    Off course 2s    Engine overheating 0s"));
        assert!(text.iter().any(|t| t.starts_with("Damage 14s")));
        assert!(text.contains(&"NEW BEST"));

        let lost = record(false, 45.0, 150.5);
        c.end = Some(EndInfo {
            record: lost,
            new_best: false,
            best: Some(landed),
            failure: Some(FaultKind::HullBreach),
            fixed: 3,
            ..ended(landed)
        });
        let shown = content(Screen::End, &c);
        assert_eq!(shown.title, "OXYGEN DEPLETED");
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.contains(&"Level 1    Survived 2:31    Faults fixed 3 of 10"));
        assert!(text.contains(&"Best: landed, 14s damage"));
        assert_eq!(labels(&shown), vec!["FLY AGAIN (R)", "MAIN MENU"]);
    }

    #[test]
    fn landing_leads_on_to_the_upgrades_until_the_last_level() {
        let mut c = ctx(false);
        c.end = Some(ended(record(true, 3.0, 150.0)));
        let shown = content(Screen::End, &c);
        assert_eq!(
            labels(&shown),
            vec!["CONTINUE", "FLY AGAIN (R)", "MAIN MENU"]
        );
        assert_eq!(shown.items[0].activate(), MenuAction::Open(Screen::Upgrade));

        c.level_name = "Level 5".into();
        c.level_number = 5;
        c.next_level = None;
        let shown = content(Screen::End, &c);
        assert_eq!(labels(&shown), vec!["FLY AGAIN (R)", "MAIN MENU"]);
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(text.contains(&"ALL 5 LEVELS LANDED. THE SHIP MADE IT HOME."));
    }

    #[test]
    fn the_upgrade_screen_offers_three_and_names_the_next_level() {
        let mut c = ctx(false);
        c.upgrades.choose(2, Upgrade::WiderTape);
        let shown = content(Screen::Upgrade, &c);
        assert_eq!(
            labels(&shown),
            vec!["RUN FASTER", "FASTER WRENCH", "WIDER TAPE"]
        );
        for (item, u) in shown.items.iter().zip(Upgrade::ALL) {
            assert_eq!(item.activate(), MenuAction::NextLevel(u));
        }
        let text: Vec<&str> = shown.lines.iter().map(|(t, _)| t.as_str()).collect();
        assert!(
            text.iter()
                .any(|t| t.starts_with("Wider tape") && t.ends_with("(taken 1)"))
        );
        assert!(
            text.iter()
                .any(|t| t.starts_with("Run faster") && !t.contains("taken"))
        );
        assert_eq!(shown.footer[0], "NEXT: LEVEL 2, 7 FAULTS");
    }

    #[test]
    fn the_main_menu_sums_up_the_campaign() {
        assert_eq!(progress_line(0, 0), None);
        assert_eq!(
            progress_line(2, 7).unwrap(),
            "LEVELS LANDED: 2 OF 5  -  7 FLIGHTS"
        );
        assert_eq!(
            progress_line(0, 1).unwrap(),
            "LEVELS LANDED: 0 OF 5  -  1 FLIGHT"
        );
    }
}
