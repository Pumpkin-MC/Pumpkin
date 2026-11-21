use std::sync::{Arc, Mutex};

use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomImpl,
};
use serde::Deserialize;

use crate::generation::{
    section_coords,
    structure::structures::{
        StructureGenerator, StructureGeneratorContext, StructurePieceBase,
        StructurePiecesCollector, StructurePosition,
        nether_fortress::bridge_crossing::BridgeCrossingPiece,
    },
};

pub mod bridge;
pub mod bridge_crossing;
pub mod bridge_end;
pub mod bridge_platform;
pub mod bridge_small_crossing;
pub mod bridge_stairs;
pub mod corridor_balcony;
pub mod corridor_crossing;
pub mod corridor_exit;
pub mod corridor_left_turn;
pub mod corridor_nether_warts_room;
pub mod corridor_right_turn;
pub mod corridor_stairs;
pub mod small_corridor;

/// Great reference: <https://minecraft.wiki/w/Nether_fortress>
#[derive(Deserialize)]
pub struct NetherFortressGenerator;

impl StructureGenerator for NetherFortressGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let mut collector = StructurePiecesCollector::default();
        let mut random = context.random;

        let start_x = section_coords::section_to_block(context.chunk_x) + 2;
        let start_z = section_coords::section_to_block(context.chunk_z) + 2;

        let start_piece = BridgeCrossingPiece::new_start(&mut random, start_x, start_z);
        let start_box = start_piece.bounding_box();
        let start_orientation = start_piece
            .piece
            .piece
            .facing
            .unwrap_or(BlockDirection::North);

        let start_child = PendingChild {
            piece_type: NetherFortressPieceType::BridgeCrossing,
            bbox: start_box,
            orientation: start_orientation,
            depth: 0,
        };

        collector.add_piece(Box::new(start_piece));

        let mut state = FortressState {
            previous_piece: None,
            available_bridge_pieces: get_bridge_piece_weights(),
            available_castle_pieces: get_corridor_piece_weights(),
        };

        let mut pending_children = Vec::new();
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

        collector.move_inside_heights(&mut random, 48, 70);

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

/// Weights for the bridge (exterior) pieces.
#[derive(Clone, Debug)]
pub struct PieceWeight {
    pub piece_type: NetherFortressPieceType,
    pub weight: i32,
    pub limit: u32,
    pub generated_count: u32,
    pub repeatable: bool,
}

impl PieceWeight {
    const fn new(
        piece_type: NetherFortressPieceType,
        weight: i32,
        limit: u32,
        repeatable: bool,
    ) -> Self {
        Self {
            piece_type,
            weight,
            limit,
            generated_count: 0,
            repeatable,
        }
    }

    #[must_use]
    pub const fn can_generate(&self) -> bool {
        self.limit == 0 || self.generated_count < self.limit
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NetherFortressPieceType {
    // Bridge pieces
    Bridge,
    BridgeCrossing,
    BridgeSmallCrossing,
    BridgeStairs,
    BridgePlatform,
    CorridorExit,
    // Corridor pieces
    SmallCorridor,
    CorridorCrossing,
    CorridorRightTurn,
    CorridorLeftTurn,
    CorridorStairs,
    CorridorBalcony,
    CorridorNetherWartsRoom,
    // Terminator
    BridgeEnd,
}

#[must_use]
pub fn get_bridge_piece_weights() -> Vec<PieceWeight> {
    vec![
        PieceWeight::new(NetherFortressPieceType::Bridge, 30, 0, true),
        PieceWeight::new(NetherFortressPieceType::BridgeCrossing, 10, 4, false),
        PieceWeight::new(NetherFortressPieceType::BridgeSmallCrossing, 10, 4, false),
        PieceWeight::new(NetherFortressPieceType::BridgeStairs, 10, 3, false),
        PieceWeight::new(NetherFortressPieceType::BridgePlatform, 5, 2, false),
        PieceWeight::new(NetherFortressPieceType::CorridorExit, 5, 1, false),
    ]
}

#[must_use]
pub fn get_corridor_piece_weights() -> Vec<PieceWeight> {
    vec![
        PieceWeight::new(NetherFortressPieceType::SmallCorridor, 25, 0, true),
        PieceWeight::new(NetherFortressPieceType::CorridorCrossing, 15, 5, false),
        PieceWeight::new(NetherFortressPieceType::CorridorRightTurn, 5, 10, false),
        PieceWeight::new(NetherFortressPieceType::CorridorLeftTurn, 5, 10, false),
        PieceWeight::new(NetherFortressPieceType::CorridorStairs, 10, 3, true),
        PieceWeight::new(NetherFortressPieceType::CorridorBalcony, 7, 2, false),
        PieceWeight::new(
            NetherFortressPieceType::CorridorNetherWartsRoom,
            5,
            2,
            false,
        ),
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct PendingChild {
    pub piece_type: NetherFortressPieceType,
    pub bbox: BlockBox,
    pub orientation: BlockDirection,
    pub depth: u32,
}

struct FortressState {
    previous_piece: Option<NetherFortressPieceType>,
    available_bridge_pieces: Vec<PieceWeight>,
    available_castle_pieces: Vec<PieceWeight>,
}

#[expect(clippy::too_many_lines)]
fn add_children(
    child: &PendingChild,
    start_box: &BlockBox,
    state: &mut FortressState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
) {
    match child.piece_type {
        NetherFortressPieceType::BridgeCrossing => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                8,
                3,
                false,
            );
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                3,
                8,
                false,
            );
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                3,
                8,
                false,
            );
        }
        NetherFortressPieceType::Bridge => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                3,
                false,
            );
        }
        NetherFortressPieceType::BridgeSmallCrossing => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                2,
                0,
                false,
            );
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                2,
                false,
            );
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                2,
                false,
            );
        }
        NetherFortressPieceType::BridgeStairs => {
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                6,
                2,
                false,
            );
        }
        NetherFortressPieceType::BridgePlatform | NetherFortressPieceType::BridgeEnd => {}
        NetherFortressPieceType::CorridorExit => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                5,
                3,
                true,
            );
        }
        NetherFortressPieceType::SmallCorridor | NetherFortressPieceType::CorridorStairs => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                0,
                true,
            );
        }
        NetherFortressPieceType::CorridorCrossing => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                1,
                0,
                true,
            );
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                1,
                true,
            );
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                1,
                true,
            );
        }
        NetherFortressPieceType::CorridorRightTurn => {
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                1,
                true,
            );
        }
        NetherFortressPieceType::CorridorLeftTurn => {
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                1,
                true,
            );
        }
        NetherFortressPieceType::CorridorBalcony => {
            let z_off = match child.orientation {
                BlockDirection::West | BlockDirection::North => 5,
                _ => 1,
            };
            let is_castle_left = random.next_bounded_i32(8) > 0;
            generate_child_left(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                z_off,
                is_castle_left,
            );
            let is_castle_right = random.next_bounded_i32(8) > 0;
            generate_child_right(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                0,
                z_off,
                is_castle_right,
            );
        }
        NetherFortressPieceType::CorridorNetherWartsRoom => {
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                5,
                3,
                true,
            );
            generate_child_forward(
                child,
                start_box,
                state,
                random,
                collector,
                pending_children,
                5,
                11,
                true,
            );
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn generate_child_forward(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut FortressState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    x_off: i32,
    y_off: i32,
    is_castle: bool,
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
        is_castle,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_child_left(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut FortressState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    y_off: i32,
    z_off: i32,
    is_castle: bool,
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
        is_castle,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_child_right(
    parent: &PendingChild,
    start_box: &BlockBox,
    state: &mut FortressState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    y_off: i32,
    z_off: i32,
    is_castle: bool,
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
        is_castle,
    );
}

#[expect(clippy::too_many_arguments)]
fn generate_and_add_piece(
    start_box: &BlockBox,
    state: &mut FortressState,
    random: &mut impl RandomImpl,
    collector: &mut StructurePiecesCollector,
    pending_children: &mut Vec<PendingChild>,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
    is_castle: bool,
) {
    if (foot_x - start_box.min.x).abs() <= 112 && (foot_z - start_box.min.z).abs() <= 112 {
        let (piece_opt, child_opt) = generate_piece(
            state,
            is_castle,
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
    } else {
        bridge_end::BridgeEndPiece::create(
            collector, random, foot_x, foot_y, foot_z, direction, depth,
        );
    }
}

fn update_piece_weight(pieces: &[PieceWeight]) -> i32 {
    let mut has_any_pieces = false;
    let mut total_weight = 0;

    for piece in pieces {
        if piece.limit > 0 && piece.generated_count < piece.limit {
            has_any_pieces = true;
        }
        total_weight += piece.weight;
    }

    if has_any_pieces { total_weight } else { -1 }
}

#[expect(clippy::too_many_arguments)]
fn generate_piece(
    state: &mut FortressState,
    is_castle: bool,
    collector: &mut StructurePiecesCollector,
    random: &mut impl RandomImpl,
    foot_x: i32,
    foot_y: i32,
    foot_z: i32,
    direction: BlockDirection,
    depth: u32,
) -> (Option<Box<dyn StructurePieceBase>>, Option<PendingChild>) {
    let pieces = if is_castle {
        &mut state.available_castle_pieces
    } else {
        &mut state.available_bridge_pieces
    };

    let total_weight = update_piece_weight(pieces);
    let do_stuff = total_weight > 0 && depth <= 30;
    let mut num_attempts = 0;

    while num_attempts < 5 && do_stuff {
        num_attempts += 1;
        let mut weight_selection = random.next_bounded_i32(total_weight);

        for i in 0..pieces.len() {
            weight_selection -= pieces[i].weight;
            if weight_selection < 0 {
                let piece_weight = &pieces[i];
                let same_as_prev = state.previous_piece == Some(piece_weight.piece_type)
                    && !piece_weight.repeatable;
                if !piece_weight.can_generate() || same_as_prev {
                    break;
                }

                let piece_type = piece_weight.piece_type;
                if let Some((piece, child)) = create_piece_and_child(
                    piece_type, collector, random, foot_x, foot_y, foot_z, direction, depth,
                ) {
                    pieces[i].generated_count += 1;
                    state.previous_piece = Some(piece_type);
                    if !pieces[i].can_generate() {
                        pieces.remove(i);
                    }
                    return (Some(piece), Some(child));
                }
            }
        }
    }

    bridge_end::BridgeEndPiece::create(collector, random, foot_x, foot_y, foot_z, direction, depth)
        .map_or((None, None), |piece| {
            let bbox = piece.bounding_box();
            let child = PendingChild {
                piece_type: NetherFortressPieceType::BridgeEnd,
                bbox,
                orientation: direction,
                depth,
            };
            (Some(piece), Some(child))
        })
}

#[expect(clippy::too_many_arguments)]
fn create_piece_and_child(
    piece_type: NetherFortressPieceType,
    collector: &mut StructurePiecesCollector,
    random: &mut impl RandomImpl,
    x: i32,
    y: i32,
    z: i32,
    orientation: BlockDirection,
    depth: u32,
) -> Option<(Box<dyn StructurePieceBase>, PendingChild)> {
    let piece: Box<dyn StructurePieceBase> = match piece_type {
        NetherFortressPieceType::Bridge => {
            bridge::BridgePiece::create(collector, random, x, y, z, orientation, depth)?
        }
        NetherFortressPieceType::BridgeCrossing => {
            bridge_crossing::BridgeCrossingPiece::create(collector, x, y, z, orientation, depth)?
        }
        NetherFortressPieceType::BridgeSmallCrossing => {
            bridge_small_crossing::BridgeSmallCrossingPiece::create(
                collector,
                x,
                y,
                z,
                orientation,
                depth,
            )?
        }
        NetherFortressPieceType::BridgeStairs => {
            bridge_stairs::BridgeStairsPiece::create(collector, x, y, z, depth, orientation)?
        }
        NetherFortressPieceType::BridgePlatform => {
            bridge_platform::BridgePlatformPiece::create(collector, x, y, z, depth, orientation)?
        }
        NetherFortressPieceType::CorridorExit => corridor_exit::CorridorExitPiece::create(
            collector,
            random,
            x,
            y,
            z,
            orientation,
            depth,
        )?,
        NetherFortressPieceType::SmallCorridor => {
            small_corridor::SmallCorridorPiece::create(collector, x, y, z, orientation, depth)?
        }
        NetherFortressPieceType::CorridorCrossing => {
            corridor_crossing::CorridorCrossingPiece::create(
                collector,
                x,
                y,
                z,
                orientation,
                depth,
            )?
        }
        NetherFortressPieceType::CorridorRightTurn => {
            corridor_right_turn::CorridorRightTurnPiece::create(
                collector,
                random,
                x,
                y,
                z,
                orientation,
                depth,
            )?
        }
        NetherFortressPieceType::CorridorLeftTurn => {
            corridor_left_turn::CorridorLeftTurnPiece::create(
                collector,
                random,
                x,
                y,
                z,
                orientation,
                depth,
            )?
        }
        NetherFortressPieceType::CorridorStairs => {
            corridor_stairs::CorridorStairsPiece::create(collector, x, y, z, orientation, depth)?
        }
        NetherFortressPieceType::CorridorBalcony => {
            corridor_balcony::CorridorBalconyPiece::create(collector, x, y, z, orientation, depth)?
        }
        NetherFortressPieceType::CorridorNetherWartsRoom => {
            corridor_nether_warts_room::CorridorNetherWartsRoomPiece::create(
                collector,
                x,
                y,
                z,
                orientation,
                depth,
            )?
        }
        NetherFortressPieceType::BridgeEnd => {
            bridge_end::BridgeEndPiece::create(collector, random, x, y, z, orientation, depth)?
        }
    };

    let bbox = piece.bounding_box();
    let child = PendingChild {
        piece_type,
        bbox,
        orientation,
        depth,
    };
    Some((piece, child))
}

/// The shared base state for every nether fortress piece.
#[derive(Clone)]
pub struct NetherFortressPiece {
    pub piece: crate::generation::structure::structures::StructurePiece,
}

impl NetherFortressPiece {
    #[must_use]
    pub const fn new(
        r#type: crate::generation::structure::piece::StructurePieceType,
        chain_length: u32,
        bbox: BlockBox,
    ) -> Self {
        Self {
            piece: crate::generation::structure::structures::StructurePiece::new(
                r#type,
                bbox,
                chain_length,
            ),
        }
    }

    #[must_use]
    pub const fn is_in_bounds(bb: &BlockBox) -> bool {
        bb.min.y > 10
    }
}
