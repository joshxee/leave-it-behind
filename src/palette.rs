//! Colors, in one place. The art packs share one 15-colour cold palette
//! (`assets/characters/engineer/engineer-16.gpl`): everything decorative uses
//! it. Warm colours are reserved for danger (a fault and what it does), so
//! trouble stands out against the ship. A unit test enforces both rules.

use bevy::prelude::*;

// The pack palette, darkest first (engineer-16.gpl entries 1 to 15).
pub const OUTLINE: Color = Color::srgb_u8(16, 28, 41);
pub const DEEP_SEAM: Color = Color::srgb_u8(27, 45, 62);
pub const JOINT_SHADOW: Color = Color::srgb_u8(44, 66, 85);
pub const STEEL_SHADOW: Color = Color::srgb_u8(64, 92, 112);
pub const STEEL: Color = Color::srgb_u8(86, 119, 139);
pub const PLATE_MID: Color = Color::srgb_u8(117, 149, 165);
pub const PLATE_LIGHT: Color = Color::srgb_u8(154, 179, 189);
pub const WORN_EDGE: Color = Color::srgb_u8(191, 208, 214);
pub const SPECULAR: Color = Color::srgb_u8(230, 239, 241);
pub const VISOR_DEEP: Color = Color::srgb_u8(18, 53, 79);
pub const VISOR_BLUE: Color = Color::srgb_u8(28, 88, 123);
pub const VISOR_MID: Color = Color::srgb_u8(51, 136, 173);
pub const VISOR_LIGHT: Color = Color::srgb_u8(97, 187, 210);
pub const VISOR_GLINT: Color = Color::srgb_u8(160, 224, 235);
pub const REPAIR_FABRIC: Color = Color::srgb_u8(129, 152, 163);

/// Everything outside the current room.
pub const VOID: Color = OUTLINE;

// Placeholder props (no art yet: see docs/ART_REQUESTS.md).
pub const ENGINE: Color = STEEL;
pub const ENGINE_PANEL: Color = JOINT_SHADOW;
pub const ENGINE_VENT: Color = DEEP_SEAM;
pub const BUNK: Color = STEEL_SHADOW;
pub const PILLOW: Color = PLATE_MID;
pub const BOLT_TIGHT: Color = PLATE_LIGHT;
pub const TAPE: Color = WORN_EDGE;
pub const JOYSTICK: Color = VISOR_LIGHT;
pub const JOYSTICK_BASE: Color = OUTLINE;
pub const JOYSTICK_RING: Color = PLATE_LIGHT;
pub const NAV_FRAME: Color = STEEL_SHADOW;
pub const NAV_SCREEN: Color = VISOR_DEEP;
pub const NAV_BAND: Color = Color::srgba_u8(51, 136, 173, 64);
pub const NAV_MARKER: Color = VISOR_GLINT;
/// Air rushing out of a breach (a cue, but not a warm one: it is the room's air).
pub const BREACH_AIR: Color = Color::srgba_u8(230, 239, 241, 204);
/// The ring on a bolt the wrench is snapped to.
pub const SNAP: Color = VISOR_LIGHT;

// Danger: the only warm colours in the game.
pub const ENGINE_HOT: Color = Color::srgb(0.95, 0.25, 0.10);
pub const BOLT_LOOSE: Color = Color::srgb(1.0, 0.62, 0.15);
pub const NAV_ALERT: Color = Color::srgb(1.0, 0.30, 0.25);
/// A drifting course marker close to the centre band: still danger, nearly fixed.
pub const NAV_NEAR: Color = Color::srgb(1.0, 0.80, 0.25);
pub const ALERT: Color = Color::srgb(1.0, 0.22, 0.18);

// HUD and overlays.
pub const UI_TEXT: Color = SPECULAR;
pub const UI_DIM: Color = Color::srgba_u8(191, 208, 214, 140);
pub const UI_ACCENT: Color = VISOR_LIGHT;
pub const UI_PANEL: Color = Color::srgba_u8(16, 28, 41, 235);
pub const UI_TITLE: Color = VISOR_GLINT;
pub const MAP_HULL: Color = JOINT_SHADOW;
pub const MAP_ROOM: Color = STEEL_SHADOW;
pub const MAP_PLAYER: Color = VISOR_GLINT;
pub const DIAG_FRAME: Color = VISOR_MID;
/// A menu row, and the highlighted one. Nearly opaque: legible over the ship.
pub const UI_ROW: Color = Color::srgba_u8(27, 45, 62, 242);
pub const UI_ROW_FOCUS: Color = Color::srgba_u8(28, 88, 123, 242);

#[cfg(test)]
mod tests {
    use super::*;

    /// The pack's own rule (validate.cjs): red <= green <= blue.
    fn cold(c: Color) -> bool {
        let s = c.to_srgba();
        s.red <= s.green + 1e-6 && s.green <= s.blue + 1e-6
    }

    #[test]
    fn decoration_is_cold_and_danger_is_warm() {
        let decorative = [
            OUTLINE,
            DEEP_SEAM,
            JOINT_SHADOW,
            STEEL_SHADOW,
            STEEL,
            PLATE_MID,
            PLATE_LIGHT,
            WORN_EDGE,
            SPECULAR,
            VISOR_DEEP,
            VISOR_BLUE,
            VISOR_MID,
            VISOR_LIGHT,
            VISOR_GLINT,
            REPAIR_FABRIC,
            NAV_BAND,
            BREACH_AIR,
            UI_DIM,
            UI_PANEL,
            UI_ROW,
            UI_ROW_FOCUS,
        ];
        for c in decorative {
            assert!(cold(c), "{c:?} is not a cold colour");
        }
        for c in [ENGINE_HOT, BOLT_LOOSE, NAV_ALERT, NAV_NEAR, ALERT] {
            let s = c.to_srgba();
            assert!(s.red > s.blue + 0.3, "{c:?} is not a warm colour");
        }
    }
}
