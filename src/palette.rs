//! Placeholder colors, in one place so the look stays consistent.

use bevy::prelude::*;

/// Outside the ship, and the curtains that hide other rooms.
pub const VOID: Color = Color::srgb(0.02, 0.02, 0.05);
pub const WALL: Color = Color::srgb(0.36, 0.40, 0.47);
pub const DOOR_FRAME: Color = Color::srgb(0.95, 0.75, 0.25);
/// The walkway along the corridor spine.
pub const SPINE: Color = Color::srgba(1.0, 1.0, 1.0, 0.025);
pub const WINDOW: Color = Color::srgb(0.35, 0.60, 0.85);
pub const HATCH: Color = Color::srgb(0.85, 0.65, 0.15);
pub const PROP: Color = Color::srgb(0.25, 0.28, 0.33);
pub const CRATE: Color = Color::srgb(0.42, 0.33, 0.22);
pub const BUNK: Color = Color::srgb(0.30, 0.36, 0.50);

pub const ENGINE: Color = Color::srgb(0.45, 0.45, 0.48);
pub const ENGINE_HOT: Color = Color::srgb(0.95, 0.25, 0.10);
pub const BOLT_TIGHT: Color = Color::srgb(0.62, 0.64, 0.68);
pub const BOLT_LOOSE: Color = Color::srgb(1.0, 0.62, 0.15);

pub const BREACH_HOLE: Color = Color::srgb(0.0, 0.0, 0.0);
pub const BREACH_AIR: Color = Color::srgba(0.85, 0.95, 1.0, 0.8);
pub const TAPE: Color = Color::srgb(0.86, 0.78, 0.55);

pub const NAV_SCREEN: Color = Color::srgb(0.03, 0.10, 0.08);
pub const NAV_BAND: Color = Color::srgba(0.30, 1.0, 0.55, 0.12);
pub const NAV_MARKER: Color = Color::srgb(0.55, 1.0, 0.70);
pub const NAV_ALERT: Color = Color::srgb(1.0, 0.30, 0.25);
pub const JOYSTICK: Color = Color::srgb(0.90, 0.20, 0.20);
pub const JOYSTICK_BASE: Color = Color::srgb(0.08, 0.08, 0.10);

pub const DIAG_SCREEN: Color = Color::srgb(0.10, 0.45, 0.35);

pub const PLAYER: Color = Color::srgb(1.0, 0.55, 0.20);
pub const STEEL: Color = Color::srgb(0.78, 0.80, 0.85);
pub const SNAP: Color = Color::srgb(0.55, 1.0, 0.45);

pub const UI_TEXT: Color = Color::srgb(0.92, 0.94, 0.97);
pub const UI_DIM: Color = Color::srgba(0.92, 0.94, 0.97, 0.45);
pub const UI_ACCENT: Color = Color::srgb(1.0, 0.80, 0.30);
pub const UI_PANEL: Color = Color::srgba(0.02, 0.03, 0.06, 0.92);
pub const ALERT: Color = Color::srgb(1.0, 0.22, 0.18);
pub const GOOD: Color = Color::srgb(0.45, 0.95, 0.55);

/// Floor color of each room, front to tail, so rooms are told apart at a glance.
pub const FLOORS: [Color; 5] = [
    Color::srgb(0.10, 0.14, 0.24),
    Color::srgb(0.09, 0.17, 0.17),
    Color::srgb(0.18, 0.13, 0.09),
    Color::srgb(0.13, 0.13, 0.15),
    Color::srgb(0.17, 0.10, 0.16),
];
