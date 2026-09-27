//! The vitals panel, top right: oxygen and engine heat as bars, and the
//! course, which counts down to impact while the ship drifts. It says what
//! is failing and how soon, never where (that is the diagnostic screen's
//! job). Cold while all is well; a row turns warm and pulses (with the alarm
//! flashing setting) while a fault of its kind is active.

use bevy::prelude::*;

use super::{Hud, game_font};
use crate::faults::{Fault, FaultKind, Vitals};
use crate::level::format_clock;
use crate::palette;
use crate::settings::Settings;

/// Bar width in UI pixels.
const BAR_WIDTH: f32 = 120.0;

/// One row of the panel.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gauge {
    Oxygen,
    Heat,
    Course,
}

impl Gauge {
    pub const ALL: [Gauge; 3] = [Gauge::Oxygen, Gauge::Heat, Gauge::Course];

    /// The fault kind this row watches.
    pub fn kind(self) -> FaultKind {
        match self {
            Gauge::Oxygen => FaultKind::HullBreach,
            Gauge::Heat => FaultKind::LooseBolts,
            Gauge::Course => FaultKind::TrajectoryDrift,
        }
    }
}

/// What one row shows.
#[derive(Debug, Clone, PartialEq)]
pub struct GaugeReading {
    pub label: &'static str,
    /// Bar fill, 0 to 1. The course has no bar.
    pub fill: Option<f32>,
    pub value: String,
    /// A fault of this row's kind is active.
    pub alert: bool,
}

/// What `gauge`'s row says for these vitals and faults. Oxygen reads 0%
/// only when it is gone and heat 100% only when the engine has overheated.
pub fn gauge<'a>(
    gauge: Gauge,
    vitals: &Vitals,
    faults: impl IntoIterator<Item = &'a Fault>,
) -> GaugeReading {
    let active: Vec<&Fault> = faults
        .into_iter()
        .filter(|f| f.kind() == gauge.kind() && !f.is_repaired())
        .collect();
    let alert = !active.is_empty();
    match gauge {
        Gauge::Oxygen => {
            let left = vitals.oxygen_left();
            GaugeReading {
                label: "O2",
                fill: Some(left),
                value: format!("{}%", (left * 100.0).ceil() as u32),
                alert,
            }
        }
        Gauge::Heat => {
            let heat = vitals.heat.spent;
            GaugeReading {
                label: "HEAT",
                fill: Some(heat),
                value: format!("{}%", (heat * 100.0).floor() as u32),
                alert,
            }
        }
        Gauge::Course => match active.iter().map(|f| vitals.time_left(f)).reduce(f32::min) {
            Some(left) => GaugeReading {
                label: "IMPACT IN",
                fill: None,
                value: format_clock(left),
                alert,
            },
            None => GaugeReading {
                label: "COURSE",
                fill: None,
                value: "OK".into(),
                alert,
            },
        },
    }
}

#[derive(Component, Debug)]
pub struct GaugeLabel(pub Gauge);

#[derive(Component, Debug)]
pub struct GaugeValue(pub Gauge);

#[derive(Component, Debug)]
pub struct GaugeFill(pub Gauge);

pub(super) fn spawn_vitals(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    let font = || game_font(asset_server.as_deref(), 18.0);
    commands
        .spawn((
            Hud,
            Node {
                position_type: PositionType::Absolute,
                top: px(12),
                right: px(16),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                padding: UiRect::axes(px(10), px(8)),
                ..default()
            },
            BackgroundColor(palette::UI_PANEL.with_alpha(0.6)),
            GlobalZIndex(10),
        ))
        .with_children(|panel| {
            for g in Gauge::ALL {
                panel
                    .spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: px(8),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            GaugeLabel(g),
                            Text::new(""),
                            // One line, however the row is squeezed ("IMPACT IN").
                            TextLayout::no_wrap(),
                            font(),
                            TextColor(palette::UI_DIM),
                            Node {
                                min_width: px(52),
                                ..default()
                            },
                        ));
                        if g != Gauge::Course {
                            row.spawn((
                                Node {
                                    width: px(BAR_WIDTH),
                                    height: px(10),
                                    ..default()
                                },
                                BackgroundColor(palette::GAUGE_TRACK),
                            ))
                            .with_child((
                                GaugeFill(g),
                                Node {
                                    width: percent(0),
                                    height: percent(100),
                                    ..default()
                                },
                                BackgroundColor(palette::GAUGE_FILL),
                            ));
                        }
                        row.spawn((
                            GaugeValue(g),
                            Text::new(""),
                            TextLayout::no_wrap(),
                            font(),
                            TextColor(palette::UI_TEXT),
                        ));
                    });
            }
        });
}

pub(super) fn update_vitals(
    vitals: Res<Vitals>,
    faults: Query<&Fault>,
    time: Res<Time>,
    settings: Res<Settings>,
    mut labels: Query<(&GaugeLabel, &mut Text, &mut TextColor), Without<GaugeValue>>,
    mut values: Query<(&GaugeValue, &mut Text, &mut TextColor), Without<GaugeLabel>>,
    mut fills: Query<(&GaugeFill, &mut Node, &mut BackgroundColor)>,
) {
    let readings = Gauge::ALL.map(|g| gauge(g, &vitals, faults.iter()));
    let reading = |g: Gauge| &readings[g as usize];
    let wave = (time.elapsed_secs() * 6.0).sin().abs();
    let warm = palette::ALERT.with_alpha(1.0 - 0.45 * settings.flash_scale() * (1.0 - wave));
    for (label, mut text, mut color) in &mut labels {
        let r = reading(label.0);
        if text.0 != r.label {
            text.0 = r.label.to_string();
        }
        color.0 = if r.alert { warm } else { palette::UI_DIM };
    }
    for (value, mut text, mut color) in &mut values {
        let r = reading(value.0);
        if text.0 != r.value {
            text.0 = r.value.clone();
        }
        color.0 = if r.alert { warm } else { palette::UI_TEXT };
    }
    for (fill, mut node, mut bg) in &mut fills {
        let r = reading(fill.0);
        let width = percent(100.0 * r.fill.unwrap_or(0.0));
        if node.width != width {
            node.width = width;
        }
        bg.0 = if r.alert { warm } else { palette::GAUGE_FILL };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faults::Site;

    #[test]
    fn a_quiet_ship_reads_full_cool_and_on_course() {
        let v = Vitals::default();
        let none: [&Fault; 0] = [];
        let o2 = gauge(Gauge::Oxygen, &v, none);
        assert_eq!(
            (o2.label, o2.fill, o2.value.as_str()),
            ("O2", Some(1.0), "100%")
        );
        let heat = gauge(Gauge::Heat, &v, none);
        assert_eq!((heat.fill, heat.value.as_str()), (Some(0.0), "0%"));
        let course = gauge(Gauge::Course, &v, none);
        assert_eq!((course.label, course.value.as_str()), ("COURSE", "OK"));
        assert!(!o2.alert && !heat.alert && !course.alert);
    }

    #[test]
    fn only_the_failing_system_alerts() {
        let mut v = Vitals::default();
        v.oxygen.spent = 0.225;
        v.heat.spent = 0.305;
        let breach = Fault::new(Site::AirlockPortAft, 60.0);
        let o2 = gauge(Gauge::Oxygen, &v, [&breach]);
        assert_eq!(o2.value, "78%");
        assert!(o2.alert);
        let heat = gauge(Gauge::Heat, &v, [&breach]);
        assert_eq!(
            heat.value, "30%",
            "still cooling, but nothing is heating it"
        );
        assert!(!heat.alert);
    }

    #[test]
    fn drift_counts_down_to_impact() {
        let v = Vitals::default();
        let mut drift = Fault::new(Site::Helm, 60.0);
        drift.elapsed = 17.5;
        let course = gauge(Gauge::Course, &v, [&drift]);
        assert_eq!((course.label, course.value.as_str()), ("IMPACT IN", "0:43"));
        assert!(course.alert && course.fill.is_none());
    }

    #[test]
    fn percentages_hit_the_ends_only_when_fatal() {
        let mut v = Vitals::default();
        v.oxygen.spent = 0.999;
        v.heat.spent = 0.999;
        let none: [&Fault; 0] = [];
        assert_eq!(gauge(Gauge::Oxygen, &v, none).value, "1%");
        assert_eq!(gauge(Gauge::Heat, &v, none).value, "99%");
        v.oxygen.spent = 1.0;
        assert_eq!(gauge(Gauge::Oxygen, &v, none).value, "0%");
    }
}
