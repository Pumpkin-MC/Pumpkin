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

pub struct RuinedPortalGenerator;

impl StructureGenerator for RuinedPortalGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Random variant: 0 = small standing, 1 = small fallen, 2 = large standing
        let variant = context.random.next_bounded_i32(3);
        let (width, height, depth) = match variant {
            0 => (6, 8, 1),  // small standing portal
            1 => (8, 3, 6),  // small fallen portal (on its side)
            _ => (8, 12, 1), // large standing portal
        };

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(RuinedPortalPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::RuinedPortal,
                x,
                64,
                z,
                width,
                height,
                depth,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            variant,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 64, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct RuinedPortalPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    variant: i32,
}

impl StructurePieceBase for RuinedPortalPiece {
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

        match self.variant {
            0 => self.place_small_standing(chunk, p, &box_limit),
            1 => self.place_small_fallen(chunk, p, &box_limit),
            _ => self.place_large_standing(chunk, p, &box_limit),
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}

impl RuinedPortalPiece {
    fn place_small_standing(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let obsidian = Block::OBSIDIAN.default_state;
        let crying_obsidian = Block::CRYING_OBSIDIAN.default_state;
        let netherrack = Block::NETHERRACK.default_state;
        let magma = Block::MAGMA_BLOCK.default_state;

        // Netherrack base
        p.fill(chunk, box_limit, 0, 0, 0, 5, 0, 0, netherrack);
        p.add_block(chunk, magma, 2, 0, 0, box_limit);
        p.add_block(chunk, magma, 3, 0, 0, box_limit);

        // Portal frame - left column
        p.fill(chunk, box_limit, 1, 1, 0, 1, 5, 0, obsidian);
        // Portal frame - right column
        p.fill(chunk, box_limit, 4, 1, 0, 4, 5, 0, obsidian);
        // Portal frame - top
        p.fill(chunk, box_limit, 2, 5, 0, 3, 5, 0, obsidian);

        // Ruin effect - missing blocks
        p.add_block(chunk, crying_obsidian, 1, 3, 0, box_limit);
        p.add_block(chunk, crying_obsidian, 4, 2, 0, box_limit);
        // Remove top-right corner for ruin effect
        p.add_block(chunk, Block::AIR.default_state, 4, 5, 0, box_limit);

        // Fill downward from base
        for &x in &[0, 5] {
            p.fill_downwards(chunk, netherrack, x, -1, 0, box_limit);
        }
    }

    fn place_small_fallen(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let obsidian = Block::OBSIDIAN.default_state;
        let crying_obsidian = Block::CRYING_OBSIDIAN.default_state;
        let netherrack = Block::NETHERRACK.default_state;
        let magma = Block::MAGMA_BLOCK.default_state;

        // Netherrack base
        p.fill(chunk, box_limit, 0, 0, 0, 7, 0, 5, netherrack);
        p.add_block(chunk, magma, 3, 0, 2, box_limit);
        p.add_block(chunk, magma, 4, 0, 3, box_limit);

        // Fallen portal frame (laying flat on z-axis)
        // Bottom bar
        p.fill(chunk, box_limit, 1, 1, 1, 1, 1, 4, obsidian);
        // Top bar
        p.fill(chunk, box_limit, 6, 1, 1, 6, 1, 4, obsidian);
        // Left bar
        p.fill(chunk, box_limit, 2, 1, 1, 5, 1, 1, obsidian);
        // Right bar
        p.fill(chunk, box_limit, 2, 1, 4, 5, 1, 4, obsidian);

        // Ruin effect
        p.add_block(chunk, crying_obsidian, 1, 1, 2, box_limit);
        p.add_block(chunk, crying_obsidian, 6, 1, 3, box_limit);
        // Missing corner
        p.add_block(chunk, Block::AIR.default_state, 6, 1, 4, box_limit);

        // Second layer - partial collapse
        p.add_block(chunk, obsidian, 2, 2, 1, box_limit);
        p.add_block(chunk, crying_obsidian, 3, 2, 1, box_limit);

        for &x in &[0, 7] {
            for &z in &[0, 5] {
                p.fill_downwards(chunk, netherrack, x, -1, z, box_limit);
            }
        }
    }

    fn place_large_standing(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let obsidian = Block::OBSIDIAN.default_state;
        let crying_obsidian = Block::CRYING_OBSIDIAN.default_state;
        let netherrack = Block::NETHERRACK.default_state;
        let magma = Block::MAGMA_BLOCK.default_state;

        // Netherrack base - wider
        p.fill(chunk, box_limit, 0, 0, 0, 7, 0, 0, netherrack);
        p.add_block(chunk, magma, 3, 0, 0, box_limit);
        p.add_block(chunk, magma, 4, 0, 0, box_limit);

        // Large portal frame - left column
        p.fill(chunk, box_limit, 1, 1, 0, 1, 9, 0, obsidian);
        // Right column
        p.fill(chunk, box_limit, 6, 1, 0, 6, 9, 0, obsidian);
        // Top bar
        p.fill(chunk, box_limit, 2, 9, 0, 5, 9, 0, obsidian);
        // Bottom bar
        p.fill(chunk, box_limit, 2, 1, 0, 5, 1, 0, obsidian);

        // Corner accent blocks
        p.add_block(chunk, obsidian, 0, 1, 0, box_limit);
        p.add_block(chunk, obsidian, 7, 1, 0, box_limit);

        // Top capstones
        p.add_block(chunk, obsidian, 1, 10, 0, box_limit);
        p.add_block(chunk, obsidian, 6, 10, 0, box_limit);

        // Ruin effect - crying obsidian and missing blocks
        p.add_block(chunk, crying_obsidian, 1, 4, 0, box_limit);
        p.add_block(chunk, crying_obsidian, 6, 6, 0, box_limit);
        p.add_block(chunk, crying_obsidian, 3, 9, 0, box_limit);
        // Missing blocks
        p.add_block(chunk, Block::AIR.default_state, 6, 9, 0, box_limit);
        p.add_block(chunk, Block::AIR.default_state, 6, 8, 0, box_limit);

        // Netherrack debris around base
        p.add_block(chunk, netherrack, 0, 0, 0, box_limit);
        p.add_block(chunk, netherrack, 7, 0, 0, box_limit);

        for &x in &[0, 7] {
            p.fill_downwards(chunk, netherrack, x, -1, 0, box_limit);
        }
    }
}
