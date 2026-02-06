use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    math::position::BlockPos,
    random::RandomGenerator,
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

pub struct JungleTempleGenerator;

impl StructureGenerator for JungleTempleGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(JungleTemplePiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::JungleTemple,
                x,
                64,
                z,
                12,
                14,
                15,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 64, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct JungleTemplePiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for JungleTemplePiece {
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

        let cobblestone = Block::COBBLESTONE.default_state;
        let mossy_cobblestone = Block::MOSSY_COBBLESTONE.default_state;
        let air = Block::AIR.default_state;
        let vine = Block::VINE.default_state;

        // Foundation and floor - mix of cobblestone and mossy cobblestone
        p.fill(chunk, &box_limit, 0, 0, 0, 11, 0, 14, cobblestone);
        p.fill(chunk, &box_limit, 0, 1, 0, 11, 1, 14, cobblestone);

        // Main walls (level 2-10) - cobblestone outline
        for level in 2..=10 {
            // Front and back walls
            p.fill(chunk, &box_limit, 0, level, 0, 11, level, 0, cobblestone);
            p.fill(chunk, &box_limit, 0, level, 14, 11, level, 14, cobblestone);
            // Left and right walls
            p.fill(chunk, &box_limit, 0, level, 1, 0, level, 13, cobblestone);
            p.fill(chunk, &box_limit, 11, level, 1, 11, level, 13, cobblestone);
        }

        // Interior air space (clear inside)
        p.fill(chunk, &box_limit, 1, 2, 1, 10, 9, 13, air);

        // Roof - two levels
        p.fill(chunk, &box_limit, 0, 11, 0, 11, 11, 14, cobblestone);
        p.fill(chunk, &box_limit, 1, 12, 1, 10, 12, 13, cobblestone);
        p.fill(chunk, &box_limit, 2, 13, 2, 9, 13, 12, cobblestone);

        // Entrance doorway
        p.fill(chunk, &box_limit, 4, 2, 0, 7, 5, 0, air);

        // Mossy cobblestone patches on walls (aging/decoration)
        p.add_block(chunk, mossy_cobblestone, 0, 2, 3, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 0, 2, 5, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 0, 3, 4, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 11, 2, 3, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 11, 2, 5, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 11, 3, 4, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 0, 5, 7, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 11, 5, 7, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 0, 4, 10, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 11, 4, 10, &box_limit);

        // Mossy patches on floor
        p.add_block(chunk, mossy_cobblestone, 3, 1, 3, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 5, 1, 5, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 7, 1, 7, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 3, 1, 9, &box_limit);
        p.add_block(chunk, mossy_cobblestone, 8, 1, 11, &box_limit);

        // Interior columns
        p.fill(chunk, &box_limit, 3, 2, 3, 3, 8, 3, cobblestone);
        p.fill(chunk, &box_limit, 8, 2, 3, 8, 8, 3, cobblestone);
        p.fill(chunk, &box_limit, 3, 2, 11, 3, 8, 11, cobblestone);
        p.fill(chunk, &box_limit, 8, 2, 11, 8, 8, 11, cobblestone);

        // Interior floor platform (second level)
        p.fill(chunk, &box_limit, 1, 6, 5, 10, 6, 9, cobblestone);
        p.fill(chunk, &box_limit, 2, 7, 6, 9, 7, 8, air);

        // Vines on exterior walls
        p.add_block(chunk, vine, 0, 8, 3, &box_limit);
        p.add_block(chunk, vine, 0, 7, 5, &box_limit);
        p.add_block(chunk, vine, 0, 9, 7, &box_limit);
        p.add_block(chunk, vine, 11, 8, 3, &box_limit);
        p.add_block(chunk, vine, 11, 7, 5, &box_limit);
        p.add_block(chunk, vine, 11, 9, 7, &box_limit);

        // Ladder at the back for access to second level
        p.fill(
            chunk,
            &box_limit,
            5,
            2,
            13,
            5,
            6,
            13,
            Block::LADDER.default_state,
        );

        // Lower level treasure room with chests
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            8,
            2,
            12,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            3,
            2,
            12,
            &box_limit,
        );

        // Lever (trap mechanism placeholder)
        p.add_block(chunk, Block::LEVER.default_state, 5, 3, 12, &box_limit);

        // TODO: Implement tripwire trap mechanism
        // The actual jungle temple has dispensers with arrows and tripwire traps
        // This requires directional block state support which may need the Redstone consultant

        // Fill downwards from corners to anchor into terrain
        for &x in &[0, 11] {
            for &z in &[0, 14] {
                p.fill_downwards(chunk, cobblestone, x, -1, z, &box_limit);
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
