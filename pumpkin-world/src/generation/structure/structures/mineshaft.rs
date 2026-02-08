use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{get_center_x, get_center_z},
        structure::{
            piece::StructurePieceType,
            shiftable_piece::ShiftableStructurePiece,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

/// Standard mineshaft generator for Overworld underground biomes.
pub struct MineshaftGenerator;

/// Mesa (badlands) mineshaft variant — generates at higher Y levels with dark oak.
pub struct MineshaftMesaGenerator;

impl StructureGenerator for MineshaftGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Generate at a random depth underground (Y 10-40)
        let y = 10 + context.random.next_bounded_i32(30);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(MineshaftPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::MineshaftCorridor,
                x,
                y,
                z,
                20,
                5,
                3,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            is_mesa: false,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, y, z),
            collector: Arc::new(collector.into()),
        })
    }
}

impl StructureGenerator for MineshaftMesaGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Mesa mineshafts generate higher (Y 32-64)
        let y = 32 + context.random.next_bounded_i32(32);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(MineshaftPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::MineshaftCorridor,
                x,
                y,
                z,
                20,
                5,
                3,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            is_mesa: true,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, y, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct MineshaftPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    is_mesa: bool,
}

impl StructurePieceBase for MineshaftPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let planks = if self.is_mesa {
            Block::DARK_OAK_PLANKS.default_state
        } else {
            Block::OAK_PLANKS.default_state
        };
        let fence = if self.is_mesa {
            Block::DARK_OAK_FENCE.default_state
        } else {
            Block::OAK_FENCE.default_state
        };
        let rail = Block::RAIL.default_state;
        let torch = Block::TORCH.default_state;
        let air = Block::AIR.default_state;
        let cobweb = Block::COBWEB.default_state;

        // Carve out the corridor
        p.fill(chunk, &box_limit, 0, 0, 0, 19, 4, 2, air);

        // Floor planks (sparse — every other block)
        for x in (0..20).step_by(2) {
            p.add_block(chunk, planks, x, 0, 0, &box_limit);
            p.add_block(chunk, planks, x, 0, 2, &box_limit);
        }

        // Rail track down the center
        p.fill(chunk, &box_limit, 0, 0, 1, 19, 0, 1, rail);

        // Support beams every 4 blocks
        for x in (2..18).step_by(4) {
            // Left pillar
            p.fill(chunk, &box_limit, x, 0, 0, x, 3, 0, fence);
            // Right pillar
            p.fill(chunk, &box_limit, x, 0, 2, x, 3, 2, fence);
            // Cross beam
            p.fill(chunk, &box_limit, x, 3, 0, x, 3, 2, planks);
        }

        // Torches on support beams
        p.add_block(chunk, torch, 2, 2, 0, &box_limit);
        p.add_block(chunk, torch, 14, 2, 2, &box_limit);

        // Cobwebs (sparse decoration)
        p.add_block(chunk, cobweb, 5, 3, 0, &box_limit);
        p.add_block(chunk, cobweb, 11, 3, 2, &box_limit);
        p.add_block(chunk, cobweb, 17, 4, 1, &box_limit);

        // Chest at corridor end
        p.add_block(chunk, Block::CHEST.default_state, 18, 1, 0, &box_limit);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
