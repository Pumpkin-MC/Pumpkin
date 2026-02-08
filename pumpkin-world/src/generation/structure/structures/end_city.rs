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

pub struct EndCityGenerator;

impl StructureGenerator for EndCityGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // End cities generate on End islands at Y 64+
        let y = 64;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(EndCityPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::EndCity,
                x,
                y,
                z,
                12,
                20,
                12,
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
pub struct EndCityPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for EndCityPiece {
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

        let purpur = Block::PURPUR_BLOCK.default_state;
        let end_stone_bricks = Block::END_STONE_BRICKS.default_state;
        let end_rod = Block::END_ROD.default_state;
        let air = Block::AIR.default_state;
        let magenta_stained_glass = Block::MAGENTA_STAINED_GLASS.default_state;

        // Foundation
        p.fill(chunk, &box_limit, 0, 0, 0, 11, 0, 11, end_stone_bricks);

        // Tower base (first floor)
        p.fill(chunk, &box_limit, 1, 1, 1, 10, 5, 10, air);
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 5, 11, purpur);
        p.fill(chunk, &box_limit, 11, 1, 0, 11, 5, 11, purpur);
        p.fill(chunk, &box_limit, 1, 1, 0, 10, 5, 0, purpur);
        p.fill(chunk, &box_limit, 1, 1, 11, 10, 5, 11, purpur);

        // First floor ceiling / second floor
        p.fill(chunk, &box_limit, 0, 6, 0, 11, 6, 11, purpur);

        // Second floor
        p.fill(chunk, &box_limit, 1, 7, 1, 10, 11, 10, air);
        p.fill(chunk, &box_limit, 0, 7, 0, 0, 11, 11, purpur);
        p.fill(chunk, &box_limit, 11, 7, 0, 11, 11, 11, purpur);
        p.fill(chunk, &box_limit, 1, 7, 0, 10, 11, 0, purpur);
        p.fill(chunk, &box_limit, 1, 7, 11, 10, 11, 11, purpur);

        // Second floor ceiling / third floor
        p.fill(chunk, &box_limit, 0, 12, 0, 11, 12, 11, purpur);

        // Third floor (smaller — setback)
        p.fill(chunk, &box_limit, 2, 13, 2, 9, 17, 9, air);
        p.fill(chunk, &box_limit, 1, 13, 1, 1, 17, 10, purpur);
        p.fill(chunk, &box_limit, 10, 13, 1, 10, 17, 10, purpur);
        p.fill(chunk, &box_limit, 2, 13, 1, 9, 17, 1, purpur);
        p.fill(chunk, &box_limit, 2, 13, 10, 9, 17, 10, purpur);

        // Roof
        p.fill(chunk, &box_limit, 1, 18, 1, 10, 18, 10, purpur);
        // Top spire
        p.fill(chunk, &box_limit, 5, 19, 5, 6, 19, 6, purpur);

        // Corner pillars
        for &(cx, cz) in &[(0, 0), (0, 11), (11, 0), (11, 11)] {
            p.fill(chunk, &box_limit, cx, 1, cz, cx, 18, cz, end_stone_bricks);
        }

        // Windows (magenta stained glass)
        for &x in &[4, 7] {
            p.add_block(chunk, magenta_stained_glass, x, 3, 0, &box_limit);
            p.add_block(chunk, magenta_stained_glass, x, 3, 11, &box_limit);
            p.add_block(chunk, magenta_stained_glass, x, 9, 0, &box_limit);
            p.add_block(chunk, magenta_stained_glass, x, 9, 11, &box_limit);
        }
        for &z in &[4, 7] {
            p.add_block(chunk, magenta_stained_glass, 0, 3, z, &box_limit);
            p.add_block(chunk, magenta_stained_glass, 11, 3, z, &box_limit);
            p.add_block(chunk, magenta_stained_glass, 0, 9, z, &box_limit);
            p.add_block(chunk, magenta_stained_glass, 11, 9, z, &box_limit);
        }

        // Third floor windows
        for &x in &[4, 7] {
            p.add_block(chunk, magenta_stained_glass, x, 15, 1, &box_limit);
            p.add_block(chunk, magenta_stained_glass, x, 15, 10, &box_limit);
        }
        for &z in &[4, 7] {
            p.add_block(chunk, magenta_stained_glass, 1, 15, z, &box_limit);
            p.add_block(chunk, magenta_stained_glass, 10, 15, z, &box_limit);
        }

        // End rods
        p.add_block(chunk, end_rod, 5, 19, 4, &box_limit);
        p.add_block(chunk, end_rod, 6, 19, 4, &box_limit);
        p.add_block(chunk, end_rod, 5, 19, 7, &box_limit);
        p.add_block(chunk, end_rod, 6, 19, 7, &box_limit);
        p.add_block(chunk, end_rod, 4, 19, 5, &box_limit);
        p.add_block(chunk, end_rod, 7, 19, 5, &box_limit);
        p.add_block(chunk, end_rod, 4, 19, 6, &box_limit);
        p.add_block(chunk, end_rod, 7, 19, 6, &box_limit);

        // Door opening
        p.fill(chunk, &box_limit, 5, 1, 0, 6, 3, 0, air);

        // Chest
        p.add_block(chunk, Block::CHEST.default_state, 5, 13, 5, &box_limit);

        // Fill downward
        for &x in &[0, 11] {
            for &z in &[0, 11] {
                p.fill_downwards(chunk, end_stone_bricks, x, -1, z, &box_limit);
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
