use std::sync::{Arc, Mutex};

use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        BlockProperties, DoubleBlockHalf, HorizontalFacing, LeverLikeProperties,
        OakDoorLikeProperties, OakFenceLikeProperties,
    },
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
                BlockRandomizer, StructureGenerator, StructureGeneratorContext, StructurePiece,
                StructurePieceBase, StructurePiecesCollector, StructurePosition,
                stronghold::{
                    chest_corridor::ChestCorridorPiece,
                    corridor::{CorridorPiece, SmallCorridorPiece},
                    five_way_crossing::FiveWayCrossingPiece,
                    left_turn::LeftTurnPiece,
                    library::LibraryPiece,
                    portal_room::PortalRoomPiece,
                    prison_hall::PrisonHallPiece,
                    right_turn::RightTurnPiece,
                    spiral_staircase::SpiralStaircasePiece,
                    square_room::SquareRoomPiece,
                    stairs::StairsPiece,
                },
            },
        },
    },
};

pub mod chest_corridor;
pub mod corridor;
pub mod five_way_crossing;
pub mod left_turn;
pub mod library;
pub mod portal_room;
pub mod prison_hall;
pub mod right_turn;
pub mod spiral_staircase;
pub mod square_room;
pub mod stairs;

/// Great reference: https://minecraft.wiki/w/Stronghold
#[derive(Deserialize)]
pub struct StrongholdGenerator;

impl StructureGenerator for StrongholdGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let mut collector = StructurePiecesCollector::default();
        let mut random = context.random;

        loop {
            collector.clear();
            let mut weights = get_initial_weights();
            let mut last_piece_type: Option<StrongholdPieceType> = None;
            let mut has_portal_room = false;

            // let attempt_seed = get_carver_seed(
            //     &mut context.random,
            //     context.seed as u64,
            //     context.chunk_x,
            //     context.chunk_z,
            // );

            let start_x = section_coords::section_to_block(context.chunk_x) + 2;
            let start_z = section_coords::section_to_block(context.chunk_z) + 2;

            let start_piece = SpiralStaircasePiece::new_start(&mut random, start_x, start_z);
            let start_box: Box<dyn StructurePieceBase> = Box::new(start_piece.clone());

            // In Vanilla, pieces_to_process is 'start.pieces'
            let mut pieces_to_process: Vec<Box<dyn StructurePieceBase>> = Vec::new();

            collector.add_piece(start_box);

            // Initial Fill
            start_piece.fill_openings(
                &start_piece.piece.piece,
                &mut random,
                &mut weights,
                &mut last_piece_type,
                &mut has_portal_room,
                &mut collector,
                &mut pieces_to_process,
            );

            // Growth Loop
            while !pieces_to_process.is_empty() {
                let idx = random.next_bounded_i32(pieces_to_process.len() as i32) as usize;
                let piece = pieces_to_process.remove(idx);
                piece.fill_openings(
                    &start_piece.piece.piece,
                    &mut random,
                    &mut weights,
                    &mut last_piece_type,
                    &mut has_portal_room,
                    &mut collector,
                    &mut pieces_to_process,
                );
            }

            // Shift height
            collector.shift_into(context.sea_level, context.min_y, &mut random, 10);

            if !collector.is_empty() && has_portal_room {
                break;
            }
        }

        Some(StructurePosition {
            start_pos: BlockPos::new(
                section_coords::section_to_block(context.chunk_x),
                0,
                section_coords::section_to_block(context.chunk_z),
            ),
            collector: Arc::new(Mutex::new(collector)),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntranceType {
    Opening = 0,
    WoodDoor = 1,
    Grates = 2,
    IronDoor = 3,
}

impl EntranceType {
    pub fn get_random(random: &mut impl RandomImpl) -> Self {
        match random.next_bounded_i32(5) {
            2 => Self::WoodDoor,
            3 => Self::Grates,
            4 => Self::IronDoor,
            _ => Self::Opening,
        }
    }
}

struct StoneBrickRandomizer;

impl BlockRandomizer for StoneBrickRandomizer {
    fn get_block(&self, rng: &mut RandomGenerator, is_border: bool) -> &BlockState {
        if !is_border {
            return Block::AIR.default_state;
        }

        let roll = rng.next_f32();
        if roll < 0.2 {
            Block::CRACKED_STONE_BRICKS.default_state
        } else if roll < 0.5 {
            Block::MOSSY_STONE_BRICKS.default_state
        } else {
            Block::STONE_BRICKS.default_state
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrongholdPieceType {
    Corridor,
    PrisonHall,
    LeftTurn,
    RightTurn,
    SquareRoom,
    Stairs,
    SpiralStaircase,
    FiveWayCrossing,
    ChestCorridor,
    Library,
    PortalRoom,
}
impl StrongholdPieceType {
    #[must_use]
    pub const fn as_structure_type(&self) -> StructurePieceType {
        match self {
            Self::Corridor => StructurePieceType::StrongholdCorridor,
            Self::PrisonHall => StructurePieceType::StrongholdPrisonHall,
            Self::LeftTurn => StructurePieceType::StrongholdLeftTurn,
            Self::RightTurn => StructurePieceType::StrongholdRightTurn,
            Self::SquareRoom => StructurePieceType::StrongholdSquareRoom,
            Self::Stairs => StructurePieceType::StrongholdStairs,
            Self::SpiralStaircase => StructurePieceType::StrongholdSpiralStaircase,
            Self::FiveWayCrossing => StructurePieceType::StrongholdFiveWayCrossing,
            Self::ChestCorridor => StructurePieceType::StrongholdChestCorridor,
            Self::Library => StructurePieceType::StrongholdLibrary,
            Self::PortalRoom => StructurePieceType::StrongholdPortalRoom,
        }
    }
}

#[derive(Clone)]
pub struct PieceWeight {
    pub piece_type: StrongholdPieceType,
    pub weight: i32,
    pub limit: u32,
    pub generated_count: u32, // Added to track per-generation counts
}

impl PieceWeight {
    const fn new(piece_type: StrongholdPieceType, weight: i32, limit: u32) -> Self {
        Self {
            piece_type,
            weight,
            limit,
            generated_count: 0,
        }
    }

    fn can_generate_chained(&self, chain_length: u32) -> bool {
        match self.piece_type {
            StrongholdPieceType::Library => self.can_generate() && chain_length > 4,
            StrongholdPieceType::PortalRoom => self.can_generate() && chain_length > 5,
            _ => self.can_generate(),
        }
    }

    fn can_generate(&self) -> bool {
        self.limit == 0 || self.generated_count < self.limit
    }
}

fn get_initial_weights() -> Vec<PieceWeight> {
    vec![
        PieceWeight::new(StrongholdPieceType::Corridor, 40, 0),
        PieceWeight::new(StrongholdPieceType::PrisonHall, 5, 5),
        PieceWeight::new(StrongholdPieceType::LeftTurn, 20, 0),
        PieceWeight::new(StrongholdPieceType::RightTurn, 20, 0),
        PieceWeight::new(StrongholdPieceType::SquareRoom, 10, 6),
        PieceWeight::new(StrongholdPieceType::Stairs, 5, 5),
        PieceWeight::new(StrongholdPieceType::SpiralStaircase, 5, 5),
        PieceWeight::new(StrongholdPieceType::FiveWayCrossing, 5, 4),
        PieceWeight::new(StrongholdPieceType::ChestCorridor, 5, 4),
        PieceWeight::new(StrongholdPieceType::Library, 10, 2),
        PieceWeight::new(StrongholdPieceType::PortalRoom, 20, 1),
    ]
}

#[derive(Clone)]
pub struct StrongholdPiece {
    pub piece: StructurePiece,
    pub entry_door: EntranceType,
}

impl StrongholdPiece {
    #[must_use]
    pub const fn new(r#type: StructurePieceType, chain_length: u32, bbox: BlockBox) -> Self {
        Self {
            piece: StructurePiece::new(r#type, bbox, chain_length),
            entry_door: EntranceType::Opening,
        }
    }
    const fn is_in_bounds(bb: &BlockBox) -> bool {
        bb.min.y > 10
    }

    fn generate_entrance(
        &self,
        chunk: &mut ProtoChunk,
        box_limit: &BlockBox,
        entrance_type: EntranceType,
        x: i32,
        y: i32,
        z: i32,
    ) {
        match entrance_type {
            EntranceType::Opening => {
                let air = Block::AIR.default_state;
                self.piece.fill_with_outline(
                    chunk,
                    box_limit,
                    false,
                    x,
                    y,
                    z,
                    x + 2,
                    y + 2,
                    z,
                    air,
                    air,
                );
            }
            EntranceType::WoodDoor => {
                let stone = Block::STONE_BRICKS.default_state;
                let door_lower = Block::OAK_DOOR.default_state;
                let mut door_upper = OakDoorLikeProperties::default(&Block::OAK_DOOR);
                door_upper.half = DoubleBlockHalf::Upper;
                let door_upper = BlockState::from_id(door_upper.to_state_id(&Block::OAK_DOOR));

                // Frame
                self.piece.add_block(chunk, stone, x, y, z, box_limit);
                self.piece.add_block(chunk, stone, x, y + 1, z, box_limit);
                self.piece.add_block(chunk, stone, x, y + 2, z, box_limit);
                self.piece
                    .add_block(chunk, stone, x + 1, y + 2, z, box_limit);
                self.piece
                    .add_block(chunk, stone, x + 2, y + 2, z, box_limit);
                self.piece
                    .add_block(chunk, stone, x + 2, y + 1, z, box_limit);
                self.piece.add_block(chunk, stone, x + 2, y, z, box_limit);

                // Door
                self.piece
                    .add_block(chunk, door_lower, x + 1, y, z, box_limit);
                self.piece
                    .add_block(chunk, door_upper, x + 1, y + 1, z, box_limit);
            }
            EntranceType::Grates => {
                let air = Block::CAVE_AIR.default_state;
                let mut props = OakFenceLikeProperties::default(&Block::IRON_BARS);

                // North-South facing bars
                props.west = true;
                let bar_w = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));
                props.west = false;
                props.east = true;
                let bar_e = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));
                props.west = true;
                let bar_ew = BlockState::from_id(props.to_state_id(&Block::IRON_BARS));

                self.piece.add_block(chunk, air, x + 1, y, z, box_limit);
                self.piece.add_block(chunk, air, x + 1, y + 1, z, box_limit);

                // Left side (West connection)
                self.piece.add_block(chunk, bar_w, x, y, z, box_limit);
                self.piece.add_block(chunk, bar_w, x, y + 1, z, box_limit);

                // // Top beam (Connected East-West)
                self.piece.add_block(chunk, bar_ew, x, y + 2, z, box_limit);
                self.piece
                    .add_block(chunk, bar_ew, x + 1, y + 2, z, box_limit);
                self.piece
                    .add_block(chunk, bar_ew, x + 2, y + 2, z, box_limit);

                // // Right side (East connection)
                self.piece
                    .add_block(chunk, bar_e, x + 2, y + 1, z, box_limit);
                self.piece.add_block(chunk, bar_e, x + 2, y, z, box_limit);
            }
            EntranceType::IronDoor => {
                let stone = Block::STONE_BRICKS.default_state;
                let door_lower = Block::IRON_DOOR.default_state;
                let mut door_upper = OakDoorLikeProperties::default(&Block::IRON_DOOR);
                door_upper.half = DoubleBlockHalf::Upper;
                let door_upper = BlockState::from_id(door_upper.to_state_id(&Block::IRON_DOOR));
                let mut props = LeverLikeProperties::default(&Block::STONE_BUTTON);
                props.facing = HorizontalFacing::North;
                let button_n = BlockState::from_id(props.to_state_id(&Block::STONE_BUTTON));
                props.facing = HorizontalFacing::South;
                let button_s = BlockState::from_id(props.to_state_id(&Block::STONE_BUTTON));

                // Frame
                self.piece.fill_with_outline(
                    chunk,
                    box_limit,
                    false,
                    x,
                    y,
                    z,
                    x,
                    y + 2,
                    z,
                    stone,
                    stone,
                );
                self.piece
                    .add_block(chunk, stone, x + 1, y + 2, z, box_limit);
                self.piece.fill_with_outline(
                    chunk,
                    box_limit,
                    false,
                    x + 2,
                    y,
                    z,
                    x + 2,
                    y + 2,
                    z,
                    stone,
                    stone,
                );

                // Door
                self.piece
                    .add_block(chunk, door_lower, x + 1, y, z, box_limit);
                self.piece
                    .add_block(chunk, door_upper, x + 1, y + 1, z, box_limit);

                // Buttons
                self.piece
                    .add_block(chunk, button_n, x + 2, y + 1, z + 1, box_limit);
                self.piece
                    .add_block(chunk, button_s, x + 2, y + 1, z - 1, box_limit);
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_forward_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        left_right_offset: i32,
        height_offset: i32,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
        piece: Option<StrongholdPieceType>,
    ) {
        if let Some(facing) = self.piece.facing {
            let bounding_box = self.piece.bounding_box;
            let (nx, ny, nz) = match facing {
                BlockDirection::North => (
                    bounding_box.min.x + left_right_offset,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z - 1,
                ),
                BlockDirection::South => (
                    bounding_box.min.x + left_right_offset,
                    bounding_box.min.y + height_offset,
                    bounding_box.max.z + 1,
                ),
                BlockDirection::West => (
                    bounding_box.min.x - 1,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z + left_right_offset,
                ),
                BlockDirection::East => (
                    bounding_box.max.x + 1,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z + left_right_offset,
                ),
                _ => return,
            };

            if let Some(next) = Self::piece_generator(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                nx,
                ny,
                nz,
                facing,
                self.piece.chain_length,
                piece,
            ) {
                pieces_to_process.push(next);
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_nw_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        height_offset: i32,
        left_right_offset: i32,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        if let Some(facing) = &self.piece.facing {
            let bounding_box = self.piece.bounding_box;

            // Java switch(direction) logic translated to world coordinates
            let (nx, ny, nz, next_facing) = match facing {
                BlockDirection::North | BlockDirection::South => (
                    bounding_box.min.x - 1,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z + left_right_offset,
                    BlockDirection::West,
                ),
                BlockDirection::West | BlockDirection::East => (
                    bounding_box.min.x + left_right_offset,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z - 1,
                    BlockDirection::North,
                ),
                _ => return,
            };

            if let Some(next) = Self::piece_generator(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                nx,
                ny,
                nz,
                next_facing,
                self.piece.chain_length,
                None,
            ) {
                pieces_to_process.push(next);
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn fill_se_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        height_offset: i32,
        left_right_offset: i32,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        if let Some(facing) = &self.piece.facing {
            let bounding_box = self.piece.bounding_box;

            let (nx, ny, nz, next_facing) = match facing {
                BlockDirection::North | BlockDirection::South => (
                    bounding_box.max.x + 1,
                    bounding_box.min.y + height_offset,
                    bounding_box.min.z + left_right_offset,
                    BlockDirection::East,
                ),
                BlockDirection::West | BlockDirection::East => (
                    bounding_box.min.x + left_right_offset,
                    bounding_box.min.y + height_offset,
                    bounding_box.max.z + 1,
                    BlockDirection::South,
                ),
                _ => return,
            };

            if let Some(next) = Self::piece_generator(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                nx,
                ny,
                nz,
                next_facing,
                self.piece.chain_length,
                None,
            ) {
                pieces_to_process.push(next);
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn piece_generator(
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
        piece_type: Option<StrongholdPieceType>,
    ) -> Option<Box<dyn StructurePieceBase>> {
        // Vanilla limit is 50
        if chain_length > 50 {
            return None;
        }

        // Distance check from start (112 blocks)
        let start_box = start.bounding_box;
        if (x - start_box.min.x).abs() > 112 || (z - start_box.min.z).abs() > 112 {
            return None;
        }

        let next_piece = if let Some(p_type) = piece_type {
            Self::create_piece(
                p_type,
                collector,
                random,
                x,
                y,
                z,
                orientation,
                chain_length + 1,
            )
        } else {
            Self::pick_piece(
                collector,
                random,
                weights,
                last_piece_type,
                x,
                y,
                z,
                orientation,
                chain_length + 1,
            )
        };

        if let Some(p) = next_piece {
            collector.add_piece(p.clone_box());
            return Some(p);
        }

        None
    }

    fn check_remaining_pieces(weights: &Vec<PieceWeight>, total_weight: &mut i32) -> bool {
        let mut can_generate = false;
        *total_weight = 0;

        for piece_data in weights {
            // If at least one piece with a limit hasn't reached it, we can keep going
            if piece_data.limit > 0 && piece_data.generated_count < piece_data.limit {
                can_generate = true;
            }
            *total_weight += piece_data.weight;
        }

        can_generate
    }

    #[expect(clippy::too_many_arguments)]
    fn pick_piece(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let mut total_weight = 0;

        if !Self::check_remaining_pieces(weights, &mut total_weight) {
            return None;
        }

        let mut attempt = 0;
        while attempt < 5 {
            attempt += 1;
            let mut j = random.next_bounded_i32(total_weight);

            for i in 0..weights.len() {
                j -= weights[i].weight;
                if j < 0 {
                    // Check if this piece can generate at this chain_length
                    if !weights[i].can_generate_chained(chain_length)
                        || Some(weights[i].piece_type) == *last_piece_type
                    {
                        break;
                    }

                    if let Some(p) = Self::create_piece(
                        weights[i].piece_type,
                        collector,
                        random,
                        x,
                        y,
                        z,
                        orientation,
                        chain_length,
                    ) {
                        weights[i].generated_count += 1;
                        *last_piece_type = Some(weights[i].piece_type);

                        if !weights[i].can_generate() {
                            weights.remove(i);
                        }

                        return Some(p);
                    }
                }
            }
        }

        // Fallback: Small Corridor
        if let Some(bbox) = SmallCorridorPiece::create_box(collector, x, y, z, &orientation)
            && bbox.min.y > 1
        {
            return Some(Box::new(SmallCorridorPiece::new(
                chain_length,
                bbox,
                orientation,
            )));
        }

        None
    }

    #[expect(clippy::too_many_arguments)]
    fn create_piece(
        piece_type: StrongholdPieceType,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        match piece_type {
            StrongholdPieceType::FiveWayCrossing => {
                FiveWayCrossingPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::Corridor => {
                CorridorPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::SquareRoom => {
                SquareRoomPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::PortalRoom => {
                PortalRoomPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::SpiralStaircase => {
                SpiralStaircasePiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::PrisonHall => {
                PrisonHallPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::LeftTurn => {
                LeftTurnPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::RightTurn => {
                RightTurnPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::Stairs => {
                StairsPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::ChestCorridor => {
                ChestCorridorPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            StrongholdPieceType::Library => {
                LibraryPiece::create(collector, random, x, y, z, orientation, chain_length)
            }
        }
    }
}
