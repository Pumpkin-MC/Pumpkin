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

pub struct OceanMonumentGenerator;

impl StructureGenerator for OceanMonumentGenerator {
    fn get_structure_position(
        &self,
        context: StructureGeneratorContext<'_>,
    ) -> Option<StructurePosition> {
        let chunk_center_x = get_center_x(context.chunk_x);
        let chunk_center_z = get_center_z(context.chunk_z);

        let bounding_box = BlockBox::new(
            chunk_center_x - 29,
            context.min_y,
            chunk_center_z - 29,
            chunk_center_x + 29,
            256,
            chunk_center_z + 29,
        );

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(OceanMonumentPiece {
            piece: StructurePiece::new(StructurePieceType::OceanMonumentBase, bounding_box, 0),
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(chunk_center_x, 64, chunk_center_z),
            collector: Arc::new(collector.into()),
        })
    }
}

pub struct OceanMonumentPiece {
    piece: StructurePiece,
}

impl StructurePieceBase for OceanMonumentPiece {
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
        let sample_y = chunk.get_top_y(&pumpkin_util::HeightMap::OceanFloorWg, origin.x, origin.z);
        let start_y = if sample_y <= 0 { 40 } else { sample_y - 4 };

        // Draw a giant 58x58 pyramid/stepped prismarine temple!
        for dy in 0..15 {
            let half_sz = 29 - dy;
            if half_sz < 0 {
                break;
            }
            let min_x = origin.x - half_sz;
            let max_x = origin.x + half_sz;
            let min_z = origin.z - half_sz;
            let max_z = origin.z + half_sz;
            let y = start_y + dy;

            for x in min_x..=max_x {
                for z in min_z..=max_z {
                    if chunk_box.contains(x, y, z) {
                        let is_border = x == min_x || x == max_x || z == min_z || z == max_z;
                        let block = if is_border {
                            Block::DARK_PRISMARINE
                        } else if dy % 3 == 0 {
                            Block::PRISMARINE_BRICKS
                        } else {
                            Block::PRISMARINE
                        };
                        chunk.set_block_state(x, y, z, block.default_state);
                    }
                }
            }
        }
    }
}
