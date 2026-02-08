use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

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

pub struct BastionRemnantGenerator;

impl StructureGenerator for BastionRemnantGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Bastion Remnants generate in the Nether at Y 33-75
        let y = 33;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(BastionRemnantPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::BastionRemnant,
                x,
                y,
                z,
                20,
                14,
                20,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, y, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct BastionRemnantPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for BastionRemnantPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let blackstone = Block::BLACKSTONE.default_state;
        let polished_blackstone = Block::POLISHED_BLACKSTONE.default_state;
        let polished_blackstone_bricks = Block::POLISHED_BLACKSTONE_BRICKS.default_state;
        let gold_block = Block::GOLD_BLOCK.default_state;
        let basalt = Block::BASALT.default_state;
        let air = Block::AIR.default_state;
        let lantern = Block::LANTERN.default_state;
        let magma = Block::MAGMA_BLOCK.default_state;

        // Foundation
        p.fill(chunk, &box_limit, 0, 0, 0, 19, 0, 19, blackstone);
        p.fill(chunk, &box_limit, 2, 0, 2, 17, 0, 17, polished_blackstone);

        // Outer walls
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 10, 19, blackstone);
        p.fill(chunk, &box_limit, 19, 1, 0, 19, 10, 19, blackstone);
        p.fill(chunk, &box_limit, 1, 1, 0, 18, 10, 0, blackstone);
        p.fill(chunk, &box_limit, 1, 1, 19, 18, 10, 19, blackstone);

        // Interior
        p.fill(chunk, &box_limit, 1, 1, 1, 18, 10, 18, air);

        // Inner walls (polished blackstone bricks)
        p.fill(chunk, &box_limit, 2, 1, 2, 2, 7, 17, polished_blackstone_bricks);
        p.fill(chunk, &box_limit, 17, 1, 2, 17, 7, 17, polished_blackstone_bricks);
        p.fill(chunk, &box_limit, 3, 1, 2, 16, 7, 2, polished_blackstone_bricks);
        p.fill(chunk, &box_limit, 3, 1, 17, 16, 7, 17, polished_blackstone_bricks);

        // Inner room air
        p.fill(chunk, &box_limit, 3, 1, 3, 16, 7, 16, air);

        // Inner floor
        p.fill(chunk, &box_limit, 3, 1, 3, 16, 1, 16, polished_blackstone);

        // Ceiling
        p.fill(chunk, &box_limit, 0, 11, 0, 19, 11, 19, blackstone);
        // Open ceiling center
        p.fill(chunk, &box_limit, 5, 11, 5, 14, 11, 14, air);

        // Corner towers
        for &(cx, cz) in &[(1, 1), (1, 18), (18, 1), (18, 18)] {
            p.fill(chunk, &box_limit, cx, 1, cz, cx, 12, cz, polished_blackstone_bricks);
        }

        // Basalt pillars
        for &(bx, bz) in &[(5, 5), (5, 14), (14, 5), (14, 14)] {
            p.fill(chunk, &box_limit, bx, 1, bz, bx, 7, bz, basalt);
        }

        // Treasure bridge (center, elevated)
        p.fill(chunk, &box_limit, 7, 4, 7, 12, 4, 12, polished_blackstone_bricks);
        // Gold treasure
        p.fill(chunk, &box_limit, 8, 5, 8, 11, 5, 11, gold_block);

        // Magma floor accents
        p.add_block(chunk, magma, 4, 1, 4, &box_limit);
        p.add_block(chunk, magma, 15, 1, 4, &box_limit);
        p.add_block(chunk, magma, 4, 1, 15, &box_limit);
        p.add_block(chunk, magma, 15, 1, 15, &box_limit);

        // Lanterns
        for &(lx, lz) in &[(5, 5), (5, 14), (14, 5), (14, 14)] {
            p.add_block(chunk, lantern, lx, 8, lz, &box_limit);
        }

        // Entrance
        p.fill(chunk, &box_limit, 8, 1, 0, 11, 4, 0, air);
        p.fill(chunk, &box_limit, 8, 1, 2, 11, 4, 2, air);

        // Chests
        p.add_block(chunk, Block::CHEST.default_state, 4, 2, 4, &box_limit);
        p.add_block(chunk, Block::CHEST.default_state, 15, 2, 15, &box_limit);

        // Fill downward from corners
        for &x in &[0, 19] {
            for &z in &[0, 19] {
                p.fill_downwards(chunk, blackstone, x, -1, z, &box_limit);
            }
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
