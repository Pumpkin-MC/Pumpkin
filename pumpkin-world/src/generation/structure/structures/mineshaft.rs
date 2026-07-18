use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomGenerator,
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition, WorldPortalExt,
            },
        },
    },
};

pub struct MineshaftGenerator {
    pub is_mesa: bool,
}

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let bounding_box = BlockBox::new(
            chunk_center_x - 16,
            context.min_y,
            chunk_center_z - 16,
            chunk_center_x + 16,
            256,
            chunk_center_z + 16,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(MineshaftPiece {
            piece: StructurePiece::new(StructurePieceType::MineshaftCrossing, bounding_box, 0),
            is_mesa: self.is_mesa,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct MineshaftPiece {
    piece: StructurePiece,
    is_mesa: bool,
}

impl StructurePieceBase for MineshaftPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }
    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let origin = self.piece.bounding_box.min;
        let start_y = 15;

        let wood_planks = if self.is_mesa {
            Block::DARK_OAK_PLANKS
        } else {
            Block::OAK_PLANKS
        };
        let wood_fence = if self.is_mesa {
            Block::DARK_OAK_FENCE
        } else {
            Block::OAK_FENCE
        };

        // Draw underground corridors
        for y in start_y..(start_y + 3) {
            for x in (origin.x - 12)..=(origin.x + 12) {
                for z in (origin.z - 12)..=(origin.z + 12) {
                    let in_center = (x - origin.x).abs() <= 2 && (z - origin.z).abs() <= 2;
                    let in_ns_corridor = (x - origin.x).abs() <= 1;
                    let in_ew_corridor = (z - origin.z).abs() <= 1;

                    if (in_center || in_ns_corridor || in_ew_corridor)
                        && chunk_box.contains(x, y, z)
                    {
                        chunk.set_block_state(x, y, z, Block::AIR.default_state);

                        if (x - origin.x).abs() == 2 && y == start_y {
                            chunk.set_block_state(x, y, z, wood_fence.default_state);
                        }
                        if (z - origin.z).abs() == 2 && y == start_y {
                            chunk.set_block_state(x, y, z, wood_fence.default_state);
                        }
                    }
                }
            }
        }

        // Place rails and supports
        for x in (origin.x - 12)..=(origin.x + 12) {
            let y = start_y;
            let z = origin.z;
            if chunk_box.contains(x, y, z) {
                chunk.set_block_state(x, y, z, Block::RAIL.default_state);
            }
            if x % 5 == 0 {
                for sy in start_y..=(start_y + 2) {
                    if chunk_box.contains(x, sy, z - 1) {
                        chunk.set_block_state(x, sy, z - 1, wood_fence.default_state);
                    }
                    if chunk_box.contains(x, sy, z + 1) {
                        chunk.set_block_state(x, sy, z + 1, wood_fence.default_state);
                    }
                }
                if chunk_box.contains(x, start_y + 2, z) {
                    chunk.set_block_state(x, start_y + 2, z, wood_planks.default_state);
                }
            }
        }
    }
}
