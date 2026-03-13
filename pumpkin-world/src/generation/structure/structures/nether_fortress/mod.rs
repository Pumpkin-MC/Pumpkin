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
        StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePiecesCollector,
        StructurePosition, nether_fortress::bridge_crossing::BridgeCrossingPiece,
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
        context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let mut collector = StructurePiecesCollector::default();
        let mut random = context.random;

        let start_x = section_coords::section_to_block(context.chunk_x) + 2;
        let start_z = section_coords::section_to_block(context.chunk_z) + 2;

        let start_piece = BridgeCrossingPiece::new_start(&mut random, start_x, start_z);
        let start_box: Box<dyn crate::generation::structure::structures::StructurePieceBase> =
            Box::new(start_piece.clone());

        let mut pieces_to_process: Vec<
            Box<dyn crate::generation::structure::structures::StructurePieceBase>,
        > = Vec::new();

        let mut bridge_pieces = get_bridge_piece_weights();
        let mut corridor_pieces = get_corridor_piece_weights();

        collector.add_piece(start_box);

        start_piece.fill_openings(
            &start_piece.piece.piece,
            &mut random,
            &mut bridge_pieces,
            &mut corridor_pieces,
            &mut collector,
            &mut pieces_to_process,
        );

        while !pieces_to_process.is_empty() {
            let idx = random.next_bounded_i32(pieces_to_process.len() as i32) as usize;
            let piece = pieces_to_process.remove(idx);
            piece.fill_openings_nether(
                &start_piece.piece.piece,
                &mut random,
                &mut bridge_pieces,
                &mut corridor_pieces,
                &mut collector,
                &mut pieces_to_process,
            );
        }

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
}

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

/// The shared base state for every nether fortress piece.
#[derive(Clone)]
pub struct NetherFortressPiece {
    pub piece: StructurePiece,
}

impl NetherFortressPiece {
    #[must_use]
    pub const fn new(
        r#type: crate::generation::structure::piece::StructurePieceType,
        chain_length: u32,
        bbox: BlockBox,
    ) -> Self {
        Self {
            piece: StructurePiece::new(r#type, bbox, chain_length),
        }
    }

    pub const fn is_in_bounds(bb: &BlockBox) -> bool {
        bb.min.y > 10
    }

    /// Attempt to pick a random piece from `weights` up to 5 times.
    /// Falls back to `BridgeEnd` if nothing succeeds.
    #[allow(clippy::too_many_arguments)]
    pub fn pick_piece(
        weights: &mut Vec<PieceWeight>,
        last_piece: &mut Option<NetherFortressPieceType>,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn crate::generation::structure::structures::StructurePieceBase>> {
        let total_weight = Self::check_remaining_pieces(weights);
        if total_weight < 0 {
            return bridge_end::BridgeEndPiece::create(
                collector,
                random,
                x,
                y,
                z,
                orientation,
                chain_length,
            );
        }

        let mut attempt = 0;
        while attempt < 5 {
            attempt += 1;
            let mut k = random.next_bounded_i32(total_weight);

            for i in 0..weights.len() {
                k -= weights[i].weight;
                if k < 0 {
                    let same_as_last =
                        Some(weights[i].piece_type) == *last_piece && !weights[i].repeatable;
                    if !weights[i].can_generate() || same_as_last {
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
                        *last_piece = Some(weights[i].piece_type);

                        if !weights[i].can_generate() {
                            weights.remove(i);
                        }

                        return Some(p);
                    }
                    break;
                }
            }
        }

        bridge_end::BridgeEndPiece::create(collector, random, x, y, z, orientation, chain_length)
    }

    fn check_remaining_pieces(weights: &[PieceWeight]) -> i32 {
        let mut has_limited = false;
        let mut total = 0i32;
        for w in weights {
            if w.limit > 0 && w.generated_count < w.limit {
                has_limited = true;
            }
            total += w.weight;
        }
        if has_limited { total } else { -1 }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_piece(
        piece_type: NetherFortressPieceType,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn crate::generation::structure::structures::StructurePieceBase>> {
        match piece_type {
            NetherFortressPieceType::Bridge => {
                bridge::BridgePiece::create(collector, random, x, y, z, orientation, chain_length)
            }
            NetherFortressPieceType::BridgeCrossing => {
                bridge_crossing::BridgeCrossingPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::BridgeSmallCrossing => {
                bridge_small_crossing::BridgeSmallCrossingPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::BridgeStairs => bridge_stairs::BridgeStairsPiece::create(
                collector,
                x,
                y,
                z,
                chain_length,
                orientation,
            ),
            NetherFortressPieceType::BridgePlatform => {
                bridge_platform::BridgePlatformPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    chain_length,
                    orientation,
                )
            }
            NetherFortressPieceType::CorridorExit => corridor_exit::CorridorExitPiece::create(
                collector,
                random,
                x,
                y,
                z,
                orientation,
                chain_length,
            ),
            NetherFortressPieceType::SmallCorridor => small_corridor::SmallCorridorPiece::create(
                collector,
                x,
                y,
                z,
                orientation,
                chain_length,
            ),
            NetherFortressPieceType::CorridorCrossing => {
                corridor_crossing::CorridorCrossingPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::CorridorRightTurn => {
                corridor_right_turn::CorridorRightTurnPiece::create(
                    collector,
                    random,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::CorridorLeftTurn => {
                corridor_left_turn::CorridorLeftTurnPiece::create(
                    collector,
                    random,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::CorridorStairs => {
                corridor_stairs::CorridorStairsPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::CorridorBalcony => {
                corridor_balcony::CorridorBalconyPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
            NetherFortressPieceType::CorridorNetherWartsRoom => {
                corridor_nether_warts_room::CorridorNetherWartsRoomPiece::create(
                    collector,
                    x,
                    y,
                    z,
                    orientation,
                    chain_length,
                )
            }
        }
    }

    // ---- opening helpers --------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn fill_forward_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        bridge_pieces: &mut Vec<PieceWeight>,
        corridor_pieces: &mut Vec<PieceWeight>,
        last_piece: &mut Option<NetherFortressPieceType>,
        left_right_offset: i32,
        height_offset: i32,
        inside: bool,
        pieces_to_process: &mut Vec<
            Box<dyn crate::generation::structure::structures::StructurePieceBase>,
        >,
    ) {
        if let Some(facing) = self.piece.facing {
            let bb = self.piece.bounding_box;
            let (nx, ny, nz) = match facing {
                BlockDirection::North => (
                    bb.min.x + left_right_offset,
                    bb.min.y + height_offset,
                    bb.min.z - 1,
                ),
                BlockDirection::South => (
                    bb.min.x + left_right_offset,
                    bb.min.y + height_offset,
                    bb.max.z + 1,
                ),
                BlockDirection::West => (
                    bb.min.x - 1,
                    bb.min.y + height_offset,
                    bb.min.z + left_right_offset,
                ),
                BlockDirection::East => (
                    bb.max.x + 1,
                    bb.min.y + height_offset,
                    bb.min.z + left_right_offset,
                ),
                _ => return,
            };
            self.piece_generator(
                start,
                collector,
                random,
                bridge_pieces,
                corridor_pieces,
                last_piece,
                nx,
                ny,
                nz,
                facing,
                inside,
                pieces_to_process,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fill_nw_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        bridge_pieces: &mut Vec<PieceWeight>,
        corridor_pieces: &mut Vec<PieceWeight>,
        last_piece: &mut Option<NetherFortressPieceType>,
        height_offset: i32,
        left_right_offset: i32,
        inside: bool,
        pieces_to_process: &mut Vec<
            Box<dyn crate::generation::structure::structures::StructurePieceBase>,
        >,
    ) {
        if let Some(facing) = self.piece.facing {
            let bb = self.piece.bounding_box;
            let (nx, ny, nz, next_facing) = match facing {
                BlockDirection::North | BlockDirection::South => (
                    bb.min.x - 1,
                    bb.min.y + height_offset,
                    bb.min.z + left_right_offset,
                    BlockDirection::West,
                ),
                BlockDirection::West | BlockDirection::East => (
                    bb.min.x + left_right_offset,
                    bb.min.y + height_offset,
                    bb.min.z - 1,
                    BlockDirection::North,
                ),
                _ => return,
            };
            self.piece_generator(
                start,
                collector,
                random,
                bridge_pieces,
                corridor_pieces,
                last_piece,
                nx,
                ny,
                nz,
                next_facing,
                inside,
                pieces_to_process,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn fill_se_opening(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        bridge_pieces: &mut Vec<PieceWeight>,
        corridor_pieces: &mut Vec<PieceWeight>,
        last_piece: &mut Option<NetherFortressPieceType>,
        height_offset: i32,
        left_right_offset: i32,
        inside: bool,
        pieces_to_process: &mut Vec<
            Box<dyn crate::generation::structure::structures::StructurePieceBase>,
        >,
    ) {
        if let Some(facing) = self.piece.facing {
            let bb = self.piece.bounding_box;
            let (nx, ny, nz, next_facing) = match facing {
                BlockDirection::North | BlockDirection::South => (
                    bb.max.x + 1,
                    bb.min.y + height_offset,
                    bb.min.z + left_right_offset,
                    BlockDirection::East,
                ),
                BlockDirection::West | BlockDirection::East => (
                    bb.min.x + left_right_offset,
                    bb.min.y + height_offset,
                    bb.max.z + 1,
                    BlockDirection::South,
                ),
                _ => return,
            };
            self.piece_generator(
                start,
                collector,
                random,
                bridge_pieces,
                corridor_pieces,
                last_piece,
                nx,
                ny,
                nz,
                next_facing,
                inside,
                pieces_to_process,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn piece_generator(
        &self,
        start: &StructurePiece,
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        bridge_pieces: &mut Vec<PieceWeight>,
        corridor_pieces: &mut Vec<PieceWeight>,
        last_piece: &mut Option<NetherFortressPieceType>,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        inside: bool,
        pieces_to_process: &mut Vec<
            Box<dyn crate::generation::structure::structures::StructurePieceBase>,
        >,
    ) {
        let start_box = start.bounding_box;
        if (x - start_box.min.x).abs() > 112 || (z - start_box.min.z).abs() > 112 {
            if let Some(p) = bridge_end::BridgeEndPiece::create(
                collector,
                random,
                x,
                y,
                z,
                orientation,
                self.piece.chain_length,
            ) {
                collector.add_piece(p)
            }
            return;
        }

        let weights = if inside {
            corridor_pieces
        } else {
            bridge_pieces
        };

        if let Some(p) = Self::pick_piece(
            weights,
            last_piece,
            collector,
            random,
            x,
            y,
            z,
            orientation,
            self.piece.chain_length + 1,
        ) {
            collector.add_piece(p.clone_box());
            pieces_to_process.push(p);
        }
    }
}
