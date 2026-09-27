//! The ship as data. [`SHIP`] is a grid of 64-pixel cells, one character per
//! cell, drawn as it appears on screen. [`ShipMap`] parses it once (see
//! [`ship`]) and derives everything the game needs from it: rooms, walls and
//! doors, what is solid, floor art, and named points (spawn, helm, console,
//! breach points, engine blocks).
//!
//! Nothing else hard-codes a position or which way the ship points. To change
//! the layout, or turn the ship, edit `SHIP`: [`ShipMap::parse`] rejects a
//! layout that breaks the rules below, and the unit tests check the rest
//! (rooms fit the view, every fault site can be reached, ...).
//!
//! Legend:
//!
//! ```text
//! ' '  space                     '#'  wall (joins its neighbours)
//! '.'  floor                     '|' '-'  floor with a conduit (vertical, horizontal)
//! '='  door, slides open as the engineer comes near
//! 'X'  locked door (the airlock's outer hatch)
//! 'C' 'Q' 'E' 'H' 'A'  floor that names its room: cockpit, engineer's
//!      quarters, engine room, main hull, airlock (exactly one each)
//! 'V'  cockpit module, window row (part of the hull wall; the art faces up)
//! 'v'  cockpit module, console row (the middle cell is the pilot seat)
//! 'N'  nav display (on the floor)    'd'  diagnostic console (faces down)
//! 'P' 'S'  port / starboard engine block    'b'  bunk
//! 'c' crate  'l' locker  'o' oxygen rack  'p' pipe stack  'k' control console
//! '@'  where the engineer starts
//! '1'..'9'  hull wall with a breach point (`faults::sites` names them)
//! ```
//!
//! Rules: every floor cell is in exactly one room (rooms are walled off from
//! each other and from space); a door joins two rooms and sits in a straight
//! wall; a locked door and every breach point sit in a straight hull wall
//! (a room on one side, space on the other).

use std::collections::VecDeque;
use std::sync::LazyLock;

use bevy::prelude::*;

use crate::art::TILE;
use crate::art::tiles::{Tile, local_point, local_rect};

/// The ship, pointing up: cockpit at the top, airlock at the bottom. The
/// spine (the conduit through every door) runs from front to tail.
pub const SHIP: &str = r"
   #####VVV#####
   #Ck..vvv..k.#
   #.....|.....#
   #....NNN....#
   #....NNN....#
   #....NNN....#
   #.....|.....#
   #o....|....o#
  #######=#######
  #Q..d..|....ll#
  #......|......#
  #.....@|......#
  #......|......#
  #......|......#
  #......|......#
  #......|......#
  #bb....|....pp#
#########=#########
#E.......|......pp#
#...PP...|...SS...#
#...PP...|...SS...#
#...PP...|...SS...#
#...PP...|...SS...#
#...PP...|...SS...#
#...PP...|...SS...#
#........|........#
#########=#########
#H.......|........#
9...cc...|........#
#...cc...|........#
#........|........8
#........|........#
#........|..ccc...#
7........|..cc....#
#o.......|.......o#
#########=#########
   #A....|.....#
   4.....|.....6
   #.....|.....#
   #l....|....l#
   #.....|.....#
   3.....|.....5
   #.....|.....#
   ####1#X#2####
";

/// Half the thickness of a wall's solid core (the tiles' 24-pixel footprint).
pub const WALL_HALF: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomId {
    Cockpit,
    Quarters,
    Engine,
    Hull,
    Airlock,
}

impl RoomId {
    /// Front to tail.
    pub const ALL: [RoomId; 5] = [
        RoomId::Cockpit,
        RoomId::Quarters,
        RoomId::Engine,
        RoomId::Hull,
        RoomId::Airlock,
    ];

    /// Position in [`RoomId::ALL`] (0 = cockpit).
    pub fn index(self) -> usize {
        self as usize
    }

    /// Stable identifier (test bridge, logs).
    pub fn as_str(self) -> &'static str {
        match self {
            RoomId::Cockpit => "Cockpit",
            RoomId::Quarters => "Quarters",
            RoomId::Engine => "Engine",
            RoomId::Hull => "Hull",
            RoomId::Airlock => "Airlock",
        }
    }

    /// Player-facing name.
    pub fn name(self) -> &'static str {
        match self {
            RoomId::Cockpit => "Cockpit",
            RoomId::Quarters => "Engineer's quarters",
            RoomId::Engine => "Engine room",
            RoomId::Hull => "Main hull",
            RoomId::Airlock => "Airlock",
        }
    }

    /// The map character that names this room.
    pub fn marker(self) -> char {
        match self {
            RoomId::Cockpit => 'C',
            RoomId::Quarters => 'Q',
            RoomId::Engine => 'E',
            RoomId::Hull => 'H',
            RoomId::Airlock => 'A',
        }
    }

    pub fn from_marker(c: char) -> Option<RoomId> {
        RoomId::ALL.into_iter().find(|r| r.marker() == c)
    }

    /// Walkable interior, up to the wall faces.
    pub fn interior(self) -> Rect {
        ship().interior(self)
    }

    /// Where the camera looks while the player is in this room.
    pub fn center(self) -> Vec2 {
        self.interior().center()
    }

    /// The room owning world point `p`. Rooms meet at the middle of the
    /// walls between them.
    pub fn at(p: Vec2) -> RoomId {
        ship().room_at(p)
    }
}

/// What a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Space,
    /// Walls, doors and the cockpit's window row.
    Structure,
    /// Anything standing on the floor of a room.
    Floor,
}

pub fn kind(c: char) -> Option<Kind> {
    match c {
        ' ' => Some(Kind::Space),
        '#' | '=' | 'X' | 'V' | '1'..='9' => Some(Kind::Structure),
        '.' | '|' | '-' | '@' | 'N' | 'v' | 'P' | 'S' | 'b' | 'c' | 'l' | 'o' | 'p' | 'k' | 'd' => {
            Some(Kind::Floor)
        }
        c if RoomId::from_marker(c).is_some() => Some(Kind::Floor),
        _ => None,
    }
}

/// Grid directions (rows grow downward) with their wall-mask bits.
const SIDES: [(u8, IVec2); 4] = [
    (1, IVec2::new(0, -1)),
    (2, IVec2::new(1, 0)),
    (4, IVec2::new(0, 1)),
    (8, IVec2::new(-1, 0)),
];

/// A door in the map.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoorSpec {
    pub cell: IVec2,
    pub center: Vec2,
    /// The wall runs east-west (people pass north-south).
    pub across_x: bool,
    pub locked: bool,
}

/// A marked point on a hull wall's inner face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallMark {
    pub cell: IVec2,
    /// On the wall face, in the middle of the cell.
    pub point: Vec2,
    /// Out of the wall, into the room.
    pub normal: Vec2,
    /// The wall runs east-west.
    pub across_x: bool,
}

/// The pilot seat and its controls.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Helm {
    pub seat: Vec2,
    pub joystick: Vec2,
    /// From the seat toward the window.
    pub facing: Vec2,
}

#[derive(Debug, Clone)]
pub struct ShipMap {
    /// Columns and rows, including a margin of space all round.
    size: IVec2,
    cells: Vec<char>,
    rooms: Vec<Option<RoomId>>,
    interiors: [Option<Rect>; 5],
    /// Per cell, which quadrant corners (`[nw, ne, sw, se]`) are outside the hull.
    outside: Vec<[bool; 4]>,
    walls: Vec<Rect>,
    props: Vec<Rect>,
}

static SHIP_MAP: LazyLock<ShipMap> =
    LazyLock::new(|| ShipMap::parse(SHIP).unwrap_or_else(|e| panic!("invalid ship map: {e}")));

/// The game's ship, parsed from [`SHIP`].
pub fn ship() -> &'static ShipMap {
    &SHIP_MAP
}

impl ShipMap {
    pub fn parse(src: &str) -> Result<ShipMap, String> {
        let lines: Vec<&str> = src.lines().collect();
        let first = lines
            .iter()
            .position(|l| !l.trim().is_empty())
            .ok_or("the map is empty")?;
        let last = lines
            .iter()
            .rposition(|l| !l.trim().is_empty())
            .unwrap_or(first);
        let lines = &lines[first..=last];
        let width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i32 + 2;
        let height = lines.len() as i32 + 2;
        let n = (width * height) as usize;
        let mut cells = vec![' '; n];
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if kind(ch).is_none() {
                    return Err(format!(
                        "unknown character {ch:?} at row {row}, column {col}"
                    ));
                }
                cells[(row + 1) * width as usize + col + 1] = ch;
            }
        }
        let mut map = ShipMap {
            size: IVec2::new(width, height),
            cells,
            rooms: vec![None; n],
            interiors: [None; 5],
            outside: vec![[false; 4]; n],
            walls: Vec::new(),
            props: Vec::new(),
        };
        map.find_rooms()?;
        map.check_structure()?;
        map.outside = map.flood_outside();
        map.walls = merge(map.solid_rects(false));
        map.props = merge(map.solid_rects(true));
        Ok(map)
    }

    // ---- cells ----------------------------------------------------------

    fn slot(&self, cell: IVec2) -> Option<usize> {
        (cell.x >= 0 && cell.y >= 0 && cell.x < self.size.x && cell.y < self.size.y)
            .then(|| (cell.y * self.size.x + cell.x) as usize)
    }

    /// The character at `cell` (space outside the grid).
    pub fn get(&self, cell: IVec2) -> char {
        self.slot(cell).map_or(' ', |i| self.cells[i])
    }

    pub fn kind_at(&self, cell: IVec2) -> Kind {
        kind(self.get(cell)).unwrap_or(Kind::Space)
    }

    /// Every cell and its character, row by row.
    pub fn cells(&self) -> impl Iterator<Item = (IVec2, char)> + '_ {
        (0..self.size.y).flat_map(move |y| {
            (0..self.size.x).map(move |x| {
                let cell = IVec2::new(x, y);
                (cell, self.get(cell))
            })
        })
    }

    pub fn cells_of(&self, ch: char) -> impl Iterator<Item = IVec2> + '_ {
        self.cells()
            .filter(move |&(_, c)| c == ch)
            .map(|(cell, _)| cell)
    }

    /// World position of a cell's centre. The grid is centred on the origin.
    pub fn center(&self, cell: IVec2) -> Vec2 {
        Vec2::new(
            (cell.x as f32 + 0.5) * TILE - self.size.x as f32 * TILE / 2.0,
            self.size.y as f32 * TILE / 2.0 - (cell.y as f32 + 0.5) * TILE,
        )
    }

    /// The cell containing world point `p`.
    pub fn cell_at(&self, p: Vec2) -> IVec2 {
        IVec2::new(
            ((p.x + self.size.x as f32 * TILE / 2.0) / TILE).floor() as i32,
            ((self.size.y as f32 * TILE / 2.0 - p.y) / TILE).floor() as i32,
        )
    }

    /// Union of the squares of every cell marked `ch` (a block of cells).
    pub fn block(&self, ch: char) -> Option<Rect> {
        self.cells_of(ch)
            .map(|c| Rect::from_center_size(self.center(c), Vec2::splat(TILE)))
            .reduce(|a, b| a.union(b))
    }

    fn only(&self, ch: char) -> IVec2 {
        self.cells_of(ch)
            .next()
            .unwrap_or_else(|| panic!("the map has no {ch:?}"))
    }

    /// Structure neighbours of a cell as a mask (N=1, E=2, S=4, W=8).
    pub fn mask(&self, cell: IVec2) -> u8 {
        SIDES
            .iter()
            .filter(|(_, d)| self.kind_at(cell + *d) == Kind::Structure)
            .fold(0, |m, (bit, _)| m | bit)
    }

    /// For a straight wall (or door): whether it runs east-west.
    pub fn across_x(&self, cell: IVec2) -> bool {
        self.mask(cell) & 10 == 10
    }

    /// Position of a cockpit module cell in its row, 0 to 2 from the left.
    pub fn module_column(&self, cell: IVec2) -> usize {
        let ch = self.get(cell);
        (1..=2)
            .take_while(|&k| self.get(cell - IVec2::new(k, 0)) == ch)
            .count()
    }

    // ---- rooms ----------------------------------------------------------

    pub fn room_of(&self, cell: IVec2) -> Option<RoomId> {
        self.slot(cell).and_then(|i| self.rooms[i])
    }

    /// Walkable interior of a room, up to the wall faces.
    pub fn interior(&self, room: RoomId) -> Rect {
        self.interiors[room.index()].unwrap_or_else(|| panic!("the map has no {}", room.name()))
    }

    /// What the camera shows of a room: its floor and the solid core of its
    /// walls (not the neighbours beyond them), plus structure art that
    /// belongs to the room alone: the cockpit's window row.
    pub fn frame(&self, room: RoomId) -> Rect {
        let mut frame = self.interior(room).inflate(2.0 * WALL_HALF);
        for cell in self.cells_of('V') {
            if SIDES
                .iter()
                .any(|(_, d)| self.room_of(cell + *d) == Some(room))
            {
                frame = frame.union(Rect::from_center_size(self.center(cell), Vec2::splat(TILE)));
            }
        }
        frame
    }

    /// Rooms present in the map.
    pub fn rooms(&self) -> impl Iterator<Item = RoomId> + '_ {
        RoomId::ALL
            .into_iter()
            .filter(|r| self.interiors[r.index()].is_some())
    }

    /// The room owning world point `p`: the room of its cell, or in a wall
    /// or door the neighbouring room cell nearest `p` (so rooms meet at the
    /// wall's centre line), or else the nearest room.
    pub fn room_at(&self, p: Vec2) -> RoomId {
        let cell = self.cell_at(p);
        if let Some(room) = self.room_of(cell) {
            return room;
        }
        let mut best: Option<(f32, RoomId)> = None;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let n = cell + IVec2::new(dx, dy);
                if let Some(room) = self.room_of(n) {
                    let d = self.center(n).distance_squared(p);
                    if best.is_none_or(|(b, _)| d < b) {
                        best = Some((d, room));
                    }
                }
            }
        }
        if let Some((_, room)) = best {
            return room;
        }
        let gap = |r: RoomId| {
            let i = self.interior(r);
            p.clamp(i.min, i.max).distance_squared(p)
        };
        self.rooms()
            .min_by(|a, b| gap(*a).total_cmp(&gap(*b)))
            .expect("a map has at least one room")
    }

    fn find_rooms(&mut self) -> Result<(), String> {
        for room in RoomId::ALL {
            let marks: Vec<IVec2> = self.cells_of(room.marker()).collect();
            let start = match marks.as_slice() {
                [cell] => *cell,
                [] => continue,
                _ => return Err(format!("more than one {:?} marker", room.marker())),
            };
            let mut queue = VecDeque::from([start]);
            let mut bounds = Rect::EMPTY;
            while let Some(cell) = queue.pop_front() {
                let i = self.slot(cell).expect("inside the grid");
                match self.rooms[i] {
                    Some(r) if r == room => continue,
                    Some(other) => {
                        return Err(format!(
                            "the {} and the {} are not walled off",
                            other.name(),
                            room.name()
                        ));
                    }
                    None => {}
                }
                self.rooms[i] = Some(room);
                bounds = bounds.union(Rect::from_center_size(self.center(cell), Vec2::splat(TILE)));
                for (_, d) in SIDES {
                    let next = cell + d;
                    match self.kind_at(next) {
                        Kind::Floor => queue.push_back(next),
                        Kind::Space => {
                            return Err(format!("the {} is open to space at {next}", room.name()));
                        }
                        Kind::Structure => {}
                    }
                }
            }
            self.interiors[room.index()] = Some(bounds.inflate(TILE / 2.0 - WALL_HALF));
        }
        let unassigned = self
            .cells()
            .find(|&(cell, c)| kind(c) == Some(Kind::Floor) && self.room_of(cell).is_none());
        if let Some((cell, c)) = unassigned {
            return Err(format!(
                "{c:?} at {cell} is in no room (no room marker reaches it)"
            ));
        }
        Ok(())
    }

    fn check_structure(&self) -> Result<(), String> {
        let side = |cell: IVec2, d: IVec2| (self.kind_at(cell + d), self.room_of(cell + d));
        for (cell, c) in self.cells() {
            let straight = match self.mask(cell) {
                10 => Some((IVec2::new(0, -1), IVec2::new(0, 1))),
                5 => Some((IVec2::new(-1, 0), IVec2::new(1, 0))),
                _ => None,
            };
            match c {
                '=' => {
                    let Some((a, b)) = straight else {
                        return Err(format!("the door at {cell} is not in a straight wall"));
                    };
                    match (side(cell, a), side(cell, b)) {
                        ((Kind::Floor, Some(x)), (Kind::Floor, Some(y))) if x != y => {}
                        _ => return Err(format!("the door at {cell} does not join two rooms")),
                    }
                }
                'X' | '1'..='9' => {
                    let Some((a, b)) = straight else {
                        return Err(format!("{c:?} at {cell} is not in a straight wall"));
                    };
                    let (ka, kb) = (side(cell, a).0, side(cell, b).0);
                    if !matches!(
                        (ka, kb),
                        (Kind::Floor, Kind::Space) | (Kind::Space, Kind::Floor)
                    ) {
                        return Err(format!("{c:?} at {cell} is not in a hull wall"));
                    }
                }
                'V' => {
                    let below = cell + IVec2::new(0, 1);
                    if self.get(below) != 'v' {
                        return Err(format!(
                            "the cockpit window at {cell} has no console row below"
                        ));
                    }
                }
                _ => {}
            }
        }
        for ch in ['V', 'v'] {
            let cells: Vec<IVec2> = self.cells_of(ch).collect();
            if cells.is_empty() {
                continue;
            }
            let row = cells[0].y;
            let x0 = cells.iter().map(|c| c.x).min().unwrap_or(0);
            let run = (0..3).map(|k| IVec2::new(x0 + k, row)).collect::<Vec<_>>();
            if cells != run {
                return Err(format!(
                    "the cockpit module needs three {ch:?} cells in a row"
                ));
            }
        }
        for (ch, what) in [('@', "spawn point"), ('d', "diagnostic console")] {
            if self.cells_of(ch).count() > 1 {
                return Err(format!("the map has more than one {what} ({ch:?})"));
            }
        }
        for (ch, what) in [
            ('P', "port engine"),
            ('S', "starboard engine"),
            ('N', "nav display"),
            ('b', "bunk"),
        ] {
            let Some(block) = self.block(ch) else {
                continue;
            };
            let area = (block.width() / TILE).round() * (block.height() / TILE).round();
            if self.cells_of(ch).count() as f32 != area {
                return Err(format!("the {what} ({ch:?}) is not a filled rectangle"));
            }
        }
        for digit in '1'..='9' {
            if self.cells_of(digit).count() > 1 {
                return Err(format!("breach point {digit:?} is marked more than once"));
            }
        }
        Ok(())
    }

    // ---- hull outline ---------------------------------------------------

    /// Flood fill "outside" from space over a 3x3 block grid per cell. The
    /// centre block of a structure cell is solid, and so is each edge block
    /// toward a structure neighbour (the wall's arms). Corner blocks are
    /// never solid; the fill never enters a room cell.
    fn flood_outside(&self) -> Vec<[bool; 4]> {
        const B: i32 = 3;
        let blocks = self.size * B;
        let index = |b: IVec2| (b.y * blocks.x + b.x) as usize;
        let mut solid = vec![false; (blocks.x * blocks.y) as usize];
        let mut seen = vec![false; solid.len()];
        let mut queue = VecDeque::new();
        for (cell, c) in self.cells() {
            let origin = cell * B;
            match kind(c) {
                Some(Kind::Structure) => {
                    solid[index(origin + IVec2::ONE)] = true;
                    let mask = self.mask(cell);
                    for (bit, d) in SIDES {
                        if mask & bit != 0 || c == 'V' {
                            solid[index(origin + IVec2::ONE + d)] = true;
                        }
                    }
                }
                Some(Kind::Space) | None => {
                    for dy in 0..B {
                        for dx in 0..B {
                            let b = origin + IVec2::new(dx, dy);
                            seen[index(b)] = true;
                            queue.push_back(b);
                        }
                    }
                }
                Some(Kind::Floor) => {}
            }
        }
        while let Some(b) = queue.pop_front() {
            for (_, d) in SIDES {
                let n = b + d;
                if n.x < 0 || n.y < 0 || n.x >= blocks.x || n.y >= blocks.y {
                    continue;
                }
                let i = index(n);
                if seen[i] || solid[i] || self.kind_at(n / B) == Kind::Floor {
                    continue;
                }
                seen[i] = true;
                queue.push_back(n);
            }
        }
        self.cells()
            .map(|(cell, _)| {
                let o = cell * B;
                [(0, 0), (2, 0), (0, 2), (2, 2)].map(|(x, y)| seen[index(o + IVec2::new(x, y))])
            })
            .collect()
    }

    /// Which quadrants (`[nw, ne, sw, se]`) of a cell have floor under them.
    /// Room cells: all four. Walls: the quadrants inside the hull, so the
    /// hull outline is exact even at inner corners. Space: none.
    pub fn floor_quadrants(&self, cell: IVec2) -> [bool; 4] {
        match self.kind_at(cell) {
            Kind::Floor => [true; 4],
            Kind::Space => [false; 4],
            Kind::Structure => {
                let outside = self.slot(cell).map_or([true; 4], |i| self.outside[i]);
                outside.map(|o| !o)
            }
        }
    }

    /// Floor art for a cell: the conduit where marked, otherwise a pattern
    /// that depends on the room.
    pub fn floor_tile(&self, cell: IVec2) -> Tile {
        let h = ((cell.x as u64).wrapping_mul(73_856_093)
            ^ (cell.y as u64).wrapping_mul(19_349_663))
            % 10;
        match (self.get(cell), self.room_of(cell)) {
            ('|', _) => Tile::FloorConduitV,
            ('-', _) => Tile::FloorConduitH,
            (_, Some(RoomId::Engine)) => match h {
                0 | 1 => Tile::FloorGrate,
                2 => Tile::FloorPanelB,
                _ => Tile::FloorPanelA,
            },
            (_, Some(RoomId::Hull)) => match h {
                0 | 1 => Tile::FloorReinforced,
                2 => Tile::FloorPanelC,
                _ => Tile::FloorPanelA,
            },
            (_, Some(RoomId::Airlock)) => match h {
                0 => Tile::FloorReinforced,
                1 => Tile::FloorDrain,
                2 => Tile::FloorPanelB,
                _ => Tile::FloorPanelA,
            },
            _ => match h {
                0..=5 => Tile::FloorPanelA,
                6 | 7 => Tile::FloorPanelB,
                _ => Tile::FloorPanelC,
            },
        }
    }

    // ---- solid things ---------------------------------------------------

    /// The tile drawn for a structure cell at the start of a run (walls
    /// intact, doors closed).
    pub fn structure_tile(&self, cell: IVec2) -> Option<Tile> {
        let across = self.across_x(cell);
        match self.get(cell) {
            '#' | '1'..='9' => Some(Tile::wall(self.mask(cell))),
            '=' => Some(Tile::door(across, 0)),
            'X' => Some(Tile::locked_door(across)),
            'V' => Some(Tile::cockpit(self.module_column(cell), 0)),
            _ => None,
        }
    }

    /// Static solid rectangles: walls (`props` false: walls, locked doors,
    /// door jambs, the cockpit window row) or props (consoles, cargo,
    /// engines, the bunk). Door leaves move, so `doors` handles them.
    fn solid_rects(&self, props: bool) -> Vec<Rect> {
        const NONE: &[[f32; 4]] = &[];
        let mut out = Vec::new();
        for (cell, c) in self.cells() {
            let center = self.center(cell);
            let tile_rects: &[[f32; 4]] = match (props, c) {
                (false, '=') => Tile::door(self.across_x(cell), 3).collision(),
                (false, _) if self.kind_at(cell) == Kind::Structure => {
                    self.structure_tile(cell).map_or(NONE, Tile::collision)
                }
                // The pilot seat is where the engineer sits: only its console is solid.
                (true, 'v') => {
                    let rects = Tile::cockpit(self.module_column(cell), 1).collision();
                    if self.module_column(cell) == 1 {
                        &rects[..1]
                    } else {
                        rects
                    }
                }
                (true, 'd') => Tile::Diagnostic0.collision(),
                (true, 'c') => Tile::PropCrate.collision(),
                (true, 'l') => Tile::PropLocker.collision(),
                (true, 'o') => Tile::PropOxygenRack.collision(),
                (true, 'p') => Tile::PropPipeStack.collision(),
                (true, 'k') => Tile::PropControlConsole.collision(),
                (true, 'P' | 'S' | 'b') => {
                    out.push(Rect::from_center_size(center, Vec2::splat(TILE)));
                    NONE
                }
                _ => NONE,
            };
            out.extend(tile_rects.iter().map(|r| local_rect(center, *r)));
        }
        out
    }

    /// Walls, locked doors, door jambs and the cockpit window row, merged
    /// into as few rectangles as possible. Tape sticks to these.
    pub fn walls(&self) -> &[Rect] {
        &self.walls
    }

    /// Solid props, merged.
    pub fn props(&self) -> &[Rect] {
        &self.props
    }

    // ---- named things ---------------------------------------------------

    pub fn doors(&self) -> Vec<DoorSpec> {
        self.cells()
            .filter(|&(_, c)| c == '=' || c == 'X')
            .map(|(cell, c)| DoorSpec {
                cell,
                center: self.center(cell),
                across_x: self.across_x(cell),
                locked: c == 'X',
            })
            .collect()
    }

    /// The point on the wall face at a breach mark (`'1'..='9'`).
    pub fn wall_mark(&self, mark: char) -> Option<WallMark> {
        let cell = self.cells_of(mark).next()?;
        let normal = SIDES
            .iter()
            .find(|(_, d)| self.kind_at(cell + *d) == Kind::Floor)
            .map(|(_, d)| Vec2::new(d.x as f32, -d.y as f32))?;
        Some(WallMark {
            cell,
            point: self.center(cell) + normal * WALL_HALF,
            normal,
            across_x: self.across_x(cell),
        })
    }

    pub fn spawn(&self) -> Vec2 {
        self.center(self.only('@'))
    }

    /// The pilot seat (the middle of the cockpit's console row).
    pub fn helm(&self) -> Helm {
        let seat_cell = self
            .cells_of('v')
            .find(|&c| self.module_column(c) == 1)
            .expect("a cockpit module");
        let window = self.center(seat_cell - IVec2::new(0, 1));
        let facing = (window - self.center(seat_cell)).normalize();
        // The chair's footprint in the tile, and a stick on the console in front.
        let seat = local_point(self.center(seat_cell), [33.0, 47.0]);
        Helm {
            seat,
            joystick: seat + facing * 30.0,
            facing,
        }
    }

    /// The diagnostic console's cell.
    pub fn console_cell(&self) -> IVec2 {
        self.only('d')
    }

    /// Where the engineer uses the diagnostic console (its front edge).
    pub fn console_point(&self) -> Vec2 {
        let interaction = Tile::Diagnostic0
            .interaction()
            .expect("an interaction point");
        local_point(self.center(self.console_cell()), interaction)
    }
}

/// Merges rectangles that touch along a full edge (runs of wall cells
/// become one long rectangle), so a circle slides along them without
/// catching on seams.
pub fn merge(mut rects: Vec<Rect>) -> Vec<Rect> {
    loop {
        let before = rects.len();
        rects = merge_along(rects, true);
        rects = merge_along(rects, false);
        if rects.len() == before {
            return rects;
        }
    }
}

fn merge_along(mut rects: Vec<Rect>, along_x: bool) -> Vec<Rect> {
    // (cross min, cross max, along min, along max)
    let key = |r: &Rect| {
        if along_x {
            (r.min.y, r.max.y, r.min.x, r.max.x)
        } else {
            (r.min.x, r.max.x, r.min.y, r.max.y)
        }
    };
    rects.sort_by(|a, b| {
        let (a, b) = (key(a), key(b));
        a.0.total_cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2.total_cmp(&b.2))
    });
    let mut out: Vec<Rect> = Vec::with_capacity(rects.len());
    for r in rects {
        if let Some(last) = out.last_mut() {
            let (l, k) = (key(last), key(&r));
            if l.0 == k.0 && l.1 == k.1 && k.2 <= l.3 {
                if along_x {
                    last.max.x = last.max.x.max(r.max.x);
                } else {
                    last.max.y = last.max.y.max(r.max.y);
                }
                continue;
            }
        }
        out.push(r);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::layout::VIEW;

    /// A small ship pointing right: proves nothing depends on orientation.
    const SIDEWAYS: &str = r"
#######
#C....####
#...@.=.Q#
#.d...####
#######
";

    #[test]
    fn the_ship_parses_and_has_everything_the_game_uses() {
        let map = ShipMap::parse(SHIP).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(map.rooms().count(), RoomId::ALL.len());
        assert_eq!(map.doors().len(), 5);
        assert_eq!(map.doors().iter().filter(|d| d.locked).count(), 1);
        for ch in ['@', 'd', 'V', 'v', 'N', 'P', 'S', 'b'] {
            assert!(map.cells_of(ch).next().is_some(), "no {ch:?}");
        }
        assert_eq!(map.cells_of('@').count(), 1);
    }

    #[test]
    fn every_room_fits_the_view_with_its_walls() {
        let map = ship();
        for room in RoomId::ALL {
            let frame = map.frame(room);
            assert!(
                frame.width() <= VIEW.x && frame.height() <= VIEW.y,
                "{room:?} is {}x{}",
                frame.width(),
                frame.height()
            );
            assert!(frame.contains(room.interior().min) && frame.contains(room.interior().max));
        }
        // The cockpit's window row is shown whole.
        let window = map.cells_of('V').next().unwrap();
        assert!(map.frame(RoomId::Cockpit).max.y >= map.center(window).y + TILE / 2.0);
    }

    #[test]
    fn doors_join_every_room() {
        let map = ship();
        let mut reached = vec![RoomId::Quarters];
        let mut grew = true;
        while grew {
            grew = false;
            for door in map.doors().iter().filter(|d| !d.locked) {
                let step = if door.across_x { Vec2::Y } else { Vec2::X } * TILE / 2.0;
                let (a, b) = (
                    map.room_at(door.center + step),
                    map.room_at(door.center - step),
                );
                for (from, to) in [(a, b), (b, a)] {
                    if reached.contains(&from) && !reached.contains(&to) {
                        reached.push(to);
                        grew = true;
                    }
                }
            }
        }
        assert_eq!(reached.len(), RoomId::ALL.len(), "{reached:?}");
    }

    #[test]
    fn rooms_meet_at_the_middle_of_a_door() {
        let map = ship();
        for door in map.doors().iter().filter(|d| !d.locked) {
            let across = if door.across_x { Vec2::Y } else { Vec2::X };
            let a = map.room_at(door.center + across * 0.5);
            let b = map.room_at(door.center - across * 0.5);
            assert_ne!(a, b, "{door:?}");
            assert_eq!(map.room_at(door.center + across * 40.0), a);
        }
    }

    #[test]
    fn the_cockpit_is_at_the_front_and_the_airlock_at_the_tail() {
        // Pointing up: front to tail runs from the top of the map down.
        let ys: Vec<f32> = RoomId::ALL.iter().map(|r| r.center().y).collect();
        assert!(ys.windows(2).all(|w| w[0] > w[1]), "{ys:?}");
    }

    #[test]
    fn walls_merge_into_long_runs() {
        let map = ship();
        let longest = map
            .walls()
            .iter()
            .map(|r| r.width().max(r.height()))
            .fold(0.0, f32::max);
        assert!(longest > 8.0 * TILE, "{longest}");
        // No two wall rectangles overlap.
        let walls = map.walls();
        for (i, a) in walls.iter().enumerate() {
            for b in &walls[i + 1..] {
                let o = a.intersect(*b);
                assert!(
                    o.width() <= 0.0 || o.height() <= 0.0,
                    "{a:?} overlaps {b:?}"
                );
            }
        }
    }

    #[test]
    fn hull_outline_is_exact_at_inner_corners() {
        let map = ship();
        // Where the narrow cockpit meets the wider quarters' front wall, only
        // the corner outside both rooms is space.
        let quarters = map.cells_of('Q').next().unwrap();
        let t_junction = map
            .cells()
            .find(|&(c, ch)| ch == '#' && c.y == quarters.y - 1 && map.mask(c) == 11)
            .map(|(c, _)| c)
            .expect("a wall joining the cockpit's side wall");
        let quads = map.floor_quadrants(t_junction);
        assert_eq!(quads.iter().filter(|q| !**q).count(), 1, "{quads:?}");
        // A straight hull wall keeps only its inner half.
        let breach = map.wall_mark('7').unwrap();
        let q = map.floor_quadrants(breach.cell);
        assert_eq!(
            q,
            [false, true, false, true],
            "the left hull wall keeps its east half"
        );
    }

    #[test]
    fn a_sideways_ship_derives_the_same_things() {
        let map = ShipMap::parse(SIDEWAYS).unwrap_or_else(|e| panic!("{e}"));
        let door = map.doors()[0];
        assert!(!door.across_x, "a door in a wall running north-south");
        let (left, right) = (
            map.room_at(door.center - Vec2::X * 10.0),
            map.room_at(door.center + Vec2::X * 10.0),
        );
        assert_eq!((left, right), (RoomId::Cockpit, RoomId::Quarters));
        assert!(map.interior(RoomId::Cockpit).contains(map.spawn()));
    }

    #[test]
    fn broken_layouts_are_rejected() {
        let open = "#C.\n###";
        assert!(ShipMap::parse(open).unwrap_err().contains("open to space"));
        let unknown = "#?#";
        assert!(
            ShipMap::parse(unknown)
                .unwrap_err()
                .contains("unknown character")
        );
        let joined = "#####\n#C.Q#\n#####";
        assert!(
            ShipMap::parse(joined)
                .unwrap_err()
                .contains("not walled off")
        );
    }

    #[test]
    fn merge_joins_touching_runs() {
        let rects = vec![
            Rect::new(0.0, 0.0, 10.0, 5.0),
            Rect::new(10.0, 0.0, 30.0, 5.0),
            Rect::new(0.0, 5.0, 30.0, 9.0),
            Rect::new(50.0, 0.0, 60.0, 5.0),
        ];
        let merged = merge(rects);
        assert_eq!(merged.len(), 2, "{merged:?}");
        assert!(merged.contains(&Rect::new(0.0, 0.0, 30.0, 9.0)));
    }

    #[test]
    fn cell_coordinates_round_trip() {
        let map = ship();
        for (cell, _) in map.cells() {
            assert_eq!(map.cell_at(map.center(cell)), cell);
        }
        assert_eq!(
            map.center(map.cell_at(Vec2::ZERO)).x,
            0.0,
            "the spine is at x = 0"
        );
    }
}
