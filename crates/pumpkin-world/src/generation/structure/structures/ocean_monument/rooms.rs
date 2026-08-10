use pumpkin_data::Block;
use pumpkin_util::{
    BlockDirection,
    math::block_box::BlockBox,
    random::{RandomGenerator, RandomImpl},
};

use crate::{ProtoChunk, generation::structure::piece::StructurePieceType};

use super::{
    BASE_BLACK, BASE_GRAY, BASE_LIGHT, LAMP, MonumentPiece,
    graph::{RoomDefinition, RoomGraph},
};

#[derive(Clone, Copy)]
pub(super) enum RoomKind {
    Entry,
    Core,
    DoubleX,
    DoubleXY,
    DoubleY,
    DoubleYZ,
    DoubleZ,
    Simple(i32),
    SimpleTop,
    Wing(i32),
    Penthouse,
}

impl RoomKind {
    pub const fn piece_type(self) -> StructurePieceType {
        match self {
            Self::Entry => StructurePieceType::OceanMonumentEntryRoom,
            Self::Core => StructurePieceType::OceanMonumentCoreRoom,
            Self::DoubleX => StructurePieceType::OceanMonumentDoubleXRoom,
            Self::DoubleXY => StructurePieceType::OceanMonumentDoubleXYRoom,
            Self::DoubleY => StructurePieceType::OceanMonumentDoubleYRoom,
            Self::DoubleYZ => StructurePieceType::OceanMonumentDoubleYZRoom,
            Self::DoubleZ => StructurePieceType::OceanMonumentDoubleZRoom,
            Self::Simple(_) => StructurePieceType::OceanMonumentSimpleRoom,
            Self::SimpleTop => StructurePieceType::OceanMonumentSimpleTopRoom,
            Self::Wing(_) => StructurePieceType::OceanMonumentWingRoom,
            Self::Penthouse => StructurePieceType::OceanMonumentPenthouse,
        }
    }

    const fn dimensions(self) -> (i32, i32, i32) {
        match self {
            Self::Core => (2, 2, 2),
            Self::DoubleX => (2, 1, 1),
            Self::DoubleXY => (2, 2, 1),
            Self::DoubleY => (1, 2, 1),
            Self::DoubleYZ => (1, 2, 2),
            Self::DoubleZ => (1, 1, 2),
            _ => (1, 1, 1),
        }
    }
}

pub(super) struct RoomPiece {
    pub piece: MonumentPiece,
    kind: RoomKind,
    room: Option<usize>,
}

impl RoomPiece {
    pub const fn for_room(
        kind: RoomKind,
        direction: BlockDirection,
        room: usize,
        room_index: i32,
    ) -> Self {
        let dimensions = kind.dimensions();
        Self {
            piece: MonumentPiece::for_room(
                kind,
                direction,
                room_index,
                dimensions.0,
                dimensions.1,
                dimensions.2,
            ),
            kind,
            room: Some(room),
        }
    }

    pub const fn special(
        kind: RoomKind,
        direction: BlockDirection,
        bounding_box: BlockBox,
    ) -> Self {
        Self {
            piece: MonumentPiece::special(kind, direction, bounding_box),
            kind,
            room: None,
        }
    }

    pub fn fit(
        random: &mut RandomGenerator,
        direction: BlockDirection,
        room: usize,
        graph: &mut RoomGraph,
    ) -> Self {
        let room_index = graph.rooms[room].index;
        let east = graph.rooms[room].connections[BlockDirection::East as usize];
        let north = graph.rooms[room].connections[BlockDirection::North as usize];
        let up = graph.rooms[room].connections[BlockDirection::Up as usize];
        let kind = if opening(&graph.rooms[room], BlockDirection::East)
            && east.is_some_and(|east| !graph.rooms[east].claimed)
            && opening(&graph.rooms[room], BlockDirection::Up)
            && up.is_some_and(|up| !graph.rooms[up].claimed)
            && east.is_some_and(|east| {
                opening(&graph.rooms[east], BlockDirection::Up)
                    && graph.rooms[east].connections[BlockDirection::Up as usize]
                        .is_some_and(|up| !graph.rooms[up].claimed)
            }) {
            let east = east.unwrap_or(0);
            let up = up.unwrap_or(0);
            let east_up = graph.connection(east, BlockDirection::Up);
            claim(graph, &[room, east, up, east_up]);
            RoomKind::DoubleXY
        } else if opening(&graph.rooms[room], BlockDirection::North)
            && north.is_some_and(|north| !graph.rooms[north].claimed)
            && opening(&graph.rooms[room], BlockDirection::Up)
            && up.is_some_and(|up| !graph.rooms[up].claimed)
            && north.is_some_and(|north| {
                opening(&graph.rooms[north], BlockDirection::Up)
                    && graph.rooms[north].connections[BlockDirection::Up as usize]
                        .is_some_and(|up| !graph.rooms[up].claimed)
            })
        {
            let north = north.unwrap_or(0);
            let up = up.unwrap_or(0);
            let north_up = graph.connection(north, BlockDirection::Up);
            claim(graph, &[room, north, up, north_up]);
            RoomKind::DoubleYZ
        } else if opening(&graph.rooms[room], BlockDirection::North)
            && north.is_some_and(|north| !graph.rooms[north].claimed)
        {
            let north = north.unwrap_or(0);
            claim(graph, &[room, north]);
            RoomKind::DoubleZ
        } else if opening(&graph.rooms[room], BlockDirection::East)
            && east.is_some_and(|east| !graph.rooms[east].claimed)
        {
            if let Some(east) = east {
                claim(graph, &[room, east]);
            }
            RoomKind::DoubleX
        } else if opening(&graph.rooms[room], BlockDirection::Up)
            && up.is_some_and(|up| !graph.rooms[up].claimed)
        {
            if let Some(up) = up {
                claim(graph, &[room, up]);
            }
            RoomKind::DoubleY
        } else if ![
            BlockDirection::West,
            BlockDirection::East,
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::Up,
        ]
        .into_iter()
        .any(|direction| opening(&graph.rooms[room], direction))
        {
            claim(graph, &[room]);
            RoomKind::SimpleTop
        } else {
            claim(graph, &[room]);
            RoomKind::Simple(random.next_bounded_i32(3))
        };

        Self::for_room(kind, direction, room, room_index)
    }

    pub fn place(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        match self.kind {
            RoomKind::Entry => self.place_entry(chunk, chunk_box, sea_level, graph),
            RoomKind::Core => self.place_core(chunk, chunk_box),
            RoomKind::DoubleX => self.place_double_x(chunk, chunk_box, sea_level, graph),
            RoomKind::DoubleXY => self.place_double_xy(chunk, chunk_box, sea_level, graph),
            RoomKind::DoubleY => self.place_double_y(chunk, chunk_box, sea_level, graph),
            RoomKind::DoubleYZ => self.place_double_yz(chunk, chunk_box, sea_level, graph),
            RoomKind::DoubleZ => self.place_double_z(chunk, chunk_box, sea_level, graph),
            RoomKind::Simple(design) => {
                self.place_simple(chunk, random, chunk_box, sea_level, graph, design);
            }
            RoomKind::SimpleTop => {
                self.place_simple_top(chunk, random, chunk_box, sea_level, graph);
            }
            RoomKind::Wing(design) => {
                self.place_wing(chunk, chunk_box, design);
            }
            RoomKind::Penthouse => {
                self.place_penthouse(chunk, chunk_box, sea_level);
            }
        }
    }

    fn room<'a>(&self, graph: &'a RoomGraph) -> &'a RoomDefinition {
        &graph.rooms[self.room.unwrap_or(0)]
    }

    fn place_entry(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let room = self.room(graph);
        p.fill(chunk, chunk_box, 0, 3, 0, 2, 3, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 5, 3, 0, 7, 3, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 0, 2, 0, 1, 2, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 6, 2, 0, 7, 2, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 0, 1, 0, 0, 1, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 7, 1, 0, 7, 1, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 0, 1, 7, 7, 3, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 1, 0, 2, 3, 0, BASE_LIGHT);
        p.fill(chunk, chunk_box, 5, 1, 0, 6, 3, 0, BASE_LIGHT);
        if opening(room, BlockDirection::North) {
            p.water(chunk, chunk_box, sea_level, 3, 1, 7, 4, 2, 7);
        }
        if opening(room, BlockDirection::West) {
            p.water(chunk, chunk_box, sea_level, 0, 1, 3, 1, 2, 4);
        }
        if opening(room, BlockDirection::East) {
            p.water(chunk, chunk_box, sea_level, 6, 1, 3, 7, 2, 4);
        }
    }

    fn place_core(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox) {
        let p = &self.piece;
        p.fill_on_water(chunk, chunk_box, 1, 8, 0, 14, 8, 14, BASE_GRAY);
        p.fill(chunk, chunk_box, 0, 7, 0, 0, 7, 15, BASE_LIGHT);
        p.fill(chunk, chunk_box, 15, 7, 0, 15, 7, 15, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 7, 0, 15, 7, 0, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 7, 15, 14, 7, 15, BASE_LIGHT);
        for y in 1..=6 {
            let block = if y == 2 || y == 6 {
                BASE_GRAY
            } else {
                BASE_LIGHT
            };
            for x in [0, 15] {
                p.fill(chunk, chunk_box, x, y, 0, x, y, 1, block);
                p.fill(chunk, chunk_box, x, y, 6, x, y, 9, block);
                p.fill(chunk, chunk_box, x, y, 14, x, y, 15, block);
            }
            p.block(chunk, chunk_box, 1, y, 0, block);
            p.fill(chunk, chunk_box, 6, y, 0, 9, y, 0, block);
            p.block(chunk, chunk_box, 14, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 15, 14, y, 15, block);
        }
        p.fill(chunk, chunk_box, 6, 3, 6, 9, 6, 9, BASE_BLACK);
        p.fill(
            chunk,
            chunk_box,
            7,
            4,
            7,
            8,
            5,
            8,
            Block::GOLD_BLOCK.default_state,
        );
        for y in [3, 6] {
            for x in [6, 9] {
                p.block(chunk, chunk_box, x, y, 6, LAMP);
                p.block(chunk, chunk_box, x, y, 9, LAMP);
            }
        }
        for (x, z) in [(5, 6), (5, 9), (10, 6), (10, 9)] {
            p.fill(chunk, chunk_box, x, 1, z, x, 2, z, BASE_LIGHT);
        }
        for (x, z) in [(6, 5), (9, 5), (6, 10), (9, 10)] {
            p.fill(chunk, chunk_box, x, 1, z, x, 2, z, BASE_LIGHT);
        }
        for (x, z) in [(5, 5), (5, 10), (10, 5), (10, 10)] {
            p.fill(chunk, chunk_box, x, 2, z, x, 6, z, BASE_LIGHT);
        }
        for (x, z0, z1) in [(5, 1, 6), (10, 1, 6), (5, 9, 14), (10, 9, 14)] {
            p.fill(chunk, chunk_box, x, 7, z0, x, 7, z1, BASE_LIGHT);
        }
        for (z, x0, x1) in [(5, 1, 6), (10, 1, 6), (5, 9, 14), (10, 9, 14)] {
            p.fill(chunk, chunk_box, x0, 7, z, x1, 7, z, BASE_LIGHT);
        }
        for &(x, z0, z1) in &[(2, 2, 3), (13, 2, 3), (2, 12, 13), (13, 12, 13)] {
            p.fill(chunk, chunk_box, x, 1, z0, x, 1, z1, BASE_LIGHT);
        }
        for (x, z) in [(3, 2), (12, 2), (3, 13), (12, 13)] {
            p.block(chunk, chunk_box, x, 1, z, BASE_LIGHT);
        }
    }

    fn place_double_x(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let west = self.room.unwrap_or(0);
        let east = graph.connection(west, BlockDirection::East);
        let west_room = &graph.rooms[west];
        let east_room = &graph.rooms[east];
        if west_room.index / 25 > 0 {
            p.default_floor(
                chunk,
                chunk_box,
                8,
                0,
                opening(east_room, BlockDirection::Down),
            );
            p.default_floor(
                chunk,
                chunk_box,
                0,
                0,
                opening(west_room, BlockDirection::Down),
            );
        }
        if west_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 4, 1, 7, 4, 6, BASE_GRAY);
        }
        if east_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 8, 4, 1, 14, 4, 6, BASE_GRAY);
        }
        for (y, block) in [(3, BASE_LIGHT), (2, BASE_GRAY), (1, BASE_LIGHT)] {
            p.fill(chunk, chunk_box, 0, y, 0, 0, y, 7, block);
            p.fill(chunk, chunk_box, 15, y, 0, 15, y, 7, block);
            p.fill(chunk, chunk_box, 1, y, 0, 15, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 7, 14, y, 7, block);
        }
        p.fill(chunk, chunk_box, 5, 1, 0, 10, 1, 4, BASE_LIGHT);
        p.fill(chunk, chunk_box, 6, 2, 0, 9, 2, 3, BASE_GRAY);
        p.fill(chunk, chunk_box, 5, 3, 0, 10, 3, 4, BASE_LIGHT);
        p.block(chunk, chunk_box, 6, 2, 3, LAMP);
        p.block(chunk, chunk_box, 9, 2, 3, LAMP);
        for (room, x, south, north, west_or_east) in [
            (west_room, 0, (3, 4), (3, 4), BlockDirection::West),
            (east_room, 15, (11, 12), (11, 12), BlockDirection::East),
        ] {
            if opening(room, BlockDirection::South) {
                p.water(chunk, chunk_box, sea_level, south.0, 1, 0, south.1, 2, 0);
            }
            if opening(room, BlockDirection::North) {
                p.water(chunk, chunk_box, sea_level, north.0, 1, 7, north.1, 2, 7);
            }
            if opening(room, west_or_east) {
                p.water(chunk, chunk_box, sea_level, x, 1, 3, x, 2, 4);
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn place_double_xy(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let west = self.room.unwrap_or(0);
        let east = graph.connection(west, BlockDirection::East);
        let west_up = graph.connection(west, BlockDirection::Up);
        let east_up = graph.connection(east, BlockDirection::Up);
        let west_room = &graph.rooms[west];
        let east_room = &graph.rooms[east];
        let west_up_room = &graph.rooms[west_up];
        let east_up_room = &graph.rooms[east_up];
        if west_room.index / 25 > 0 {
            p.default_floor(
                chunk,
                chunk_box,
                8,
                0,
                opening(east_room, BlockDirection::Down),
            );
            p.default_floor(
                chunk,
                chunk_box,
                0,
                0,
                opening(west_room, BlockDirection::Down),
            );
        }
        if west_up_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 8, 1, 7, 8, 6, BASE_GRAY);
        }
        if east_up_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 8, 8, 1, 14, 8, 6, BASE_GRAY);
        }
        for y in 1..=7 {
            let block = if y == 2 || y == 6 {
                BASE_GRAY
            } else {
                BASE_LIGHT
            };
            p.fill(chunk, chunk_box, 0, y, 0, 0, y, 7, block);
            p.fill(chunk, chunk_box, 15, y, 0, 15, y, 7, block);
            p.fill(chunk, chunk_box, 1, y, 0, 15, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 7, 14, y, 7, block);
        }
        for (x, z0, z1) in [
            (2, 3, 4),
            (3, 2, 2),
            (3, 5, 5),
            (13, 3, 4),
            (11, 2, 2),
            (11, 5, 5),
        ] {
            let x1 = if matches!(x, 3 | 11) { x + 1 } else { x };
            p.fill(chunk, chunk_box, x, 1, z0, x1, 7, z1, BASE_LIGHT);
        }
        p.fill(chunk, chunk_box, 5, 1, 3, 5, 3, 4, BASE_LIGHT);
        p.fill(chunk, chunk_box, 10, 1, 3, 10, 3, 4, BASE_LIGHT);
        p.fill(chunk, chunk_box, 5, 7, 2, 10, 7, 5, BASE_LIGHT);
        for (x, z) in [(5, 2), (10, 2), (5, 5), (10, 5)] {
            p.fill(chunk, chunk_box, x, 5, z, x, 7, z, BASE_LIGHT);
            p.block(chunk, chunk_box, x, 4, z, LAMP);
        }
        for (x, z) in [(6, 2), (9, 2), (6, 5), (9, 5)] {
            p.block(chunk, chunk_box, x, 6, z, BASE_LIGHT);
        }
        p.fill(chunk, chunk_box, 5, 4, 3, 6, 4, 4, BASE_LIGHT);
        p.fill(chunk, chunk_box, 9, 4, 3, 10, 4, 4, BASE_LIGHT);
        for (room, offset_x, side) in [
            (west_room, 0, BlockDirection::West),
            (east_room, 8, BlockDirection::East),
        ] {
            if opening(room, BlockDirection::South) {
                p.water(
                    chunk,
                    chunk_box,
                    sea_level,
                    offset_x + 3,
                    1,
                    0,
                    offset_x + 4,
                    2,
                    0,
                );
            }
            if opening(room, BlockDirection::North) {
                p.water(
                    chunk,
                    chunk_box,
                    sea_level,
                    offset_x + 3,
                    1,
                    7,
                    offset_x + 4,
                    2,
                    7,
                );
            }
            if opening(room, side) {
                let x = if side == BlockDirection::West { 0 } else { 15 };
                p.water(chunk, chunk_box, sea_level, x, 1, 3, x, 2, 4);
            }
        }
        for (room, offset_x, side) in [
            (west_up_room, 0, BlockDirection::West),
            (east_up_room, 8, BlockDirection::East),
        ] {
            if opening(room, BlockDirection::South) {
                p.water(
                    chunk,
                    chunk_box,
                    sea_level,
                    offset_x + 3,
                    5,
                    0,
                    offset_x + 4,
                    6,
                    0,
                );
            }
            if opening(room, BlockDirection::North) {
                p.water(
                    chunk,
                    chunk_box,
                    sea_level,
                    offset_x + 3,
                    5,
                    7,
                    offset_x + 4,
                    6,
                    7,
                );
            }
            if opening(room, side) {
                let x = if side == BlockDirection::West { 0 } else { 15 };
                p.water(chunk, chunk_box, sea_level, x, 5, 3, x, 6, 4);
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn place_double_y(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        _sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let lower = self.room.unwrap_or(0);
        let upper = graph.connection(lower, BlockDirection::Up);
        let lower_room = &graph.rooms[lower];
        let upper_room = &graph.rooms[upper];
        if lower_room.index / 25 > 0 {
            p.default_floor(
                chunk,
                chunk_box,
                0,
                0,
                opening(lower_room, BlockDirection::Down),
            );
        }
        if upper_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 8, 1, 6, 8, 6, BASE_GRAY);
        }
        p.fill(chunk, chunk_box, 0, 4, 0, 0, 4, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 7, 4, 0, 7, 4, 7, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 4, 0, 6, 4, 0, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 4, 7, 6, 4, 7, BASE_LIGHT);
        for (x, z0, z1) in [(2, 1, 2), (5, 1, 2), (2, 5, 6), (5, 5, 6)] {
            p.fill(chunk, chunk_box, x, 4, z0, x, 4, z1, BASE_LIGHT);
        }
        for (x, z) in [(1, 2), (6, 2), (1, 5), (6, 5)] {
            p.block(chunk, chunk_box, x, 4, z, BASE_LIGHT);
        }
        for (level, room) in [(1, lower_room), (5, upper_room)] {
            for (direction, fixed, a, b) in [
                (BlockDirection::South, 0, 2, 5),
                (BlockDirection::North, 7, 2, 5),
            ] {
                if opening(room, direction) {
                    p.fill(
                        chunk,
                        chunk_box,
                        a,
                        level,
                        fixed,
                        a,
                        level + 2,
                        fixed,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        b,
                        level,
                        fixed,
                        b,
                        level + 2,
                        fixed,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        a + 1,
                        level + 2,
                        fixed,
                        b - 1,
                        level + 2,
                        fixed,
                        BASE_LIGHT,
                    );
                } else {
                    p.fill(
                        chunk,
                        chunk_box,
                        0,
                        level,
                        fixed,
                        7,
                        level + 2,
                        fixed,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        0,
                        level + 1,
                        fixed,
                        7,
                        level + 1,
                        fixed,
                        BASE_GRAY,
                    );
                }
            }
            for (direction, fixed, a, b) in [
                (BlockDirection::West, 0, 2, 5),
                (BlockDirection::East, 7, 2, 5),
            ] {
                if opening(room, direction) {
                    p.fill(
                        chunk,
                        chunk_box,
                        fixed,
                        level,
                        a,
                        fixed,
                        level + 2,
                        a,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        fixed,
                        level,
                        b,
                        fixed,
                        level + 2,
                        b,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        fixed,
                        level + 2,
                        a + 1,
                        fixed,
                        level + 2,
                        b - 1,
                        BASE_LIGHT,
                    );
                } else {
                    p.fill(
                        chunk,
                        chunk_box,
                        fixed,
                        level,
                        0,
                        fixed,
                        level + 2,
                        7,
                        BASE_LIGHT,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        fixed,
                        level + 1,
                        0,
                        fixed,
                        level + 1,
                        7,
                        BASE_GRAY,
                    );
                }
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn place_double_yz(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let south = self.room.unwrap_or(0);
        let north = graph.connection(south, BlockDirection::North);
        let south_up = graph.connection(south, BlockDirection::Up);
        let north_up = graph.connection(north, BlockDirection::Up);
        let south_room = &graph.rooms[south];
        let north_room = &graph.rooms[north];
        let south_up_room = &graph.rooms[south_up];
        let north_up_room = &graph.rooms[north_up];
        if south_room.index / 25 > 0 {
            p.default_floor(
                chunk,
                chunk_box,
                0,
                8,
                opening(north_room, BlockDirection::Down),
            );
            p.default_floor(
                chunk,
                chunk_box,
                0,
                0,
                opening(south_room, BlockDirection::Down),
            );
        }
        if south_up_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 8, 1, 6, 8, 7, BASE_GRAY);
        }
        if north_up_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 8, 8, 6, 8, 14, BASE_GRAY);
        }
        for y in 1..=7 {
            let block = if y == 2 || y == 6 {
                BASE_GRAY
            } else {
                BASE_LIGHT
            };
            p.fill(chunk, chunk_box, 0, y, 0, 0, y, 15, block);
            p.fill(chunk, chunk_box, 7, y, 0, 7, y, 15, block);
            p.fill(chunk, chunk_box, 1, y, 0, 6, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 15, 6, y, 15, block);
        }
        for y in 1..=7 {
            let block = if y == 2 || y == 6 { LAMP } else { BASE_BLACK };
            p.fill(chunk, chunk_box, 3, y, 7, 4, y, 8, block);
        }
        for (room, z, cross) in [(south_room, 0, 3), (north_room, 15, 11)] {
            let longitudinal = if z == 0 {
                BlockDirection::South
            } else {
                BlockDirection::North
            };
            if opening(room, longitudinal) {
                p.water(chunk, chunk_box, sea_level, 3, 1, z, 4, 2, z);
            }
            if opening(room, BlockDirection::West) {
                p.water(chunk, chunk_box, sea_level, 0, 1, cross, 0, 2, cross + 1);
            }
            if opening(room, BlockDirection::East) {
                p.water(chunk, chunk_box, sea_level, 7, 1, cross, 7, 2, cross + 1);
            }
        }
        for (room, z, cross) in [(south_up_room, 0, 3), (north_up_room, 15, 11)] {
            let longitudinal = if z == 0 {
                BlockDirection::South
            } else {
                BlockDirection::North
            };
            if opening(room, longitudinal) {
                p.water(chunk, chunk_box, sea_level, 3, 5, z, 4, 6, z);
            }
            if opening(room, BlockDirection::West) {
                p.water(chunk, chunk_box, sea_level, 0, 5, cross, 0, 6, cross + 1);
                p.fill(
                    chunk,
                    chunk_box,
                    1,
                    4,
                    cross - 1,
                    2,
                    4,
                    cross + 2,
                    BASE_LIGHT,
                );
                p.fill(
                    chunk,
                    chunk_box,
                    1,
                    1,
                    cross - 1,
                    1,
                    3,
                    cross - 1,
                    BASE_LIGHT,
                );
                p.fill(
                    chunk,
                    chunk_box,
                    1,
                    1,
                    cross + 2,
                    1,
                    3,
                    cross + 2,
                    BASE_LIGHT,
                );
            }
            if opening(room, BlockDirection::East) {
                p.water(chunk, chunk_box, sea_level, 7, 5, cross, 7, 6, cross + 1);
                p.fill(
                    chunk,
                    chunk_box,
                    5,
                    4,
                    cross - 1,
                    6,
                    4,
                    cross + 2,
                    BASE_LIGHT,
                );
                p.fill(
                    chunk,
                    chunk_box,
                    6,
                    1,
                    cross - 1,
                    6,
                    3,
                    cross - 1,
                    BASE_LIGHT,
                );
                p.fill(
                    chunk,
                    chunk_box,
                    6,
                    1,
                    cross + 2,
                    6,
                    3,
                    cross + 2,
                    BASE_LIGHT,
                );
            }
        }
    }

    fn place_double_z(
        &self,
        chunk: &mut ProtoChunk,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let south = self.room.unwrap_or(0);
        let north = graph.connection(south, BlockDirection::North);
        let south_room = &graph.rooms[south];
        let north_room = &graph.rooms[north];
        if south_room.index / 25 > 0 {
            p.default_floor(
                chunk,
                chunk_box,
                0,
                8,
                opening(north_room, BlockDirection::Down),
            );
            p.default_floor(
                chunk,
                chunk_box,
                0,
                0,
                opening(south_room, BlockDirection::Down),
            );
        }
        if south_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 4, 1, 6, 4, 7, BASE_GRAY);
        }
        if north_room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 4, 8, 6, 4, 14, BASE_GRAY);
        }
        for (y, block) in [(3, BASE_LIGHT), (2, BASE_GRAY), (1, BASE_LIGHT)] {
            p.fill(chunk, chunk_box, 0, y, 0, 0, y, 15, block);
            p.fill(chunk, chunk_box, 7, y, 0, 7, y, 15, block);
            p.fill(chunk, chunk_box, 1, y, 0, 7, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 15, 6, y, 15, block);
        }
        for (x, z0, z1) in [(1, 1, 2), (6, 1, 2), (1, 13, 14), (6, 13, 14)] {
            p.fill(chunk, chunk_box, x, 1, z0, x, 1, z1, BASE_LIGHT);
            p.fill(chunk, chunk_box, x, 3, z0, x, 3, z1, BASE_LIGHT);
        }
        for (x, z) in [(2, 6), (5, 6), (2, 9), (5, 9)] {
            p.fill(chunk, chunk_box, x, 1, z, x, 3, z, BASE_LIGHT);
        }
        p.fill(chunk, chunk_box, 3, 2, 6, 4, 2, 6, BASE_LIGHT);
        p.fill(chunk, chunk_box, 3, 2, 9, 4, 2, 9, BASE_LIGHT);
        p.fill(chunk, chunk_box, 2, 2, 7, 2, 2, 8, BASE_LIGHT);
        p.fill(chunk, chunk_box, 5, 2, 7, 5, 2, 8, BASE_LIGHT);
        for (x, z) in [(2, 5), (5, 5), (2, 10), (5, 10)] {
            p.block(chunk, chunk_box, x, 2, z, LAMP);
            p.block(chunk, chunk_box, x, 3, z, BASE_LIGHT);
        }
        for (room, z, cross) in [(south_room, 0, 3), (north_room, 15, 11)] {
            let longitudinal = if z == 0 {
                BlockDirection::South
            } else {
                BlockDirection::North
            };
            if opening(room, longitudinal) {
                p.water(chunk, chunk_box, sea_level, 3, 1, z, 4, 2, z);
            }
            if opening(room, BlockDirection::West) {
                p.water(chunk, chunk_box, sea_level, 0, 1, cross, 0, 2, cross + 1);
            }
            if opening(room, BlockDirection::East) {
                p.water(chunk, chunk_box, sea_level, 7, 1, cross, 7, 2, cross + 1);
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    fn place_simple(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
        design: i32,
    ) {
        let p = &self.piece;
        let room = self.room(graph);
        if room.index / 25 > 0 {
            p.default_floor(chunk, chunk_box, 0, 0, opening(room, BlockDirection::Down));
        }
        if room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 4, 1, 6, 4, 6, BASE_GRAY);
        }
        let center_pillar = design != 0
            && random.next_bool()
            && !opening(room, BlockDirection::Down)
            && !opening(room, BlockDirection::Up)
            && room.count_openings() > 1;
        match design {
            0 => {
                for (x0, z0, lamp_x, lamp_z) in
                    [(0, 0, 1, 1), (5, 0, 6, 1), (0, 5, 1, 6), (5, 5, 6, 6)]
                {
                    p.fill(chunk, chunk_box, x0, 1, z0, x0 + 2, 1, z0 + 2, BASE_LIGHT);
                    p.fill(chunk, chunk_box, x0, 3, z0, x0 + 2, 3, z0 + 2, BASE_LIGHT);
                    let outer_x = if x0 == 0 { x0 } else { x0 + 2 };
                    let outer_z = if z0 == 0 { z0 } else { z0 + 2 };
                    p.fill(
                        chunk,
                        chunk_box,
                        outer_x,
                        2,
                        z0,
                        outer_x,
                        2,
                        z0 + 2,
                        BASE_GRAY,
                    );
                    p.fill(
                        chunk,
                        chunk_box,
                        x0 + 1 - i32::from(x0 != 0),
                        2,
                        outer_z,
                        x0 + 2 - i32::from(x0 != 0),
                        2,
                        outer_z,
                        BASE_GRAY,
                    );
                    p.block(chunk, chunk_box, lamp_x, 2, lamp_z, LAMP);
                }
                if opening(room, BlockDirection::South) {
                    p.fill(chunk, chunk_box, 3, 3, 0, 4, 3, 0, BASE_LIGHT);
                } else {
                    p.fill(chunk, chunk_box, 3, 3, 0, 4, 3, 1, BASE_LIGHT);
                    p.fill(chunk, chunk_box, 3, 2, 0, 4, 2, 0, BASE_GRAY);
                    p.fill(chunk, chunk_box, 3, 1, 0, 4, 1, 1, BASE_LIGHT);
                }
                if opening(room, BlockDirection::North) {
                    p.fill(chunk, chunk_box, 3, 3, 7, 4, 3, 7, BASE_LIGHT);
                } else {
                    p.fill(chunk, chunk_box, 3, 3, 6, 4, 3, 7, BASE_LIGHT);
                    p.fill(chunk, chunk_box, 3, 2, 7, 4, 2, 7, BASE_GRAY);
                    p.fill(chunk, chunk_box, 3, 1, 6, 4, 1, 7, BASE_LIGHT);
                }
                if opening(room, BlockDirection::West) {
                    p.fill(chunk, chunk_box, 0, 3, 3, 0, 3, 4, BASE_LIGHT);
                } else {
                    p.fill(chunk, chunk_box, 0, 3, 3, 1, 3, 4, BASE_LIGHT);
                    p.fill(chunk, chunk_box, 0, 2, 3, 0, 2, 4, BASE_GRAY);
                    p.fill(chunk, chunk_box, 0, 1, 3, 1, 1, 4, BASE_LIGHT);
                }
                if opening(room, BlockDirection::East) {
                    p.fill(chunk, chunk_box, 7, 3, 3, 7, 3, 4, BASE_LIGHT);
                } else {
                    p.fill(chunk, chunk_box, 6, 3, 3, 7, 3, 4, BASE_LIGHT);
                    p.fill(chunk, chunk_box, 7, 2, 3, 7, 2, 4, BASE_GRAY);
                    p.fill(chunk, chunk_box, 6, 1, 3, 7, 1, 4, BASE_LIGHT);
                }
            }
            1 => {
                for (x, z) in [(2, 2), (2, 5), (5, 5), (5, 2)] {
                    p.fill(chunk, chunk_box, x, 1, z, x, 3, z, BASE_LIGHT);
                    p.block(chunk, chunk_box, x, 2, z, LAMP);
                }
                for (x0, z0, x1, z1) in [
                    (0, 0, 1, 0),
                    (0, 1, 0, 1),
                    (0, 7, 1, 7),
                    (0, 6, 0, 6),
                    (6, 7, 7, 7),
                    (7, 6, 7, 6),
                    (6, 0, 7, 0),
                    (7, 1, 7, 1),
                ] {
                    p.fill(chunk, chunk_box, x0, 1, z0, x1, 3, z1, BASE_LIGHT);
                }
                for (x, z) in [
                    (1, 0),
                    (0, 1),
                    (1, 7),
                    (0, 6),
                    (6, 7),
                    (7, 6),
                    (6, 0),
                    (7, 1),
                ] {
                    p.block(chunk, chunk_box, x, 2, z, BASE_GRAY);
                }
                for (direction, x0, z0, x1, z1) in [
                    (BlockDirection::South, 1, 0, 6, 0),
                    (BlockDirection::North, 1, 7, 6, 7),
                    (BlockDirection::West, 0, 1, 0, 6),
                    (BlockDirection::East, 7, 1, 7, 6),
                ] {
                    if !opening(room, direction) {
                        p.fill(chunk, chunk_box, x0, 1, z0, x1, 1, z1, BASE_LIGHT);
                        p.fill(chunk, chunk_box, x0, 2, z0, x1, 2, z1, BASE_GRAY);
                        p.fill(chunk, chunk_box, x0, 3, z0, x1, 3, z1, BASE_LIGHT);
                    }
                }
            }
            _ => {
                for (y, block) in [(1, BASE_LIGHT), (2, BASE_BLACK), (3, BASE_LIGHT)] {
                    p.fill(chunk, chunk_box, 0, y, 0, 0, y, 7, block);
                    p.fill(chunk, chunk_box, 7, y, 0, 7, y, 7, block);
                    p.fill(chunk, chunk_box, 1, y, 0, 6, y, 0, block);
                    p.fill(chunk, chunk_box, 1, y, 7, 6, y, 7, block);
                }
                for (direction, x0, z0, x1, z1) in [
                    (BlockDirection::South, 3, 0, 4, 0),
                    (BlockDirection::North, 3, 7, 4, 7),
                    (BlockDirection::West, 0, 3, 0, 4),
                    (BlockDirection::East, 7, 3, 7, 4),
                ] {
                    p.fill(chunk, chunk_box, x0, 1, z0, x1, 2, z1, BASE_BLACK);
                    if opening(room, direction) {
                        p.water(chunk, chunk_box, sea_level, x0, 1, z0, x1, 2, z1);
                    }
                }
            }
        }
        if center_pillar {
            p.fill(chunk, chunk_box, 3, 1, 3, 4, 1, 4, BASE_LIGHT);
            p.fill(chunk, chunk_box, 3, 2, 3, 4, 2, 4, BASE_GRAY);
            p.fill(chunk, chunk_box, 3, 3, 3, 4, 3, 4, BASE_LIGHT);
        }
    }

    fn place_simple_top(
        &self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_box: &BlockBox,
        sea_level: i32,
        graph: &RoomGraph,
    ) {
        let p = &self.piece;
        let room = self.room(graph);
        if room.index / 25 > 0 {
            p.default_floor(chunk, chunk_box, 0, 0, opening(room, BlockDirection::Down));
        }
        if room.connections[BlockDirection::Up as usize].is_none() {
            p.fill_on_water(chunk, chunk_box, 1, 4, 1, 6, 4, 6, BASE_GRAY);
        }
        for x in 1..=6 {
            for z in 1..=6 {
                if random.next_bounded_i32(3) != 0 {
                    let y = if random.next_bounded_i32(4) == 0 {
                        2
                    } else {
                        3
                    };
                    p.fill(
                        chunk,
                        chunk_box,
                        x,
                        y,
                        z,
                        x,
                        3,
                        z,
                        Block::WET_SPONGE.default_state,
                    );
                }
            }
        }
        for (y, block) in [(1, BASE_LIGHT), (2, BASE_BLACK), (3, BASE_LIGHT)] {
            p.fill(chunk, chunk_box, 0, y, 0, 0, y, 7, block);
            p.fill(chunk, chunk_box, 7, y, 0, 7, y, 7, block);
            p.fill(chunk, chunk_box, 1, y, 0, 6, y, 0, block);
            p.fill(chunk, chunk_box, 1, y, 7, 6, y, 7, block);
        }
        p.fill(chunk, chunk_box, 0, 1, 3, 0, 2, 4, BASE_BLACK);
        p.fill(chunk, chunk_box, 7, 1, 3, 7, 2, 4, BASE_BLACK);
        p.fill(chunk, chunk_box, 3, 1, 0, 4, 2, 0, BASE_BLACK);
        p.fill(chunk, chunk_box, 3, 1, 7, 4, 2, 7, BASE_BLACK);
        if opening(room, BlockDirection::South) {
            p.water(chunk, chunk_box, sea_level, 3, 1, 0, 4, 2, 0);
        }
    }

    fn place_wing(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox, design: i32) {
        let p = &self.piece;
        if design & 1 == 0 {
            for i in 0..4 {
                p.fill(
                    chunk,
                    chunk_box,
                    10 - i,
                    3 - i,
                    20 - i,
                    12 + i,
                    3 - i,
                    20,
                    BASE_LIGHT,
                );
            }
            p.fill(chunk, chunk_box, 7, 0, 6, 15, 0, 16, BASE_LIGHT);
            p.fill(chunk, chunk_box, 6, 0, 6, 6, 3, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 16, 0, 6, 16, 3, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 7, 1, 7, 7, 1, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 15, 1, 7, 15, 1, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 7, 1, 6, 9, 3, 6, BASE_LIGHT);
            p.fill(chunk, chunk_box, 13, 1, 6, 15, 3, 6, BASE_LIGHT);
            p.fill(chunk, chunk_box, 8, 1, 7, 9, 1, 7, BASE_LIGHT);
            p.fill(chunk, chunk_box, 13, 1, 7, 14, 1, 7, BASE_LIGHT);
            p.fill(chunk, chunk_box, 9, 0, 5, 13, 0, 5, BASE_LIGHT);
            p.fill(chunk, chunk_box, 10, 0, 7, 12, 0, 7, BASE_BLACK);
            p.fill(chunk, chunk_box, 8, 0, 10, 8, 0, 12, BASE_BLACK);
            p.fill(chunk, chunk_box, 14, 0, 10, 14, 0, 12, BASE_BLACK);
            for z in (7..=18).rev().step_by(3) {
                p.block(chunk, chunk_box, 6, 3, z, LAMP);
                p.block(chunk, chunk_box, 16, 3, z, LAMP);
            }
            for (x, z) in [(10, 10), (12, 10), (10, 12), (12, 12)] {
                p.block(chunk, chunk_box, x, 0, z, LAMP);
            }
            p.block(chunk, chunk_box, 8, 3, 6, LAMP);
            p.block(chunk, chunk_box, 14, 3, 6, LAMP);
            for (x, z) in [(4, 4), (18, 4), (4, 18), (18, 18)] {
                p.block(chunk, chunk_box, x, 2, z, BASE_LIGHT);
                p.block(chunk, chunk_box, x, 1, z, LAMP);
                p.block(chunk, chunk_box, x, 0, z, BASE_LIGHT);
            }
            p.block(chunk, chunk_box, 9, 7, 20, BASE_LIGHT);
            p.block(chunk, chunk_box, 13, 7, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 6, 0, 21, 7, 4, 21, BASE_LIGHT);
            p.fill(chunk, chunk_box, 15, 0, 21, 16, 4, 21, BASE_LIGHT);
            p.spawn_elder(chunk, chunk_box, 11, 2, 16);
        } else {
            p.fill(chunk, chunk_box, 9, 3, 18, 13, 3, 20, BASE_LIGHT);
            p.fill(chunk, chunk_box, 9, 0, 18, 9, 2, 18, BASE_LIGHT);
            p.fill(chunk, chunk_box, 13, 0, 18, 13, 2, 18, BASE_LIGHT);
            for x in [9, 13] {
                p.block(chunk, chunk_box, x, 6, 20, BASE_LIGHT);
                p.block(chunk, chunk_box, x, 5, 20, LAMP);
                p.block(chunk, chunk_box, x, 4, 20, BASE_LIGHT);
            }
            p.fill(chunk, chunk_box, 7, 3, 7, 15, 3, 14, BASE_LIGHT);
            for x in [10, 12] {
                p.fill(chunk, chunk_box, x, 0, 10, x, 6, 10, BASE_LIGHT);
                p.fill(chunk, chunk_box, x, 0, 12, x, 6, 12, BASE_LIGHT);
                for z in [10, 12] {
                    p.block(chunk, chunk_box, x, 0, z, LAMP);
                    p.block(chunk, chunk_box, x, 4, z, LAMP);
                }
            }
            for x in [8, 14] {
                p.fill(chunk, chunk_box, x, 0, 7, x, 2, 7, BASE_LIGHT);
                p.fill(chunk, chunk_box, x, 0, 14, x, 2, 14, BASE_LIGHT);
            }
            p.fill(chunk, chunk_box, 8, 3, 8, 8, 3, 13, BASE_BLACK);
            p.fill(chunk, chunk_box, 14, 3, 8, 14, 3, 13, BASE_BLACK);
            p.spawn_elder(chunk, chunk_box, 11, 5, 13);
        }
    }

    fn place_penthouse(&self, chunk: &mut ProtoChunk, chunk_box: &BlockBox, sea_level: i32) {
        let p = &self.piece;
        p.fill(chunk, chunk_box, 2, -1, 2, 11, -1, 11, BASE_LIGHT);
        p.fill(chunk, chunk_box, 0, -1, 0, 1, -1, 11, BASE_GRAY);
        p.fill(chunk, chunk_box, 12, -1, 0, 13, -1, 11, BASE_GRAY);
        p.fill(chunk, chunk_box, 2, -1, 0, 11, -1, 1, BASE_GRAY);
        p.fill(chunk, chunk_box, 2, -1, 12, 11, -1, 13, BASE_GRAY);
        p.fill(chunk, chunk_box, 0, 0, 0, 0, 0, 13, BASE_LIGHT);
        p.fill(chunk, chunk_box, 13, 0, 0, 13, 0, 13, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 0, 0, 12, 0, 0, BASE_LIGHT);
        p.fill(chunk, chunk_box, 1, 0, 13, 12, 0, 13, BASE_LIGHT);
        for i in (2..=11).step_by(3) {
            p.block(chunk, chunk_box, 0, 0, i, LAMP);
            p.block(chunk, chunk_box, 13, 0, i, LAMP);
            p.block(chunk, chunk_box, i, 0, 0, LAMP);
        }
        p.fill(chunk, chunk_box, 2, 0, 3, 4, 0, 9, BASE_LIGHT);
        p.fill(chunk, chunk_box, 9, 0, 3, 11, 0, 9, BASE_LIGHT);
        p.fill(chunk, chunk_box, 4, 0, 9, 9, 0, 11, BASE_LIGHT);
        for (x, z) in [(5, 8), (8, 8), (10, 10), (3, 10)] {
            p.block(chunk, chunk_box, x, 0, z, BASE_LIGHT);
        }
        p.fill(chunk, chunk_box, 3, 0, 3, 3, 0, 7, BASE_BLACK);
        p.fill(chunk, chunk_box, 10, 0, 3, 10, 0, 7, BASE_BLACK);
        p.fill(chunk, chunk_box, 6, 0, 10, 7, 0, 10, BASE_BLACK);
        for x in [3, 10] {
            for z in (2..=8).step_by(3) {
                p.fill(chunk, chunk_box, x, 0, z, x, 2, z, BASE_LIGHT);
            }
        }
        p.fill(chunk, chunk_box, 5, 0, 10, 5, 2, 10, BASE_LIGHT);
        p.fill(chunk, chunk_box, 8, 0, 10, 8, 2, 10, BASE_LIGHT);
        p.fill(chunk, chunk_box, 6, -1, 7, 7, -1, 8, BASE_BLACK);
        p.water(chunk, chunk_box, sea_level, 6, -1, 3, 7, -1, 4);
        p.spawn_elder(chunk, chunk_box, 6, 1, 6);
    }
}

const fn opening(room: &RoomDefinition, direction: BlockDirection) -> bool {
    room.openings[direction as usize]
}

fn claim(graph: &mut RoomGraph, rooms: &[usize]) {
    for &room in rooms {
        graph.rooms[room].claimed = true;
    }
}
