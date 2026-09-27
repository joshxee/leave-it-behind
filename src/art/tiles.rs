//! Ship tiles: every cell of `ship-atlas.png` with its manifest data.

use bevy::prelude::*;

// Generated from assets/environment/derelict-ship/manifest.json.
// `tests::tiles_match_the_manifest` checks every value against the manifest.

/// Every tile in `ship-atlas.png`, in atlas order (the discriminant is the atlas index).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    FloorPanelA = 0,
    FloorPanelB = 1,
    FloorPanelC = 2,
    FloorGrate = 3,
    FloorConduitH = 4,
    FloorConduitV = 5,
    FloorHatch = 6,
    FloorReinforced = 7,
    FloorDrain = 8,
    FloorEdgeN = 9,
    FloorEdgeE = 10,
    FloorEdgeS = 11,
    FloorEdgeW = 12,
    FloorEdgeNe = 13,
    FloorEdgeSe = 14,
    FloorEdgeSw = 15,
    FloorEdgeNw = 16,
    Space0 = 17,
    Space1 = 18,
    WallPillar = 19,
    WallN = 20,
    WallE = 21,
    WallNe = 22,
    WallS = 23,
    WallNs = 24,
    WallEs = 25,
    WallNes = 26,
    WallW = 27,
    WallNw = 28,
    WallEw = 29,
    WallNew = 30,
    WallSw = 31,
    WallNsw = 32,
    WallEsw = 33,
    WallNesw = 34,
    WallBreachEw = 35,
    WallPatchedEw = 36,
    WallBreachNs = 37,
    WallPatchedNs = 38,
    DoorEw0 = 39,
    DoorEw1 = 40,
    DoorEw2 = 41,
    DoorEw3 = 42,
    DoorEwLocked = 43,
    DoorNs0 = 44,
    DoorNs1 = 45,
    DoorNs2 = 46,
    DoorNs3 = 47,
    DoorNsLocked = 48,
    Cockpit00 = 49,
    Cockpit10 = 50,
    Cockpit20 = 51,
    Cockpit01 = 52,
    Cockpit11 = 53,
    Cockpit21 = 54,
    Diagnostic0 = 55,
    Diagnostic1 = 56,
    Diagnostic2 = 57,
    PropPilotChair = 58,
    PropControlConsole = 59,
    PropLocker = 60,
    PropCrate = 61,
    PropOxygenRack = 62,
    PropPipeStack = 63,
}

impl Tile {
    pub const ALL: [Tile; 64] = [
        Tile::FloorPanelA,
        Tile::FloorPanelB,
        Tile::FloorPanelC,
        Tile::FloorGrate,
        Tile::FloorConduitH,
        Tile::FloorConduitV,
        Tile::FloorHatch,
        Tile::FloorReinforced,
        Tile::FloorDrain,
        Tile::FloorEdgeN,
        Tile::FloorEdgeE,
        Tile::FloorEdgeS,
        Tile::FloorEdgeW,
        Tile::FloorEdgeNe,
        Tile::FloorEdgeSe,
        Tile::FloorEdgeSw,
        Tile::FloorEdgeNw,
        Tile::Space0,
        Tile::Space1,
        Tile::WallPillar,
        Tile::WallN,
        Tile::WallE,
        Tile::WallNe,
        Tile::WallS,
        Tile::WallNs,
        Tile::WallEs,
        Tile::WallNes,
        Tile::WallW,
        Tile::WallNw,
        Tile::WallEw,
        Tile::WallNew,
        Tile::WallSw,
        Tile::WallNsw,
        Tile::WallEsw,
        Tile::WallNesw,
        Tile::WallBreachEw,
        Tile::WallPatchedEw,
        Tile::WallBreachNs,
        Tile::WallPatchedNs,
        Tile::DoorEw0,
        Tile::DoorEw1,
        Tile::DoorEw2,
        Tile::DoorEw3,
        Tile::DoorEwLocked,
        Tile::DoorNs0,
        Tile::DoorNs1,
        Tile::DoorNs2,
        Tile::DoorNs3,
        Tile::DoorNsLocked,
        Tile::Cockpit00,
        Tile::Cockpit10,
        Tile::Cockpit20,
        Tile::Cockpit01,
        Tile::Cockpit11,
        Tile::Cockpit21,
        Tile::Diagnostic0,
        Tile::Diagnostic1,
        Tile::Diagnostic2,
        Tile::PropPilotChair,
        Tile::PropControlConsole,
        Tile::PropLocker,
        Tile::PropCrate,
        Tile::PropOxygenRack,
        Tile::PropPipeStack,
    ];

    /// The tile's id in the manifest.
    pub fn id(self) -> &'static str {
        match self {
            Tile::FloorPanelA => "floor_panel_a",
            Tile::FloorPanelB => "floor_panel_b",
            Tile::FloorPanelC => "floor_panel_c",
            Tile::FloorGrate => "floor_grate",
            Tile::FloorConduitH => "floor_conduit_h",
            Tile::FloorConduitV => "floor_conduit_v",
            Tile::FloorHatch => "floor_hatch",
            Tile::FloorReinforced => "floor_reinforced",
            Tile::FloorDrain => "floor_drain",
            Tile::FloorEdgeN => "floor_edge_n",
            Tile::FloorEdgeE => "floor_edge_e",
            Tile::FloorEdgeS => "floor_edge_s",
            Tile::FloorEdgeW => "floor_edge_w",
            Tile::FloorEdgeNe => "floor_edge_ne",
            Tile::FloorEdgeSe => "floor_edge_se",
            Tile::FloorEdgeSw => "floor_edge_sw",
            Tile::FloorEdgeNw => "floor_edge_nw",
            Tile::Space0 => "space_0",
            Tile::Space1 => "space_1",
            Tile::WallPillar => "wall_pillar",
            Tile::WallN => "wall_n",
            Tile::WallE => "wall_e",
            Tile::WallNe => "wall_ne",
            Tile::WallS => "wall_s",
            Tile::WallNs => "wall_ns",
            Tile::WallEs => "wall_es",
            Tile::WallNes => "wall_nes",
            Tile::WallW => "wall_w",
            Tile::WallNw => "wall_nw",
            Tile::WallEw => "wall_ew",
            Tile::WallNew => "wall_new",
            Tile::WallSw => "wall_sw",
            Tile::WallNsw => "wall_nsw",
            Tile::WallEsw => "wall_esw",
            Tile::WallNesw => "wall_nesw",
            Tile::WallBreachEw => "wall_breach_ew",
            Tile::WallPatchedEw => "wall_patched_ew",
            Tile::WallBreachNs => "wall_breach_ns",
            Tile::WallPatchedNs => "wall_patched_ns",
            Tile::DoorEw0 => "door_ew_0",
            Tile::DoorEw1 => "door_ew_1",
            Tile::DoorEw2 => "door_ew_2",
            Tile::DoorEw3 => "door_ew_3",
            Tile::DoorEwLocked => "door_ew_locked",
            Tile::DoorNs0 => "door_ns_0",
            Tile::DoorNs1 => "door_ns_1",
            Tile::DoorNs2 => "door_ns_2",
            Tile::DoorNs3 => "door_ns_3",
            Tile::DoorNsLocked => "door_ns_locked",
            Tile::Cockpit00 => "cockpit_0_0",
            Tile::Cockpit10 => "cockpit_1_0",
            Tile::Cockpit20 => "cockpit_2_0",
            Tile::Cockpit01 => "cockpit_0_1",
            Tile::Cockpit11 => "cockpit_1_1",
            Tile::Cockpit21 => "cockpit_2_1",
            Tile::Diagnostic0 => "diagnostic_0",
            Tile::Diagnostic1 => "diagnostic_1",
            Tile::Diagnostic2 => "diagnostic_2",
            Tile::PropPilotChair => "prop_pilot_chair",
            Tile::PropControlConsole => "prop_control_console",
            Tile::PropLocker => "prop_locker",
            Tile::PropCrate => "prop_crate",
            Tile::PropOxygenRack => "prop_oxygen_rack",
            Tile::PropPipeStack => "prop_pipe_stack",
        }
    }

    /// Solid footprints as tile-local `[x, y, width, height]` pixels (top left origin).
    pub fn collision(self) -> &'static [[f32; 4]] {
        match self {
            Tile::FloorPanelA
            | Tile::FloorPanelB
            | Tile::FloorPanelC
            | Tile::FloorGrate
            | Tile::FloorConduitH
            | Tile::FloorConduitV
            | Tile::FloorHatch
            | Tile::FloorReinforced
            | Tile::FloorDrain
            | Tile::FloorEdgeN
            | Tile::FloorEdgeE
            | Tile::FloorEdgeS
            | Tile::FloorEdgeW
            | Tile::FloorEdgeNe
            | Tile::FloorEdgeSe
            | Tile::FloorEdgeSw
            | Tile::FloorEdgeNw
            | Tile::Space0
            | Tile::Space1 => &[],
            Tile::WallPillar => &[[20.0, 20.0, 24.0, 24.0]],
            Tile::WallN => &[[20.0, 20.0, 24.0, 24.0], [20.0, 0.0, 24.0, 20.0]],
            Tile::WallE => &[[20.0, 20.0, 24.0, 24.0], [44.0, 20.0, 20.0, 24.0]],
            Tile::WallNe => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [44.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallS => &[[20.0, 20.0, 24.0, 24.0], [20.0, 44.0, 24.0, 20.0]],
            Tile::WallNs => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [20.0, 44.0, 24.0, 20.0],
            ],
            Tile::WallEs => &[
                [20.0, 20.0, 24.0, 24.0],
                [44.0, 20.0, 20.0, 24.0],
                [20.0, 44.0, 24.0, 20.0],
            ],
            Tile::WallNes => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [44.0, 20.0, 20.0, 24.0],
                [20.0, 44.0, 24.0, 20.0],
            ],
            Tile::WallW => &[[20.0, 20.0, 24.0, 24.0], [0.0, 20.0, 20.0, 24.0]],
            Tile::WallNw => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallEw => &[
                [20.0, 20.0, 24.0, 24.0],
                [44.0, 20.0, 20.0, 24.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallNew => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [44.0, 20.0, 20.0, 24.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallSw => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 44.0, 24.0, 20.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallNsw => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [20.0, 44.0, 24.0, 20.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallEsw => &[
                [20.0, 20.0, 24.0, 24.0],
                [44.0, 20.0, 20.0, 24.0],
                [20.0, 44.0, 24.0, 20.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallNesw => &[
                [20.0, 20.0, 24.0, 24.0],
                [20.0, 0.0, 24.0, 20.0],
                [44.0, 20.0, 20.0, 24.0],
                [20.0, 44.0, 24.0, 20.0],
                [0.0, 20.0, 20.0, 24.0],
            ],
            Tile::WallBreachEw => &[[0.0, 20.0, 21.0, 24.0], [45.0, 20.0, 19.0, 24.0]],
            Tile::WallPatchedEw
            | Tile::DoorEw0
            | Tile::DoorEw1
            | Tile::DoorEw2
            | Tile::DoorEwLocked => &[[0.0, 20.0, 64.0, 24.0]],
            Tile::WallBreachNs => &[[20.0, 0.0, 24.0, 21.0], [20.0, 45.0, 24.0, 19.0]],
            Tile::WallPatchedNs
            | Tile::DoorNs0
            | Tile::DoorNs1
            | Tile::DoorNs2
            | Tile::DoorNsLocked => &[[20.0, 0.0, 24.0, 64.0]],
            Tile::DoorEw3 => &[[0.0, 20.0, 14.0, 24.0], [50.0, 20.0, 14.0, 24.0]],
            Tile::DoorNs3 => &[[20.0, 0.0, 24.0, 14.0], [20.0, 50.0, 24.0, 14.0]],
            Tile::Cockpit00 | Tile::Cockpit10 | Tile::Cockpit20 => &[[0.0, 8.0, 64.0, 50.0]],
            Tile::Cockpit01 | Tile::Cockpit21 => &[[8.0, 1.0, 48.0, 38.0]],
            Tile::Cockpit11 => &[[0.0, 0.0, 64.0, 27.0], [18.0, 30.0, 30.0, 34.0]],
            Tile::Diagnostic0 | Tile::Diagnostic1 | Tile::Diagnostic2 => {
                &[[10.0, 16.0, 45.0, 43.0]]
            }
            Tile::PropPilotChair => &[[17.0, 20.0, 31.0, 31.0]],
            Tile::PropControlConsole => &[[6.0, 17.0, 52.0, 36.0]],
            Tile::PropLocker => &[[13.0, 7.0, 38.0, 50.0]],
            Tile::PropCrate => &[[10.0, 14.0, 44.0, 39.0]],
            Tile::PropOxygenRack => &[[10.0, 10.0, 45.0, 47.0]],
            Tile::PropPipeStack => &[[10.0, 9.0, 44.0, 47.0]],
        }
    }

    /// Where a player interacts with it, in tile-local pixels.
    pub fn interaction(self) -> Option<[f32; 2]> {
        match self {
            Tile::Diagnostic0 | Tile::Diagnostic1 | Tile::Diagnostic2 => Some([32.0, 59.0]),
            Tile::PropControlConsole => Some([32.0, 53.0]),
            _ => None,
        }
    }
}

impl Tile {
    /// Index in `ship-atlas.png` (8 columns of 64-pixel cells).
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The connecting wall for a neighbour mask (N=1, E=2, S=4, W=8).
    pub fn wall(mask: u8) -> Tile {
        Tile::ALL[Tile::WallPillar.index() + (mask & 15) as usize]
    }

    /// Door frame `frame` (0 closed .. 3 open) in a wall running east-west
    /// (`across_x`: people pass north-south) or north-south.
    pub fn door(across_x: bool, frame: usize) -> Tile {
        let first = if across_x {
            Tile::DoorEw0
        } else {
            Tile::DoorNs0
        };
        Tile::ALL[first.index() + frame.min(3)]
    }

    pub fn locked_door(across_x: bool) -> Tile {
        if across_x {
            Tile::DoorEwLocked
        } else {
            Tile::DoorNsLocked
        }
    }

    /// Breached and patched versions of a straight wall.
    pub fn breach(across_x: bool) -> Tile {
        if across_x {
            Tile::WallBreachEw
        } else {
            Tile::WallBreachNs
        }
    }

    pub fn patched(across_x: bool) -> Tile {
        if across_x {
            Tile::WallPatchedEw
        } else {
            Tile::WallPatchedNs
        }
    }

    /// Cockpit module cell (`col` 0..3 left to right, `row` 0 = window row).
    pub fn cockpit(col: usize, row: usize) -> Tile {
        let first = if row == 0 {
            Tile::Cockpit00
        } else {
            Tile::Cockpit01
        };
        Tile::ALL[first.index() + col.min(2)]
    }

    /// The diagnostic screen: `Some(frame)` live (0 or 1), `None` powered off.
    pub fn diagnostic(live: Option<usize>) -> Tile {
        match live {
            Some(frame) => Tile::ALL[Tile::Diagnostic0.index() + frame % 2],
            None => Tile::Diagnostic2,
        }
    }
}

/// A tile-local rectangle (`[x, y, w, h]`, pixels, y down) in world space for
/// a cell centred at `center`.
pub fn local_rect(center: Vec2, [x, y, w, h]: [f32; 4]) -> Rect {
    Rect::from_center_size(
        center + Vec2::new(x + w / 2.0 - 32.0, 32.0 - y - h / 2.0),
        Vec2::new(w, h),
    )
}

/// A tile-local point (pixels, y down) in world space.
pub fn local_point(center: Vec2, [x, y]: [f32; 2]) -> Vec2 {
    center + Vec2::new(x - 32.0, 32.0 - y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_match_the_manifest() {
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../assets/environment/derelict-ship/manifest.json"
        ))
        .unwrap();
        let tiles = manifest["tiles"].as_array().unwrap();
        assert_eq!(tiles.len(), Tile::ALL.len());
        for t in tiles {
            let index = t["atlas_index"].as_u64().unwrap() as usize;
            let tile = Tile::ALL[index];
            assert_eq!(tile.index(), index);
            assert_eq!(tile.id(), t["id"].as_str().unwrap());
            let collision: Vec<[f32; 4]> = t["collision"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    let v: Vec<f32> = r
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|n| n.as_f64().unwrap() as f32)
                        .collect();
                    [v[0], v[1], v[2], v[3]]
                })
                .collect();
            assert_eq!(tile.collision(), collision.as_slice(), "{}", tile.id());
            let interaction = t.get("interaction_pixels").map(|p| {
                let p = p.as_array().unwrap();
                [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32]
            });
            assert_eq!(tile.interaction(), interaction, "{}", tile.id());
            if let Some(mask) = t.get("connection_mask").and_then(|m| m.as_u64())
                && t["category"] == "wall"
            {
                assert_eq!(Tile::wall(mask as u8), tile);
            }
        }
    }

    #[test]
    fn variants_pick_the_right_cells() {
        assert_eq!(Tile::wall(10), Tile::WallEw);
        assert_eq!(Tile::wall(5), Tile::WallNs);
        assert_eq!(Tile::door(true, 3), Tile::DoorEw3);
        assert_eq!(Tile::door(false, 9), Tile::DoorNs3);
        assert_eq!(Tile::cockpit(2, 1), Tile::Cockpit21);
        assert_eq!(Tile::diagnostic(Some(3)), Tile::Diagnostic1);
        assert_eq!(Tile::diagnostic(None), Tile::Diagnostic2);
    }

    #[test]
    fn local_coordinates_flip_y() {
        let r = local_rect(Vec2::ZERO, [20.0, 0.0, 24.0, 20.0]);
        assert_eq!(r, Rect::new(-12.0, 12.0, 12.0, 32.0));
        assert_eq!(local_point(Vec2::ZERO, [32.0, 59.0]), Vec2::new(0.0, -27.0));
    }
}
