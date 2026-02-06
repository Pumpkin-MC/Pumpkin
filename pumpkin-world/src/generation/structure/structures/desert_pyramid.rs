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

pub struct DesertPyramidGenerator;

impl StructureGenerator for DesertPyramidGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(DesertPyramidPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::DesertTemple,
                x,
                64,
                z,
                21,
                15,
                21,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            has_placed_trap: [false; 4],
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 64, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct DesertPyramidPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    has_placed_trap: [bool; 4],
}

impl StructurePieceBase for DesertPyramidPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        if !self
            .shiftable_structure_piece
            .adjust_to_min_height(chunk, -14)
        {
            return;
        }

        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let sandstone = Block::SANDSTONE.default_state;
        let cut_sandstone = Block::CUT_SANDSTONE.default_state;
        let chiseled_sandstone = Block::CHISELED_SANDSTONE.default_state;
        let orange_terracotta = Block::ORANGE_TERRACOTTA.default_state;
        let blue_terracotta = Block::BLUE_TERRACOTTA.default_state;
        let air = Block::AIR.default_state;
        let tnt = Block::TNT.default_state;
        let sandstone_slab = Block::SANDSTONE_SLAB.default_state;
        let stone_pressure_plate = Block::STONE_PRESSURE_PLATE.default_state;

        // Clear interior with air above and fill with sandstone
        p.fill(chunk, &box_limit, 0, 0, 0, 20, 0, 20, sandstone);

        // Main sandstone structure (walls)
        for level in 1..=9 {
            p.fill_with_outline(
                chunk,
                &box_limit,
                true,
                level,
                level,
                level,
                20 - level,
                level,
                20 - level,
                sandstone,
                air,
            );
        }

        // Four towers at corners
        // Tower 1 (NW)
        p.fill(chunk, &box_limit, 0, 0, 0, 4, 9, 4, sandstone);
        // Tower 2 (NE)
        p.fill(chunk, &box_limit, 16, 0, 0, 20, 9, 4, sandstone);
        // Tower 3 (SW)
        p.fill(chunk, &box_limit, 0, 0, 16, 4, 9, 20, sandstone);
        // Tower 4 (SE)
        p.fill(chunk, &box_limit, 16, 0, 16, 20, 9, 20, sandstone);

        // Tower interiors (hollow)
        p.fill(chunk, &box_limit, 1, 1, 1, 3, 8, 3, air);
        p.fill(chunk, &box_limit, 17, 1, 1, 19, 8, 3, air);
        p.fill(chunk, &box_limit, 1, 1, 17, 3, 8, 19, air);
        p.fill(chunk, &box_limit, 17, 1, 17, 19, 8, 19, air);

        // Tower tops (crenellations)
        for &x_off in &[0, 4, 16, 20] {
            for &z_off in &[0, 4, 16, 20] {
                p.fill_downwards(chunk, sandstone, x_off, -1, z_off, &box_limit);
            }
        }

        // Tower pinnacles
        p.add_block(chunk, sandstone, 2, 10, 0, &box_limit);
        p.add_block(chunk, sandstone, 2, 10, 4, &box_limit);
        p.add_block(chunk, sandstone, 0, 10, 2, &box_limit);
        p.add_block(chunk, sandstone, 4, 10, 2, &box_limit);
        p.add_block(chunk, sandstone, 18, 10, 0, &box_limit);
        p.add_block(chunk, sandstone, 18, 10, 4, &box_limit);
        p.add_block(chunk, sandstone, 16, 10, 2, &box_limit);
        p.add_block(chunk, sandstone, 20, 10, 2, &box_limit);
        p.add_block(chunk, sandstone, 2, 10, 16, &box_limit);
        p.add_block(chunk, sandstone, 2, 10, 20, &box_limit);
        p.add_block(chunk, sandstone, 0, 10, 18, &box_limit);
        p.add_block(chunk, sandstone, 4, 10, 18, &box_limit);
        p.add_block(chunk, sandstone, 18, 10, 16, &box_limit);
        p.add_block(chunk, sandstone, 18, 10, 20, &box_limit);
        p.add_block(chunk, sandstone, 16, 10, 18, &box_limit);
        p.add_block(chunk, sandstone, 20, 10, 18, &box_limit);

        // Orange terracotta decorations on towers
        p.fill(chunk, &box_limit, 1, 10, 1, 3, 10, 3, orange_terracotta);
        p.fill(
            chunk,
            &box_limit,
            17,
            10,
            1,
            19,
            10,
            3,
            orange_terracotta,
        );
        p.fill(
            chunk,
            &box_limit,
            1,
            10,
            17,
            3,
            10,
            19,
            orange_terracotta,
        );
        p.fill(
            chunk,
            &box_limit,
            17,
            10,
            17,
            19,
            10,
            19,
            orange_terracotta,
        );

        // Front entrance
        p.fill(chunk, &box_limit, 8, 1, 0, 12, 4, 0, cut_sandstone);
        p.fill(chunk, &box_limit, 9, 1, 0, 11, 3, 0, air);

        // Chiseled sandstone decorations
        p.add_block(chunk, cut_sandstone, 9, 1, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 9, 2, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 9, 3, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 10, 3, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 11, 3, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 11, 2, 1, &box_limit);
        p.add_block(chunk, cut_sandstone, 11, 1, 1, &box_limit);

        // Blue terracotta - center floor pattern
        p.fill(chunk, &box_limit, 9, 1, 5, 11, 1, 5, blue_terracotta);
        p.add_block(chunk, orange_terracotta, 10, 1, 5, &box_limit);

        // Center wool/terracotta pattern - decorative floor
        p.fill(chunk, &box_limit, 9, 1, 10, 11, 1, 10, blue_terracotta);
        p.add_block(chunk, orange_terracotta, 10, 1, 10, &box_limit);

        // Underground chamber (the treasure room)
        p.fill(chunk, &box_limit, 7, -4, 7, 13, -3, 13, cut_sandstone);
        p.fill(chunk, &box_limit, 8, -4, 8, 12, -2, 12, air);

        // Walls of underground chamber
        p.fill(chunk, &box_limit, 7, -3, 7, 7, -2, 13, sandstone);
        p.fill(chunk, &box_limit, 13, -3, 7, 13, -2, 13, sandstone);
        p.fill(chunk, &box_limit, 8, -3, 7, 12, -2, 7, sandstone);
        p.fill(chunk, &box_limit, 8, -3, 13, 12, -2, 13, sandstone);

        // Floor of underground chamber
        p.fill(chunk, &box_limit, 8, -4, 8, 12, -4, 12, cut_sandstone);

        // Orange terracotta pattern in treasure room floor
        p.fill(chunk, &box_limit, 8, -4, 8, 8, -4, 12, orange_terracotta);
        p.fill(chunk, &box_limit, 12, -4, 8, 12, -4, 12, orange_terracotta);
        p.fill(chunk, &box_limit, 9, -4, 8, 11, -4, 8, orange_terracotta);
        p.fill(chunk, &box_limit, 9, -4, 12, 11, -4, 12, orange_terracotta);
        p.add_block(chunk, blue_terracotta, 10, -4, 10, &box_limit);

        // TNT trap under pressure plate
        p.fill(chunk, &box_limit, 9, -5, 9, 11, -5, 11, tnt);
        p.add_block(chunk, stone_pressure_plate, 10, -3, 10, &box_limit);
        p.add_block(chunk, sandstone_slab, 10, -2, 10, &box_limit);

        // 4 chests in underground chamber
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            8,
            -3,
            10,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            12,
            -3,
            10,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            10,
            -3,
            8,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            10,
            -3,
            12,
            &box_limit,
        );

        // Central shaft from surface to underground
        p.fill(chunk, &box_limit, 10, -5, 10, 10, 0, 10, air);

        // Chiseled sandstone decoration on facade
        p.add_block(chunk, chiseled_sandstone, 10, 0, 0, &box_limit);
        p.add_block(chunk, chiseled_sandstone, 10, 4, 0, &box_limit);

        // Fill around structure base with sand to blend with desert
        for x in 0..=20 {
            for z in 0..=20 {
                p.fill_downwards(chunk, sandstone, x, -6, z, &box_limit);
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
