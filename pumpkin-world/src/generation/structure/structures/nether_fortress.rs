use std::sync::{Arc, Mutex};

use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockHalf, BlockProperties, ChestType, HorizontalFacing, StairShape},
};
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        section_coords,
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

macro_rules! impl_piece_base {
    ($ty:ty) => {
        impl StructurePieceBase for $ty {
            fn get_structure_piece(&self) -> &StructurePiece {
                &self.piece.piece
            }
            fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
                &mut self.piece.piece
            }
            fn clone_box(&self) -> Box<dyn StructurePieceBase> {
                Box::new(self.clone())
            }
            fn place(
                &mut self,
                chunk: &mut ProtoChunk,
                random: &mut RandomGenerator,
                seed: i64,
                chunk_box: &BlockBox,
            ) {
                self.place_blocks(chunk, random, seed, chunk_box);
            }
        }
    };
}

type FenceProps = pumpkin_data::block_properties::OakFenceLikeProperties;

fn make_fence(n: bool, s: bool, e: bool, w: bool) -> &'static BlockState {
    let props = FenceProps {
        north: n,
        south: s,
        east: e,
        west: w,
        waterlogged: false,
    };
    BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_FENCE))
}

fn fence_we() -> &'static BlockState {
    make_fence(false, false, true, true)
}
fn fence_ns() -> &'static BlockState {
    make_fence(true, true, false, false)
}
fn fence_nsw() -> &'static BlockState {
    make_fence(true, true, false, true)
}
fn fence_nse() -> &'static BlockState {
    make_fence(true, true, true, false)
}
fn fence_ne() -> &'static BlockState {
    make_fence(true, false, true, false)
}
fn fence_se() -> &'static BlockState {
    make_fence(false, true, true, false)
}
fn fence_sw() -> &'static BlockState {
    make_fence(false, true, false, true)
}
fn fence_nw() -> &'static BlockState {
    make_fence(true, false, false, true)
}
fn fence_e() -> &'static BlockState {
    make_fence(false, false, true, false)
}
fn fence_w() -> &'static BlockState {
    make_fence(false, false, false, true)
}

type StairProps = pumpkin_data::block_properties::OakStairsLikeProperties;

fn make_stairs(facing: HorizontalFacing) -> &'static BlockState {
    let props = StairProps {
        facing,
        half: BlockHalf::Bottom,
        shape: StairShape::Straight,
        waterlogged: false,
    };
    BlockState::from_id(props.to_state_id(&Block::NETHER_BRICK_STAIRS))
}

type ChestProps = pumpkin_data::block_properties::ChestLikeProperties;

fn make_chest(facing: HorizontalFacing) -> &'static BlockState {
    let props = ChestProps {
        facing,
        r#type: ChestType::Single,
        waterlogged: false,
    };
    BlockState::from_id(props.to_state_id(&Block::CHEST))
}

#[derive(Deserialize)]
pub struct NetherFortressGenerator;

impl StructureGenerator for NetherFortressGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let mut collector = StructurePiecesCollector::default();
        let mut random = context.random;

        let start_x = section_coords::section_to_block(context.chunk_x) + 2;
        let start_z = section_coords::section_to_block(context.chunk_z) + 2;

        let mut start = StartPiece::new(&mut random, start_x, start_z);

        let start_piece = start.piece.clone();
        collector.add_piece(Box::new(start_piece.clone()));

        start_piece.fill_openings(&mut start, &mut random, &mut collector);

        while !start.pieces.is_empty() {
            let idx = random.next_bounded_i32(start.pieces.len() as i32) as usize;
            let mut piece = start.pieces.remove(idx);
            piece.fill_openings(&mut start, &mut random, &mut collector);
        }

        if collector.is_empty() {
            return None;
        }

        collector.shift_into_y_range(&mut random, 48, 70);

        Some(StructurePosition {
            start_pos: BlockPos::new(
                section_coords::section_to_block(context.chunk_x),
                64,
                section_coords::section_to_block(context.chunk_z),
            ),
            collector: Arc::new(Mutex::new(collector)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetherFortressPieceType {
    Bridge,
    BridgeCrossing,
    BridgeSmallCrossing,
    BridgeStairs,
    BridgePlatform,
    BridgeEnd,
    CorridorExit,
    SmallCorridor,
    CorridorCrossing,
    CorridorRightTurn,
    CorridorLeftTurn,
    CorridorStairs,
    CorridorBalcony,
    CorridorNetherWartsRoom,
}

#[derive(Clone)]
pub struct PieceData {
    pub piece_type: NetherFortressPieceType,
    pub weight: i32,
    pub limit: i32,
    pub generated_count: i32,
    pub repeatable: bool,
}

impl PieceData {
    const fn new(piece_type: NetherFortressPieceType, weight: i32, limit: i32) -> Self {
        Self {
            piece_type,
            weight,
            limit,
            generated_count: 0,
            repeatable: false,
        }
    }

    const fn new_repeatable(piece_type: NetherFortressPieceType, weight: i32, limit: i32) -> Self {
        Self {
            piece_type,
            weight,
            limit,
            generated_count: 0,
            repeatable: true,
        }
    }

    fn can_generate(&self) -> bool {
        self.limit == 0 || self.generated_count < self.limit
    }
}

fn get_bridge_pieces() -> Vec<PieceData> {
    vec![
        PieceData::new_repeatable(NetherFortressPieceType::Bridge, 30, 0),
        PieceData::new(NetherFortressPieceType::BridgeCrossing, 10, 4),
        PieceData::new(NetherFortressPieceType::BridgeSmallCrossing, 10, 4),
        PieceData::new(NetherFortressPieceType::BridgeStairs, 10, 3),
        PieceData::new(NetherFortressPieceType::BridgePlatform, 5, 2),
        PieceData::new(NetherFortressPieceType::CorridorExit, 5, 1),
    ]
}

fn get_corridor_pieces() -> Vec<PieceData> {
    vec![
        PieceData::new_repeatable(NetherFortressPieceType::SmallCorridor, 25, 0),
        PieceData::new(NetherFortressPieceType::CorridorCrossing, 15, 5),
        PieceData::new(NetherFortressPieceType::CorridorRightTurn, 5, 10),
        PieceData::new(NetherFortressPieceType::CorridorLeftTurn, 5, 10),
        PieceData::new_repeatable(NetherFortressPieceType::CorridorStairs, 10, 3),
        PieceData::new(NetherFortressPieceType::CorridorBalcony, 7, 2),
        PieceData::new(NetherFortressPieceType::CorridorNetherWartsRoom, 5, 2),
    ]
}

pub struct StartPiece {
    pub piece: BridgeCrossingPiece,
    pub bridge_pieces: Vec<PieceData>,
    pub corridor_pieces: Vec<PieceData>,
    pub last_piece: Option<NetherFortressPieceType>,
    pub pieces: Vec<FortressPiece>,
}

impl StartPiece {
    pub fn new(random: &mut RandomGenerator, x: i32, z: i32) -> Self {
        let facing = StructurePiece::get_random_horizontal_direction(random);
        let bbox = BlockBox::create_box(x, 64, z, facing.get_axis(), 19, 10, 19);

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeCrossing,
            NetherFortressPieceType::BridgeCrossing,
            0,
            bbox,
        );
        piece.piece.set_facing(Some(facing));

        Self {
            piece: BridgeCrossingPiece { piece },
            bridge_pieces: get_bridge_pieces(),
            corridor_pieces: get_corridor_pieces(),
            last_piece: None,
            pieces: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct NetherFortressPiece {
    pub piece: StructurePiece,
    pub piece_type: NetherFortressPieceType,
}

impl NetherFortressPiece {
    pub const fn new(
        structure_type: StructurePieceType,
        piece_type: NetherFortressPieceType,
        chain_length: u32,
        bbox: BlockBox,
    ) -> Self {
        Self {
            piece: StructurePiece::new(structure_type, bbox, chain_length),
            piece_type,
        }
    }

    fn is_in_bounds(bb: &BlockBox) -> bool {
        bb.min.y > 10
    }
}

#[derive(Clone)]
pub enum FortressPiece {
    Bridge(BridgePiece),
    BridgeCrossing(BridgeCrossingPiece),
    BridgeSmallCrossing(BridgeSmallCrossingPiece),
    BridgeStairs(BridgeStairsPiece),
    BridgePlatform(BridgePlatformPiece),
    BridgeEnd(BridgeEndPiece),
    CorridorExit(CorridorExitPiece),
    SmallCorridor(SmallCorridorPiece),
    CorridorCrossing(CorridorCrossingPiece),
    CorridorRightTurn(CorridorRightTurnPiece),
    CorridorLeftTurn(CorridorLeftTurnPiece),
    CorridorStairs(CorridorStairsPiece),
    CorridorBalcony(CorridorBalconyPiece),
    CorridorNetherWartsRoom(CorridorNetherWartsRoomPiece),
}

macro_rules! for_each_variant {
    ($self:expr, $p:ident => $body:expr) => {
        match $self {
            FortressPiece::Bridge($p) => $body,
            FortressPiece::BridgeCrossing($p) => $body,
            FortressPiece::BridgeSmallCrossing($p) => $body,
            FortressPiece::BridgeStairs($p) => $body,
            FortressPiece::BridgePlatform($p) => $body,
            FortressPiece::BridgeEnd($p) => $body,
            FortressPiece::CorridorExit($p) => $body,
            FortressPiece::SmallCorridor($p) => $body,
            FortressPiece::CorridorCrossing($p) => $body,
            FortressPiece::CorridorRightTurn($p) => $body,
            FortressPiece::CorridorLeftTurn($p) => $body,
            FortressPiece::CorridorStairs($p) => $body,
            FortressPiece::CorridorBalcony($p) => $body,
            FortressPiece::CorridorNetherWartsRoom($p) => $body,
        }
    };
}

impl FortressPiece {
    fn piece(&self) -> &StructurePiece {
        for_each_variant!(self, p => &p.piece.piece)
    }

    fn fill_openings(
        &mut self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        match self {
            FortressPiece::BridgePlatform(_) | FortressPiece::BridgeEnd(_) => {}
            FortressPiece::Bridge(p) => p.fill_openings(start, random, collector),
            FortressPiece::BridgeCrossing(p) => p.fill_openings(start, random, collector),
            FortressPiece::BridgeSmallCrossing(p) => p.fill_openings(start, random, collector),
            FortressPiece::BridgeStairs(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorExit(p) => p.fill_openings(start, random, collector),
            FortressPiece::SmallCorridor(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorCrossing(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorRightTurn(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorLeftTurn(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorStairs(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorBalcony(p) => p.fill_openings(start, random, collector),
            FortressPiece::CorridorNetherWartsRoom(p) => p.fill_openings(start, random, collector),
        }
    }

    fn to_boxed(&self) -> Box<dyn StructurePieceBase> {
        for_each_variant!(self, p => Box::new(p.clone()))
    }

    fn bounding_box(&self) -> BlockBox {
        self.piece().bounding_box
    }
}

fn check_remaining_pieces(pieces: &[PieceData]) -> i32 {
    let mut has_limited = false;
    let mut total_weight = 0;

    for p in pieces {
        if p.limit > 0 && p.generated_count < p.limit {
            has_limited = true;
        }
        total_weight += p.weight;
    }

    if has_limited { total_weight } else { -1 }
}

#[allow(clippy::too_many_arguments)]
fn pick_piece(
    start: &mut StartPiece,
    inside: bool,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    chain_length: u32,
    collector: &StructurePiecesCollector,
) -> Option<FortressPiece> {
    let pieces = if inside {
        &mut start.corridor_pieces
    } else {
        &mut start.bridge_pieces
    };
    let total_weight = check_remaining_pieces(pieces);
    let can_generate = total_weight > 0 && chain_length <= 30;

    if !can_generate {
        return create_bridge_end(random, x, y, z, facing, chain_length, collector);
    }

    for _ in 0..5 {
        let mut target = random.next_bounded_i32(total_weight);

        let mut selected_idx = None;
        for (idx, piece_data) in pieces.iter().enumerate() {
            target -= piece_data.weight;
            if target < 0 {
                if !piece_data.can_generate() {
                    break;
                }
                if Some(piece_data.piece_type) == start.last_piece && !piece_data.repeatable {
                    break;
                }
                selected_idx = Some(idx);
                break;
            }
        }

        if let Some(idx) = selected_idx {
            let piece_data = &pieces[idx];
            let piece_type = piece_data.piece_type;

            if let Some(new_piece) =
                create_piece(piece_type, random, x, y, z, facing, chain_length, collector)
            {
                pieces[idx].generated_count += 1;
                start.last_piece = Some(piece_type);

                if !pieces[idx].can_generate() {
                    pieces.remove(idx);
                }

                return Some(new_piece);
            }
        }
    }

    create_bridge_end(random, x, y, z, facing, chain_length, collector)
}

fn create_bridge_end(
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    chain_length: u32,
    collector: &StructurePiecesCollector,
) -> Option<FortressPiece> {
    BridgeEndPiece::create(random, x, y, z, facing, chain_length, collector)
        .map(FortressPiece::BridgeEnd)
}

#[allow(clippy::too_many_arguments)]
fn create_piece(
    piece_type: NetherFortressPieceType,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    chain_length: u32,
    collector: &StructurePiecesCollector,
) -> Option<FortressPiece> {
    match piece_type {
        NetherFortressPieceType::Bridge => {
            BridgePiece::create(x, y, z, facing, chain_length, collector).map(FortressPiece::Bridge)
        }
        NetherFortressPieceType::BridgeCrossing => {
            BridgeCrossingPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::BridgeCrossing)
        }
        NetherFortressPieceType::BridgeSmallCrossing => {
            BridgeSmallCrossingPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::BridgeSmallCrossing)
        }
        NetherFortressPieceType::BridgeStairs => {
            BridgeStairsPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::BridgeStairs)
        }
        NetherFortressPieceType::BridgePlatform => {
            BridgePlatformPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::BridgePlatform)
        }
        NetherFortressPieceType::BridgeEnd => {
            BridgeEndPiece::create(random, x, y, z, facing, chain_length, collector)
                .map(FortressPiece::BridgeEnd)
        }
        NetherFortressPieceType::CorridorExit => {
            CorridorExitPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorExit)
        }
        NetherFortressPieceType::SmallCorridor => {
            SmallCorridorPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::SmallCorridor)
        }
        NetherFortressPieceType::CorridorCrossing => {
            CorridorCrossingPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorCrossing)
        }
        NetherFortressPieceType::CorridorRightTurn => {
            CorridorRightTurnPiece::create(random, x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorRightTurn)
        }
        NetherFortressPieceType::CorridorLeftTurn => {
            CorridorLeftTurnPiece::create(random, x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorLeftTurn)
        }
        NetherFortressPieceType::CorridorStairs => {
            CorridorStairsPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorStairs)
        }
        NetherFortressPieceType::CorridorBalcony => {
            CorridorBalconyPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorBalcony)
        }
        NetherFortressPieceType::CorridorNetherWartsRoom => {
            CorridorNetherWartsRoomPiece::create(x, y, z, facing, chain_length, collector)
                .map(FortressPiece::CorridorNetherWartsRoom)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn piece_generator(
    start: &mut StartPiece,
    random: &mut RandomGenerator,
    x: i32,
    y: i32,
    z: i32,
    facing: BlockDirection,
    chain_length: u32,
    inside: bool,
    collector: &mut StructurePiecesCollector,
) -> Option<FortressPiece> {
    let start_bbox = start.piece.piece.piece.bounding_box;

    if (x - start_bbox.min.x).abs() > 112 || (z - start_bbox.min.z).abs() > 112 {
        return create_bridge_end(random, x, y, z, facing, chain_length, collector);
    }

    if let Some(new_piece) = pick_piece(
        start,
        inside,
        random,
        x,
        y,
        z,
        facing,
        chain_length + 1,
        collector,
    ) {
        collector.add_piece(new_piece.to_boxed());
        start.pieces.push(new_piece.clone());
        return Some(new_piece);
    }

    None
}

fn fill_forward_opening(
    piece: &StructurePiece,
    start: &mut StartPiece,
    random: &mut RandomGenerator,
    left_right_offset: i32,
    height_offset: i32,
    inside: bool,
    collector: &mut StructurePiecesCollector,
) {
    let facing = piece.facing.unwrap_or(BlockDirection::North);
    let bbox = piece.bounding_box;
    let chain_length = piece.chain_length;

    let (x, z, new_facing) = match facing {
        BlockDirection::North => (bbox.min.x + left_right_offset, bbox.min.z - 1, facing),
        BlockDirection::South => (bbox.min.x + left_right_offset, bbox.max.z + 1, facing),
        BlockDirection::West => (bbox.min.x - 1, bbox.min.z + left_right_offset, facing),
        BlockDirection::East => (bbox.max.x + 1, bbox.min.z + left_right_offset, facing),
        _ => return,
    };

    piece_generator(
        start,
        random,
        x,
        bbox.min.y + height_offset,
        z,
        new_facing,
        chain_length,
        inside,
        collector,
    );
}

fn fill_nw_opening(
    piece: &StructurePiece,
    start: &mut StartPiece,
    random: &mut RandomGenerator,
    height_offset: i32,
    left_right_offset: i32,
    inside: bool,
    collector: &mut StructurePiecesCollector,
) {
    let facing = piece.facing.unwrap_or(BlockDirection::North);
    let bbox = piece.bounding_box;
    let chain_length = piece.chain_length;

    let (x, z, new_facing) = match facing {
        BlockDirection::North | BlockDirection::South => (
            bbox.min.x - 1,
            bbox.min.z + left_right_offset,
            BlockDirection::West,
        ),
        BlockDirection::West | BlockDirection::East => (
            bbox.min.x + left_right_offset,
            bbox.min.z - 1,
            BlockDirection::North,
        ),
        _ => return,
    };

    piece_generator(
        start,
        random,
        x,
        bbox.min.y + height_offset,
        z,
        new_facing,
        chain_length,
        inside,
        collector,
    );
}

fn fill_se_opening(
    piece: &StructurePiece,
    start: &mut StartPiece,
    random: &mut RandomGenerator,
    height_offset: i32,
    left_right_offset: i32,
    inside: bool,
    collector: &mut StructurePiecesCollector,
) {
    let facing = piece.facing.unwrap_or(BlockDirection::North);
    let bbox = piece.bounding_box;
    let chain_length = piece.chain_length;

    let (x, z, new_facing) = match facing {
        BlockDirection::North | BlockDirection::South => (
            bbox.max.x + 1,
            bbox.min.z + left_right_offset,
            BlockDirection::East,
        ),
        BlockDirection::West | BlockDirection::East => (
            bbox.min.x + left_right_offset,
            bbox.max.z + 1,
            BlockDirection::South,
        ),
        _ => return,
    };

    piece_generator(
        start,
        random,
        x,
        bbox.min.y + height_offset,
        z,
        new_facing,
        chain_length,
        inside,
        collector,
    );
}

#[derive(Clone)]
pub struct BridgePiece {
    pub piece: NetherFortressPiece,
}

impl BridgePiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, -3, 0, 5, 10, 19, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridge,
            NetherFortressPieceType::Bridge,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 1, 3, false, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 0, 4, 4, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 5, 0, 3, 7, 18, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 0, 5, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 5, 0, 4, 5, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 2, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 13, 4, 2, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 15, 4, 1, 18, bricks, bricks);

        for i in 0..=4 {
            for j in 0..=2 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, 18 - j, &bb);
            }
        }

        // Fence railings with proper connections
        let fence_nse = fence_nse();
        let fence_nsw = fence_nsw();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 1, 1, 0, 4, 1, fence_nse, fence_nse);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 4, 0, 4, 4, fence_nse, fence_nse);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 14, 0, 4, 14, fence_nse, fence_nse);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 1, 17, 0, 4, 17, fence_nse, fence_nse);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 1, 1, 4, 4, 1, fence_nsw, fence_nsw);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 4, 4, 4, 4, fence_nsw, fence_nsw);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 14, 4, 4, 14, fence_nsw, fence_nsw);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 1, 17, 4, 4, 17, fence_nsw, fence_nsw);
    }
}

impl_piece_base!(BridgePiece);

#[derive(Clone)]
pub struct BridgeCrossingPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeCrossingPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -8, -3, 0, 19, 10, 19, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeCrossing,
            NetherFortressPieceType::BridgeCrossing,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 8, 3, false, collector);
        fill_nw_opening(&self.piece.piece, start, random, 3, 8, false, collector);
        fill_se_opening(&self.piece.piece, start, random, 3, 8, false, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        // Main cross platform
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 3, 0, 11, 4, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 7, 18, 4, 11, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 7, 18, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 8, 18, 7, 10, air, air);

        // Side walls
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 5, 0, 7, 5, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 5, 11, 7, 5, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 0, 11, 5, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 11, 11, 5, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 7, 7, 5, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 7, 18, 5, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 11, 7, 5, 11, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 11, 18, 5, 11, bricks, bricks);

        // Lower sections
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 2, 0, 11, 2, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 2, 13, 11, 2, 18, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 0, 0, 11, 1, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 0, 15, 11, 1, 18, bricks, bricks);

        for i in 7..=11 {
            for j in 0..=2 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, 18 - j, &bb);
            }
        }

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 7, 5, 2, 11, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 13, 2, 7, 18, 2, 11, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 7, 3, 1, 11, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 15, 0, 7, 18, 1, 11, bricks, bricks);

        for i in 0..=2 {
            for j in 7..=11 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, 18 - i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(BridgeCrossingPiece);

#[derive(Clone)]
pub struct BridgeSmallCrossingPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeSmallCrossingPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 9, 7, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeSmallCrossing,
            NetherFortressPieceType::BridgeSmallCrossing,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 2, 0, false, collector);
        fill_nw_opening(&self.piece.piece, start, random, 0, 2, false, collector);
        fill_se_opening(&self.piece.piece, start, random, 0, 2, false, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 6, 1, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 7, 6, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 1, 6, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 6, 1, 6, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 2, 0, 6, 6, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 2, 6, 6, 6, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 6, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 5, 0, 6, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 2, 0, 6, 6, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 2, 5, 6, 6, 6, bricks, bricks);

        let fence_we = fence_we();
        let fence_ns = fence_ns();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 6, 0, 4, 6, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 5, 0, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 6, 6, 4, 6, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 6, 4, 5, 6, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 2, 0, 6, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 2, 0, 5, 4, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 6, 2, 6, 6, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 5, 2, 6, 5, 4, fence_ns, fence_ns);

        for i in 0..=6 {
            for j in 0..=6 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(BridgeSmallCrossingPiece);

#[derive(Clone)]
pub struct BridgeStairsPiece {
    pub piece: NetherFortressPiece,
}

impl BridgeStairsPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 11, 7, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeStairs,
            NetherFortressPieceType::BridgeStairs,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_se_opening(&self.piece.piece, start, random, 6, 2, false, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 6, 1, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 10, 6, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 1, 8, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 2, 0, 6, 8, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 1, 0, 8, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 2, 1, 6, 8, 6, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 2, 6, 5, 8, 6, bricks, bricks);

        let fence_we = fence_we();
        let fence_ns = fence_ns();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 2, 0, 5, 4, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 3, 2, 6, 5, 2, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 3, 4, 6, 5, 4, fence_ns, fence_ns);

        self.piece.piece.add_block(chunk, bricks, 5, 2, 5, &bb);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 5, 4, 3, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 3, 2, 5, 3, 4, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 2, 5, 2, 5, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 2, 5, 1, 6, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 7, 1, 5, 7, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 8, 2, 6, 8, 4, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 6, 0, 4, 8, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 5, 0, fence_we, fence_we);

        for i in 0..=6 {
            for j in 0..=6 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(BridgeStairsPiece);

#[derive(Clone)]
pub struct BridgePlatformPiece {
    pub piece: NetherFortressPiece,
    pub has_blaze_spawner: bool,
}

impl BridgePlatformPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -2, 0, 0, 7, 8, 9, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgePlatform,
            NetherFortressPieceType::BridgePlatform,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self {
            piece,
            has_blaze_spawner: false,
        })
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;
        let spawner = Block::SPAWNER.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 6, 7, 7, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 0, 0, 5, 1, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 2, 1, 5, 2, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 2, 5, 3, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 4, 3, 5, 4, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 2, 0, 1, 4, 2, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 2, 0, 5, 4, 2, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 5, 2, 1, 5, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 5, 2, 5, 5, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 3, 0, 5, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 5, 3, 6, 5, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 5, 8, 5, 5, 8, bricks, bricks);

        // Fence railings
        self.piece.piece.add_block(chunk, fence_w(), 1, 6, 3, &bb);
        self.piece.piece.add_block(chunk, fence_e(), 5, 6, 3, &bb);
        self.piece.piece.add_block(chunk, fence_ne(), 0, 6, 3, &bb);
        self.piece.piece.add_block(chunk, fence_nw(), 6, 6, 3, &bb);
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            0,
            6,
            4,
            0,
            6,
            7,
            fence_ns(),
            fence_ns(),
        );
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            6,
            6,
            4,
            6,
            6,
            7,
            fence_ns(),
            fence_ns(),
        );
        self.piece.piece.add_block(chunk, fence_se(), 0, 6, 8, &bb);
        self.piece.piece.add_block(chunk, fence_sw(), 6, 6, 8, &bb);
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            1,
            6,
            8,
            5,
            6,
            8,
            fence_we(),
            fence_we(),
        );
        self.piece.piece.add_block(chunk, fence_e(), 1, 7, 8, &bb);
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            2,
            7,
            8,
            4,
            7,
            8,
            fence_we(),
            fence_we(),
        );
        self.piece.piece.add_block(chunk, fence_w(), 5, 7, 8, &bb);
        self.piece.piece.add_block(chunk, fence_e(), 2, 8, 8, &bb);
        self.piece.piece.add_block(chunk, fence_we(), 3, 8, 8, &bb);
        self.piece.piece.add_block(chunk, fence_w(), 4, 8, 8, &bb);

        // Blaze spawner (guarded to prevent double-placement across chunk boundaries)
        if !self.has_blaze_spawner {
            let pos = self.piece.piece.offset_pos(3, 5, 5);
            if bb.contains_pos(&pos) {
                self.has_blaze_spawner = true;
                self.piece.piece.add_block(chunk, spawner, 3, 5, 5, &bb);
                // TODO: Set block entity type to Blaze (requires ProtoChunk block entity support)
            }
        }

        for i in 0..=6 {
            for j in 0..=6 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(BridgePlatformPiece);

#[derive(Clone)]
pub struct BridgeEndPiece {
    pub piece: NetherFortressPiece,
    pub seed: i32,
}

impl BridgeEndPiece {
    pub fn create(
        random: &mut RandomGenerator,
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, -3, 0, 5, 10, 8, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeEnd,
            NetherFortressPieceType::BridgeEnd,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self {
            piece,
            seed: random.next_i32(),
        })
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;

        let mut rng = RandomGenerator::Legacy(
            pumpkin_util::random::legacy_rand::LegacyRand::from_seed(self.seed as u64),
        );

        for i in 0..=4 {
            for j in 3..=4 {
                let k = rng.next_bounded_i32(8);
                self.piece
                    .piece
                    .fill_with_outline(chunk, &bb, false, i, j, 0, i, j, k, bricks, bricks);
            }
        }

        let i = rng.next_bounded_i32(8);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 0, 5, i, bricks, bricks);
        let i = rng.next_bounded_i32(8);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 5, 0, 4, 5, i, bricks, bricks);

        for ix in 0..=4 {
            let j = rng.next_bounded_i32(5);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, ix, 2, 0, ix, 2, j, bricks, bricks);
        }

        for ix in 0..=4 {
            for j in 0..=1 {
                let k = rng.next_bounded_i32(3);
                self.piece
                    .piece
                    .fill_with_outline(chunk, &bb, false, ix, j, 0, ix, j, k, bricks, bricks);
            }
        }
    }
}

impl_piece_base!(BridgeEndPiece);

#[derive(Clone)]
pub struct CorridorExitPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorExitPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -5, -3, 0, 13, 14, 13, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorExit,
            NetherFortressPieceType::CorridorExit,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 5, 3, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;
        let lava = Block::LAVA.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 0, 12, 4, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 12, 13, 12, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 1, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 0, 12, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 11, 4, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 5, 11, 10, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 9, 11, 7, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 9, 0, 7, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 11, 2, 10, 12, 10, bricks, bricks);

        // Default fence fill at entrance (vanilla line 699-701)
        let default_fence = Block::NETHER_BRICK_FENCE.default_state;
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            5,
            8,
            0,
            7,
            8,
            0,
            default_fence,
            default_fence,
        );

        let fence_we = fence_we();
        let fence_ns = fence_ns();

        for i in (1..=11).step_by(2) {
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, i, 10, 0, i, 11, 0, fence_we, fence_we);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, i, 10, 12, i, 11, 12, fence_we, fence_we);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 0, 10, i, 0, 11, i, fence_ns, fence_ns);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 12, 10, i, 12, 11, i, fence_ns, fence_ns);
            self.piece.piece.add_block(chunk, bricks, i, 13, 0, &bb);
            self.piece.piece.add_block(chunk, bricks, i, 13, 12, &bb);
            self.piece.piece.add_block(chunk, bricks, 0, 13, i, &bb);
            self.piece.piece.add_block(chunk, bricks, 12, 13, i, &bb);
            if i != 11 {
                self.piece
                    .piece
                    .add_block(chunk, fence_we, i + 1, 13, 0, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_we, i + 1, 13, 12, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_ns, 0, 13, i + 1, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_ns, 12, 13, i + 1, &bb);
            }
        }

        self.piece.piece.add_block(chunk, fence_ne(), 0, 13, 0, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_se(), 0, 13, 12, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_sw(), 12, 13, 12, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_nw(), 12, 13, 0, &bb);

        for i in (3..=9).step_by(2) {
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                1,
                7,
                i,
                1,
                8,
                i,
                fence_nsw(),
                fence_nsw(),
            );
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                11,
                7,
                i,
                11,
                8,
                i,
                fence_nse(),
                fence_nse(),
            );
        }

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 8, 2, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 4, 12, 2, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 0, 0, 8, 1, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 0, 9, 8, 1, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 4, 3, 1, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 9, 0, 4, 12, 1, 8, bricks, bricks);

        for i in 4..=8 {
            for j in 0..=2 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, 12 - j, &bb);
            }
        }

        for i in 0..=2 {
            for j in 4..=8 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, 12 - i, -1, j, &bb);
            }
        }

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 5, 5, 7, 5, 7, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 1, 6, 6, 4, 6, air, air);
        self.piece.piece.add_block(chunk, bricks, 6, 0, 6, &bb);
        self.piece.piece.add_block(chunk, lava, 6, 5, 6, &bb);
        // TODO: schedule fluid tick for lava at (6, 5, 6) - requires ProtoChunk fluid tick support
    }
}

impl_piece_base!(CorridorExitPiece);

#[derive(Clone)]
pub struct SmallCorridorPiece {
    pub piece: NetherFortressPiece,
}

impl SmallCorridorPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, 0, 0, 5, 7, 5, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressSmallCorridor,
            NetherFortressPieceType::SmallCorridor,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 1, 0, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 5, 4, air, air);

        let fence_ns = fence_ns();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 4, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 1, 0, 4, 1, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 3, 0, 4, 3, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 1, 4, 4, 1, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 3, 4, 4, 3, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 0, 4, 6, 4, bricks, bricks);

        for i in 0..=4 {
            for j in 0..=4 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(SmallCorridorPiece);

#[derive(Clone)]
pub struct CorridorCrossingPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorCrossingPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, 0, 0, 5, 7, 5, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorCrossing,
            NetherFortressPieceType::CorridorCrossing,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 1, 0, true, collector);
        fill_nw_opening(&self.piece.piece, start, random, 0, 1, true, collector);
        fill_se_opening(&self.piece.piece, start, random, 0, 1, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 5, 4, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 4, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 4, 0, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 4, 4, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 0, 4, 6, 4, bricks, bricks);

        for i in 0..=4 {
            for j in 0..=4 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorCrossingPiece);

#[derive(Clone)]
pub struct CorridorRightTurnPiece {
    pub piece: NetherFortressPiece,
    pub contains_chest: bool,
}

impl CorridorRightTurnPiece {
    pub fn create(
        random: &mut RandomGenerator,
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, 0, 0, 5, 7, 5, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorRightTurn,
            NetherFortressPieceType::CorridorRightTurn,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        let contains_chest = random.next_bounded_i32(3) == 0;
        Some(Self {
            piece,
            contains_chest,
        })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_se_opening(&self.piece.piece, start, random, 0, 1, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 5, 4, air, air);

        let fence_we = fence_we();
        let fence_ns = fence_ns();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 1, 0, 4, 1, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 3, 0, 4, 3, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 4, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 2, 4, 4, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 4, 1, 4, 4, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 3, 3, 4, 3, 4, 4, fence_we, fence_we);

        // Chest (33% chance, guarded to prevent double-placement across chunk boundaries)
        if self.contains_chest {
            let pos = self.piece.piece.offset_pos(1, 2, 3);
            if bb.contains_pos(&pos) {
                self.contains_chest = false;
                let chest = make_chest(HorizontalFacing::East);
                self.piece.piece.add_block(chunk, chest, 1, 2, 3, &bb);
                // TODO: Associate loot table LootTables.NETHER_BRIDGE_CHEST
            }
        }

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 0, 4, 6, 4, bricks, bricks);

        for i in 0..=4 {
            for j in 0..=4 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorRightTurnPiece);

#[derive(Clone)]
pub struct CorridorLeftTurnPiece {
    pub piece: NetherFortressPiece,
    pub contains_chest: bool,
}

impl CorridorLeftTurnPiece {
    pub fn create(
        random: &mut RandomGenerator,
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, 0, 0, 5, 7, 5, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorLeftTurn,
            NetherFortressPieceType::CorridorLeftTurn,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        let contains_chest = random.next_bounded_i32(3) == 0;
        Some(Self {
            piece,
            contains_chest,
        })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_nw_opening(&self.piece.piece, start, random, 0, 1, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 4, 1, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 4, 5, 4, air, air);

        let fence_we = fence_we();
        let fence_ns = fence_ns();
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 4, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 1, 4, 4, 1, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 3, 3, 4, 4, 3, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 0, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 4, 3, 5, 4, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 4, 1, 4, 4, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 3, 3, 4, 3, 4, 4, fence_we, fence_we);

        // Chest (33% chance, guarded to prevent double-placement across chunk boundaries)
        if self.contains_chest {
            let pos = self.piece.piece.offset_pos(3, 2, 3);
            if bb.contains_pos(&pos) {
                self.contains_chest = false;
                let chest = make_chest(HorizontalFacing::West);
                self.piece.piece.add_block(chunk, chest, 3, 2, 3, &bb);
                // TODO: Associate loot table LootTables.NETHER_BRIDGE_CHEST
            }
        }

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 0, 4, 6, 4, bricks, bricks);

        for i in 0..=4 {
            for j in 0..=4 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, i, -1, j, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorLeftTurnPiece);

#[derive(Clone)]
pub struct CorridorStairsPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorStairsPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -1, -7, 0, 5, 14, 10, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorStairs,
            NetherFortressPieceType::CorridorStairs,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 1, 0, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;
        let stairs = make_stairs(HorizontalFacing::South);

        let fence_ns = fence_ns();

        for i in 0..=9 {
            let j = 1.max(7 - i);
            let k = (j + 5).min(14 - i).min(13);

            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 0, 0, i, 4, j, i, bricks, bricks);
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                1,
                j + 1,
                i,
                3,
                k - 1,
                i,
                air,
                air,
            );

            if i <= 6 {
                self.piece.piece.add_block(chunk, stairs, 1, j + 1, i, &bb);
                self.piece.piece.add_block(chunk, stairs, 2, j + 1, i, &bb);
                self.piece.piece.add_block(chunk, stairs, 3, j + 1, i, &bb);
            }

            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 0, k, i, 4, k, i, bricks, bricks);
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                0,
                j + 1,
                i,
                0,
                k - 1,
                i,
                bricks,
                bricks,
            );
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                4,
                j + 1,
                i,
                4,
                k - 1,
                i,
                bricks,
                bricks,
            );

            if (i & 1) == 0 {
                self.piece.piece.fill_with_outline(
                    chunk,
                    &bb,
                    false,
                    0,
                    j + 2,
                    i,
                    0,
                    j + 3,
                    i,
                    fence_ns,
                    fence_ns,
                );
                self.piece.piece.fill_with_outline(
                    chunk,
                    &bb,
                    false,
                    4,
                    j + 2,
                    i,
                    4,
                    j + 3,
                    i,
                    fence_ns,
                    fence_ns,
                );
            }

            for m in 0..=4 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, m, -1, i, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorStairsPiece);

#[derive(Clone)]
pub struct CorridorBalconyPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorBalconyPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -3, 0, 0, 9, 7, 9, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorBalcony,
            NetherFortressPieceType::CorridorBalcony,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        let facing = self.piece.piece.facing.unwrap_or(BlockDirection::North);
        let i = if facing == BlockDirection::West || facing == BlockDirection::North {
            5
        } else {
            1
        };

        let inside = random.next_bounded_i32(8) > 0;
        fill_nw_opening(&self.piece.piece, start, random, 0, i, inside, collector);
        let inside = random.next_bounded_i32(8) > 0;
        fill_se_opening(&self.piece.piece, start, random, 0, i, inside, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;

        let fence_ns = fence_ns();
        let fence_we = fence_we();

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 0, 8, 1, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 8, 5, 8, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 6, 0, 8, 6, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 0, 2, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 2, 0, 8, 5, 0, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 0, 1, 4, 0, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 3, 0, 7, 4, 0, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 4, 8, 2, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 1, 4, 2, 2, 4, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 1, 4, 7, 2, 4, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 8, 7, 3, 8, fence_we, fence_we);
        self.piece.piece.add_block(chunk, fence_se(), 0, 3, 8, &bb);
        self.piece.piece.add_block(chunk, fence_sw(), 8, 3, 8, &bb);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 6, 0, 3, 7, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 3, 6, 8, 3, 7, fence_ns, fence_ns);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 4, 0, 5, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 3, 4, 8, 5, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 3, 5, 2, 5, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 6, 3, 5, 7, 5, 5, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 1, 4, 5, 1, 5, 5, fence_we, fence_we);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 7, 4, 5, 7, 5, 5, fence_we, fence_we);

        for i in 0..=5 {
            for j in 0..=8 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, j, -1, i, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorBalconyPiece);

#[derive(Clone)]
pub struct CorridorNetherWartsRoomPiece {
    pub piece: NetherFortressPiece,
}

impl CorridorNetherWartsRoomPiece {
    pub fn create(
        x: i32,
        y: i32,
        z: i32,
        facing: BlockDirection,
        chain_length: u32,
        collector: &StructurePiecesCollector,
    ) -> Option<Self> {
        let bbox = BlockBox::rotated(x, y, z, -5, -3, 0, 13, 14, 13, &facing);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressCorridorNetherWartsRoom,
            NetherFortressPieceType::CorridorNetherWartsRoom,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(facing));
        Some(Self { piece })
    }

    pub fn fill_openings(
        &self,
        start: &mut StartPiece,
        random: &mut RandomGenerator,
        collector: &mut StructurePiecesCollector,
    ) {
        fill_forward_opening(&self.piece.piece, start, random, 5, 3, true, collector);
        fill_forward_opening(&self.piece.piece, start, random, 5, 11, true, collector);
    }

    fn place_blocks(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let bricks = Block::NETHER_BRICKS.default_state;
        let air = Block::AIR.default_state;
        let soul_sand = Block::SOUL_SAND.default_state;
        let nether_wart = Block::NETHER_WART.default_state;

        let fence_we = fence_we();
        let fence_ns = fence_ns();

        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 3, 0, 12, 4, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 12, 13, 12, air, air);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 5, 0, 1, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 11, 5, 0, 12, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 11, 4, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 5, 11, 10, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 9, 11, 7, 12, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 0, 4, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 5, 0, 10, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 9, 0, 7, 12, 1, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 11, 2, 10, 12, 10, bricks, bricks);

        for i in (1..=11).step_by(2) {
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, i, 10, 0, i, 11, 0, fence_we, fence_we);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, i, 10, 12, i, 11, 12, fence_we, fence_we);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 0, 10, i, 0, 11, i, fence_ns, fence_ns);
            self.piece
                .piece
                .fill_with_outline(chunk, &bb, false, 12, 10, i, 12, 11, i, fence_ns, fence_ns);
            self.piece.piece.add_block(chunk, bricks, i, 13, 0, &bb);
            self.piece.piece.add_block(chunk, bricks, i, 13, 12, &bb);
            self.piece.piece.add_block(chunk, bricks, 0, 13, i, &bb);
            self.piece.piece.add_block(chunk, bricks, 12, 13, i, &bb);
            if i != 11 {
                self.piece
                    .piece
                    .add_block(chunk, fence_we, i + 1, 13, 0, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_we, i + 1, 13, 12, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_ns, 0, 13, i + 1, &bb);
                self.piece
                    .piece
                    .add_block(chunk, fence_ns, 12, 13, i + 1, &bb);
            }
        }

        self.piece.piece.add_block(chunk, fence_ne(), 0, 13, 0, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_se(), 0, 13, 12, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_sw(), 12, 13, 12, &bb);
        self.piece
            .piece
            .add_block(chunk, fence_nw(), 12, 13, 0, &bb);

        for i in (3..=9).step_by(2) {
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                1,
                7,
                i,
                1,
                8,
                i,
                fence_nsw(),
                fence_nsw(),
            );
            self.piece.piece.fill_with_outline(
                chunk,
                &bb,
                false,
                11,
                7,
                i,
                11,
                8,
                i,
                fence_nse(),
                fence_nse(),
            );
        }

        // Central staircase (north-facing nether brick stairs, rising from z=4 to z=10)
        let stairs_north = make_stairs(HorizontalFacing::North);
        for j in 0..=6 {
            let k = j + 4;
            for l in 5..=7 {
                self.piece
                    .piece
                    .add_block(chunk, stairs_north, l, 5 + j, k, &bb);
            }
            if (5..=8).contains(&k) {
                self.piece.piece.fill_with_outline(
                    chunk,
                    &bb,
                    false,
                    5,
                    5,
                    k,
                    7,
                    j + 4,
                    k,
                    bricks,
                    bricks,
                );
            } else if (9..=10).contains(&k) {
                self.piece.piece.fill_with_outline(
                    chunk,
                    &bb,
                    false,
                    5,
                    8,
                    k,
                    7,
                    j + 4,
                    k,
                    bricks,
                    bricks,
                );
            }
            if j >= 1 {
                self.piece.piece.fill_with_outline(
                    chunk,
                    &bb,
                    false,
                    5,
                    6 + j,
                    k,
                    7,
                    9 + j,
                    k,
                    air,
                    air,
                );
            }
        }

        // Top stairs at back
        for j in 5..=7 {
            self.piece
                .piece
                .add_block(chunk, stairs_north, j, 12, 11, &bb);
        }

        // Fence accents on staircase platform
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            5,
            6,
            7,
            5,
            7,
            7,
            fence_nse(),
            fence_nse(),
        );
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            7,
            6,
            7,
            7,
            7,
            7,
            fence_nsw(),
            fence_nsw(),
        );

        // Clear air at back ceiling opening
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 5, 13, 12, 7, 13, 12, air, air);

        // Platform blocks around the soul sand gardens
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 2, 3, 5, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 9, 3, 5, 10, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 2, 5, 4, 2, 5, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 9, 5, 2, 10, 5, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 9, 5, 9, 10, 5, 10, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 10, 5, 4, 10, 5, 8, bricks, bricks);

        // Side stairs (east/west facing)
        let stairs_east = make_stairs(HorizontalFacing::East);
        let stairs_west = make_stairs(HorizontalFacing::West);
        self.piece.piece.add_block(chunk, stairs_west, 4, 5, 2, &bb);
        self.piece.piece.add_block(chunk, stairs_west, 4, 5, 3, &bb);
        self.piece.piece.add_block(chunk, stairs_west, 4, 5, 9, &bb);
        self.piece
            .piece
            .add_block(chunk, stairs_west, 4, 5, 10, &bb);
        self.piece.piece.add_block(chunk, stairs_east, 8, 5, 2, &bb);
        self.piece.piece.add_block(chunk, stairs_east, 8, 5, 3, &bb);
        self.piece.piece.add_block(chunk, stairs_east, 8, 5, 9, &bb);
        self.piece
            .piece
            .add_block(chunk, stairs_east, 8, 5, 10, &bb);

        // Soul sand and nether wart
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 3, 4, 4, 4, 4, 8, soul_sand, soul_sand);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 8, 4, 4, 9, 4, 8, soul_sand, soul_sand);
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            3,
            5,
            4,
            4,
            5,
            8,
            nether_wart,
            nether_wart,
        );
        self.piece.piece.fill_with_outline(
            chunk,
            &bb,
            false,
            8,
            5,
            4,
            9,
            5,
            8,
            nether_wart,
            nether_wart,
        );

        // Lower cross floor
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 2, 0, 8, 2, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 2, 4, 12, 2, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 0, 0, 8, 1, 3, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 4, 0, 9, 8, 1, 12, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 0, 0, 4, 3, 1, 8, bricks, bricks);
        self.piece
            .piece
            .fill_with_outline(chunk, &bb, false, 9, 0, 4, 12, 1, 8, bricks, bricks);

        for l in 4..=8 {
            for m in 0..=2 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, l, -1, m, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, l, -1, 12 - m, &bb);
            }
        }
        for l in 0..=2 {
            for m in 4..=8 {
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, l, -1, m, &bb);
                self.piece
                    .piece
                    .fill_downwards(chunk, bricks, 12 - l, -1, m, &bb);
            }
        }
    }
}

impl_piece_base!(CorridorNetherWartsRoomPiece);
