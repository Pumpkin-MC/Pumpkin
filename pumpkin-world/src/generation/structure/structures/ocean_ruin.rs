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

/// Cold ocean ruins - stone brick based
pub struct ColdOceanRuinGenerator;

impl StructureGenerator for ColdOceanRuinGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(OceanRuinPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::OceanRuin,
                x,
                48,
                z,
                8,
                6,
                8,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            is_cold: true,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 48, z),
            collector: Arc::new(collector.into()),
        })
    }
}

/// Warm ocean ruins - sandstone based
pub struct WarmOceanRuinGenerator;

impl StructureGenerator for WarmOceanRuinGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(OceanRuinPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::OceanRuin,
                x,
                48,
                z,
                8,
                6,
                8,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            is_cold: false,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 48, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct OceanRuinPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    is_cold: bool,
}

impl StructurePieceBase for OceanRuinPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        if !self
            .shiftable_structure_piece
            .adjust_to_average_height(chunk)
        {
            return;
        }

        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        if self.is_cold {
            self.place_cold(chunk, p, &box_limit);
        } else {
            self.place_warm(chunk, p, &box_limit);
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}

impl OceanRuinPiece {
    fn place_cold(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let stone_bricks = Block::STONE_BRICKS.default_state;
        let cracked = Block::CRACKED_STONE_BRICKS.default_state;
        let chiseled = Block::CHISELED_STONE_BRICKS.default_state;
        let gravel = Block::GRAVEL.default_state;
        let air = Block::AIR.default_state;

        // Foundation - partially buried
        p.fill(chunk, box_limit, 0, 0, 0, 7, 0, 7, gravel);

        // Walls - partially ruined
        p.fill(chunk, box_limit, 0, 1, 0, 0, 4, 7, stone_bricks);
        p.fill(chunk, box_limit, 7, 1, 0, 7, 4, 7, stone_bricks);
        p.fill(chunk, box_limit, 1, 1, 0, 6, 4, 0, stone_bricks);
        p.fill(chunk, box_limit, 1, 1, 7, 6, 4, 7, stone_bricks);

        // Interior
        p.fill(chunk, box_limit, 1, 1, 1, 6, 4, 6, air);

        // Floor
        p.fill(chunk, box_limit, 1, 0, 1, 6, 0, 6, stone_bricks);

        // Ruin effect - remove some wall blocks at the top
        p.add_block(chunk, air, 0, 4, 2, box_limit);
        p.add_block(chunk, air, 0, 4, 5, box_limit);
        p.add_block(chunk, air, 7, 4, 3, box_limit);
        p.add_block(chunk, air, 7, 3, 6, box_limit);
        p.add_block(chunk, air, 3, 4, 0, box_limit);
        p.add_block(chunk, air, 5, 4, 7, box_limit);
        p.add_block(chunk, air, 2, 4, 7, box_limit);

        // Cracked bricks for age
        p.add_block(chunk, cracked, 0, 2, 3, box_limit);
        p.add_block(chunk, cracked, 7, 2, 4, box_limit);
        p.add_block(chunk, cracked, 3, 1, 0, box_limit);
        p.add_block(chunk, cracked, 5, 3, 7, box_limit);
        p.add_block(chunk, cracked, 1, 1, 1, box_limit);

        // Chiseled decoration
        p.add_block(chunk, chiseled, 0, 1, 0, box_limit);
        p.add_block(chunk, chiseled, 7, 1, 0, box_limit);
        p.add_block(chunk, chiseled, 0, 1, 7, box_limit);
        p.add_block(chunk, chiseled, 7, 1, 7, box_limit);

        // Doorway
        p.fill(chunk, box_limit, 3, 1, 0, 4, 3, 0, air);

        // Chest
        p.add_block(chunk, Block::CHEST.default_state, 3, 1, 5, box_limit);

        // Partial roof
        p.fill(chunk, box_limit, 1, 5, 1, 3, 5, 6, stone_bricks);

        for &x in &[0, 7] {
            for &z in &[0, 7] {
                p.fill_downwards(chunk, stone_bricks, x, -1, z, box_limit);
            }
        }
    }

    fn place_warm(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let sandstone = Block::SANDSTONE.default_state;
        let cut_sandstone = Block::CUT_SANDSTONE.default_state;
        let chiseled = Block::CHISELED_SANDSTONE.default_state;
        let sand = Block::SAND.default_state;
        let air = Block::AIR.default_state;

        // Foundation - sand
        p.fill(chunk, box_limit, 0, 0, 0, 7, 0, 7, sand);

        // Walls - partially ruined sandstone
        p.fill(chunk, box_limit, 0, 1, 0, 0, 3, 7, sandstone);
        p.fill(chunk, box_limit, 7, 1, 0, 7, 3, 7, sandstone);
        p.fill(chunk, box_limit, 1, 1, 0, 6, 3, 0, sandstone);
        p.fill(chunk, box_limit, 1, 1, 7, 6, 3, 7, sandstone);

        // Interior
        p.fill(chunk, box_limit, 1, 1, 1, 6, 3, 6, air);

        // Floor
        p.fill(chunk, box_limit, 1, 0, 1, 6, 0, 6, cut_sandstone);

        // Ruin effect
        p.add_block(chunk, air, 0, 3, 2, box_limit);
        p.add_block(chunk, air, 7, 3, 4, box_limit);
        p.add_block(chunk, air, 4, 3, 0, box_limit);
        p.add_block(chunk, air, 2, 3, 7, box_limit);
        p.add_block(chunk, air, 7, 2, 6, box_limit);
        p.add_block(chunk, air, 0, 2, 5, box_limit);

        // Chiseled decoration at corners
        p.add_block(chunk, chiseled, 0, 1, 0, box_limit);
        p.add_block(chunk, chiseled, 7, 1, 0, box_limit);
        p.add_block(chunk, chiseled, 0, 1, 7, box_limit);
        p.add_block(chunk, chiseled, 7, 1, 7, box_limit);

        // Doorway
        p.fill(chunk, box_limit, 3, 1, 0, 4, 2, 0, air);

        // Chest
        p.add_block(chunk, Block::CHEST.default_state, 4, 1, 5, box_limit);

        // Partial roof
        p.fill(chunk, box_limit, 2, 4, 2, 5, 4, 5, sandstone);

        for &x in &[0, 7] {
            for &z in &[0, 7] {
                p.fill_downwards(chunk, sandstone, x, -1, z, box_limit);
            }
        }
    }
}
