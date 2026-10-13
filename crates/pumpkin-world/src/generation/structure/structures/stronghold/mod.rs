use std::sync::{Arc, Mutex};

use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        DoubleBlockHalf, HorizontalFacing, LeverLikeProperties, OakDoorLikeProperties,
        OakFenceLikeProperties,
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

/// Great reference: <https://minecraft.wiki/w/Stronghold>
#[derive(Deserialize)]
pub struct StrongholdGenerator;

impl StructureGenerator for StrongholdGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let mut collector = StructurePiecesCollector::default();
        let mut tries: i64 = 0;

        loop {
            collector.clear();
            let mut random = crate::generation::structure::structures::create_chunk_random(
                context.seed.wrapping_add(tries),
                context.chunk_x,
                context.chunk_z,
            );
            tries += 1;
            let mut state = StrongholdState::new();

            let start_x = section_coords::section_to_block(context.chunk_x) + 2;
            let start_z = section_coords::section_to_block(context.chunk_z) + 2;

            let (start_piece, start_child) =
                SpiralStaircasePiece::new_start(&mut random, start_x, start_z);
            let start_box = start_child.bbox;

            collector.add_piece(Box::new(start_piece));

            let mut pending_children: Vec<PendingChild> = Vec::new();
            add_children(
                &start_child,
                &start_box,
                &mut state,
                &mut random,
                &mut collector,
                &mut pending_children,
            );

            while !pending_children.is_empty() {
                let pos = random.next_bounded_i32(pending_children.len() as i32) as usize;
                let child = pending_children.remove(pos);

                add_children(
                    &child,
                    &start_box,
                    &mut state,
                    &mut random,
                    &mut collector,
                    &mut pending_children,
                );
            }

            // Shift height
            collector.shift_into(context.sea_level, context.min_y, &mut random, 10);

            if !collector.is_empty() && state.has_portal_room {
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
    SmallCorridor,
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
            Self::SmallCorridor => StructurePieceType::StrongholdSmallCorridor,
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

    const fn can_generate_chained(&self, chain_length: u32) -> bool {
        match self.piece_type {
            StrongholdPieceType::Library => self.can_generate() && chain_length > 4,
            StrongholdPieceType::PortalRoom => self.can_generate() && chain_length > 5,
            _ => self.can_generate(),
        }
    }

    const fn can_generate(&self) -> bool {
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

    #[allow(clippy::too_many_lines)]
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
}

#[derive(Clone, Debug)]
pub struct PendingChild {
    pub piece_type: StrongholdPieceType,
    pub bbox: BlockBox,
    pub orientation: BlockDirection,
    pub depth: u32,
    pub is_source: bool,
    pub lower_left: bool,
    pub upper_left: bool,
    pub lower_right: bool,
    pub upper_right: bool,
    pub left_child: bool,
    pub right_child: bool,
}

impl PendingChild {
    #[must_use]
    pub const fn simple(
        piece_type: StrongholdPieceType,
        bbox: BlockBox,
        orientation: BlockDirection,
        depth: u32,
    ) -> Self {
        Self {
            piece_type,
            bbox,
            orientation,
            depth,
            is_source: false,
            lower_left: false,
            upper_left: false,
            lower_right: false,
            upper_right: false,
            left_child: false,
            right_child: false,
        }
    }
}

struct StrongholdState {
    weights: Vec<PieceWeight>,
    previous_piece: Option<StrongholdPieceType>,
    imposed_piece: Option<StrongholdPieceType>,
    has_portal_room: bool,
}

impl StrongholdState {
    fn new() -> Self {
        Self {
            weights: get_initial_weights(),
            previous_piece: None,
            imposed_piece: None,
            has_portal_room: false,
        }
    }
}

#[expect(clippy::too_many_lines)]
fn add_children(
    child: &PendingChild,
    start_box: &BlockBox,
    state: &mut StrongholdState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
) {
    match child.piece_type {
        StrongholdPieceType::ChestCorridor
        | StrongholdPieceType::PrisonHall
        | StrongholdPieceType::Stairs => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                1,
            );
        }
        StrongholdPieceType::FiveWayCrossing => {
            let mut z_off_a = 3;
            let mut z_off_b = 5;
            if child.orientation == BlockDirection::West
                || child.orientation == BlockDirection::North
            {
                z_off_a = 8 - z_off_a;
                z_off_b = 8 - z_off_b;
            }

            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                5,
                1,
            );
            if child.lower_left {
                generate_child_left(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    z_off_a,
                    1,
                );
            }
            if child.upper_left {
                generate_child_left(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    z_off_b,
                    7,
                );
            }
            if child.lower_right {
                generate_child_right(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    z_off_a,
                    1,
                );
            }
            if child.upper_right {
                generate_child_right(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    z_off_b,
                    7,
                );
            }
        }
        StrongholdPieceType::LeftTurn => {
            if child.orientation != BlockDirection::North
                && child.orientation != BlockDirection::East
            {
                generate_child_right(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    1,
                );
            } else {
                generate_child_left(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    1,
                );
            }
        }
        StrongholdPieceType::Library | StrongholdPieceType::SmallCorridor => {}
        StrongholdPieceType::PortalRoom => {
            state.has_portal_room = true;
        }
        StrongholdPieceType::RightTurn => {
            if child.orientation != BlockDirection::North
                && child.orientation != BlockDirection::East
            {
                generate_child_left(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    1,
                );
            } else {
                generate_child_right(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    1,
                );
            }
        }
        StrongholdPieceType::SquareRoom => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                4,
                1,
            );
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                4,
            );
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                4,
            );
        }
        StrongholdPieceType::SpiralStaircase => {
            if child.is_source {
                state.imposed_piece = Some(StrongholdPieceType::FiveWayCrossing);
            }
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                1,
            );
        }
        StrongholdPieceType::Corridor => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                1,
            );
            if child.left_child {
                generate_child_left(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    2,
                );
            }
            if child.right_child {
                generate_child_right(
                    child,
                    start_box,
                    state,
                    random,
                    collector,
                    pending_children,
                    1,
                    2,
                );
            }
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn generate_child_forward(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut StrongholdState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    x_off: i32,
    y_off: i32,
) {
    let (nx, ny, nz) = match parent.orientation {
        BlockDirection::North => (
            parent.bbox.min.x + x_off,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z - 1,
        ),
        BlockDirection::South => (
            parent.bbox.min.x + x_off,
            parent.bbox.min.y + y_off,
            parent.bbox.max.z + 1,
        ),
        BlockDirection::West => (
            parent.bbox.min.x - 1,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z + x_off,
        ),
        BlockDirection::East => (
            parent.bbox.max.x + 1,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z + x_off,
        ),
        _ => return,
    };
    generate_and_add_piece(
        start_box,
        state,
        random,
        collector,
        pending_children,
        nx,
        ny,
        nz,
        parent.orientation,
        parent.depth,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_child_left(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut StrongholdState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    y_off: i32,
    z_off: i32,
) {
    let (nx, ny, nz, next_facing) = match parent.orientation {
        BlockDirection::North | BlockDirection::South => (
            parent.bbox.min.x - 1,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z + z_off,
            BlockDirection::West,
        ),
        BlockDirection::West | BlockDirection::East => (
            parent.bbox.min.x + z_off,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z - 1,
            BlockDirection::North,
        ),
        _ => return,
    };
    generate_and_add_piece(
        start_box,
        state,
        random,
        collector,
        pending_children,
        nx,
        ny,
        nz,
        next_facing,
        parent.depth,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_child_right(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut StrongholdState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    y_off: i32,
    z_off: i32,
) {
    let (nx, ny, nz, next_facing) = match parent.orientation {
        BlockDirection::North | BlockDirection::South => (
            parent.bbox.max.x + 1,
            parent.bbox.min.y + y_off,
            parent.bbox.min.z + z_off,
            BlockDirection::East,
        ),
        BlockDirection::West | BlockDirection::East => (
            parent.bbox.min.x + z_off,
            parent.bbox.min.y + y_off,
            parent.bbox.max.z + 1,
            BlockDirection::South,
        ),
        _ => return,
    };
    generate_and_add_piece(
        start_box,
        state,
        random,
        collector,
        pending_children,
        nx,
        ny,
        nz,
        next_facing,
        parent.depth,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_and_add_piece(
    start_box: &BlockBox,
    state: &mut StrongholdState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
) {
    if depth > 50 {
        return;
    }
    if (foot_x - start_box.min.x).abs() <= 112 && (foot_z - start_box.min.z).abs() <= 112 {
        let (piece_opt, child_opt) = generate_piece_from_small_door(
            state,
            collector,
            random,
            foot_x,
            foot_y,
            foot_z,
            direction,
            depth + 1,
        );
        if let Some(piece) = piece_opt {
            collector.add_piece(piece);
            if let Some(child) = child_opt {
                pending_children.push(child);
            }
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn generate_piece_from_small_door(
    state: &mut StrongholdState,
    collector: &mut StructurePiecesCollector,
    random: &mut impl RandomImpl,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
) -> (Option<Box<dyn StructurePieceBase>>, Option<PendingChild>) {
    let Some(total_weight) = update_piece_weight(&state.weights) else {
        return (None, None);
    };

    if let Some(imposed) = state.imposed_piece.take()
        && let Some((piece, child)) = create_piece_and_child(
            imposed, collector, random, foot_x, foot_y, foot_z, direction, depth,
        )
    {
        return (Some(piece), Some(child));
    }

    let mut num_attempts = 0;
    while num_attempts < 5 {
        num_attempts += 1;
        let mut weight_selection = random.next_bounded_i32(total_weight);

        let mut i = 0;
        while i < state.weights.len() {
            weight_selection -= state.weights[i].weight;
            if weight_selection < 0 {
                if !state.weights[i].can_generate_chained(depth)
                    || Some(state.weights[i].piece_type) == state.previous_piece
                {
                    break;
                }

                let piece_type = state.weights[i].piece_type;
                if let Some((piece, child)) = create_piece_and_child(
                    piece_type, collector, random, foot_x, foot_y, foot_z, direction, depth,
                ) {
                    state.weights[i].generated_count += 1;
                    state.previous_piece = Some(piece_type);
                    if !state.weights[i].can_generate() {
                        state.weights.remove(i);
                    }
                    return (Some(piece), Some(child));
                }
            }
            i += 1;
        }
    }

    // Fallback: FillerCorridor (SmallCorridorPiece)
    if let Some(box_) =
        SmallCorridorPiece::create_box(collector, foot_x, foot_y, foot_z, &direction)
        && box_.min.y > 1
    {
        let piece = Box::new(SmallCorridorPiece::new(depth, box_, direction));
        let child =
            PendingChild::simple(StrongholdPieceType::SmallCorridor, box_, direction, depth);
        return (Some(piece), Some(child));
    }

    (None, None)
}

fn update_piece_weight(weights: &[PieceWeight]) -> Option<i32> {
    let mut has_any_pieces = false;
    let mut total_weight = 0;

    for piece in weights {
        if piece.limit > 0 && piece.generated_count < piece.limit {
            has_any_pieces = true;
        }
        total_weight += piece.weight;
    }

    has_any_pieces.then_some(total_weight)
}

#[expect(clippy::too_many_arguments)]
fn create_piece_and_child(
    piece_type: StrongholdPieceType,
    collector: &mut StructurePiecesCollector,
    random: &mut impl RandomImpl,
    x: i32,
    y: i32,
    z: i32,
    orientation: BlockDirection,
    depth: u32,
) -> Option<(Box<dyn StructurePieceBase>, PendingChild)> {
    match piece_type {
        StrongholdPieceType::FiveWayCrossing => {
            FiveWayCrossingPiece::create(collector, random, x, y, z, orientation, depth)
        }
        StrongholdPieceType::Corridor => {
            CorridorPiece::create(collector, random, x, y, z, orientation, depth)
        }
        StrongholdPieceType::SquareRoom => {
            let piece = SquareRoomPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::SquareRoom, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::PortalRoom => {
            let piece = PortalRoomPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::PortalRoom, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::SpiralStaircase => {
            let piece =
                SpiralStaircasePiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(
                    StrongholdPieceType::SpiralStaircase,
                    bbox,
                    orientation,
                    depth,
                ),
            ))
        }
        StrongholdPieceType::PrisonHall => {
            let piece = PrisonHallPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::PrisonHall, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::LeftTurn => {
            let piece = LeftTurnPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::LeftTurn, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::RightTurn => {
            let piece = RightTurnPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::RightTurn, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::Stairs => {
            let piece = StairsPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::Stairs, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::ChestCorridor => {
            let piece = ChestCorridorPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::ChestCorridor, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::Library => {
            let piece = LibraryPiece::create(collector, random, x, y, z, orientation, depth)?;
            let bbox = piece.bounding_box();
            Some((
                piece,
                PendingChild::simple(StrongholdPieceType::Library, bbox, orientation, depth),
            ))
        }
        StrongholdPieceType::SmallCorridor => None,
    }
}
