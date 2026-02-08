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

pub struct WoodlandMansionGenerator;

impl StructureGenerator for WoodlandMansionGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(WoodlandMansionPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::WoodlandMansion,
                x,
                64,
                z,
                21,
                18,
                21,
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
pub struct WoodlandMansionPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for WoodlandMansionPiece {
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

        let dark_oak_planks = Block::DARK_OAK_PLANKS.default_state;
        let dark_oak_log = Block::DARK_OAK_LOG.default_state;
        let cobblestone = Block::COBBLESTONE.default_state;
        let birch_planks = Block::BIRCH_PLANKS.default_state;
        let glass_pane = Block::GLASS_PANE.default_state;
        let air = Block::AIR.default_state;

        // Foundation
        p.fill(chunk, &box_limit, 0, 0, 0, 20, 0, 20, cobblestone);

        // Ground floor exterior walls
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 6, 20, dark_oak_planks);
        p.fill(chunk, &box_limit, 20, 1, 0, 20, 6, 20, dark_oak_planks);
        p.fill(chunk, &box_limit, 1, 1, 0, 19, 6, 0, dark_oak_planks);
        p.fill(chunk, &box_limit, 1, 1, 20, 19, 6, 20, dark_oak_planks);

        // Ground floor interior
        p.fill(chunk, &box_limit, 1, 1, 1, 19, 6, 19, air);

        // Ground floor
        p.fill(chunk, &box_limit, 1, 0, 1, 19, 0, 19, dark_oak_planks);

        // Second floor
        p.fill(chunk, &box_limit, 1, 7, 0, 19, 7, 20, dark_oak_planks);

        // Second floor walls
        p.fill(chunk, &box_limit, 0, 8, 0, 0, 13, 20, dark_oak_planks);
        p.fill(chunk, &box_limit, 20, 8, 0, 20, 13, 20, dark_oak_planks);
        p.fill(chunk, &box_limit, 1, 8, 0, 19, 13, 0, dark_oak_planks);
        p.fill(chunk, &box_limit, 1, 8, 20, 19, 13, 20, dark_oak_planks);

        // Second floor interior
        p.fill(chunk, &box_limit, 1, 8, 1, 19, 13, 19, air);

        // Corner pillars (dark oak logs)
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 13, 0, dark_oak_log);
        p.fill(chunk, &box_limit, 20, 1, 0, 20, 13, 0, dark_oak_log);
        p.fill(chunk, &box_limit, 0, 1, 20, 0, 13, 20, dark_oak_log);
        p.fill(chunk, &box_limit, 20, 1, 20, 20, 13, 20, dark_oak_log);

        // Mid-wall pillar accents (ground floor)
        p.fill(chunk, &box_limit, 10, 1, 0, 10, 6, 0, dark_oak_log);
        p.fill(chunk, &box_limit, 10, 1, 20, 10, 6, 20, dark_oak_log);
        p.fill(chunk, &box_limit, 0, 1, 10, 0, 6, 10, dark_oak_log);
        p.fill(chunk, &box_limit, 20, 1, 10, 20, 6, 10, dark_oak_log);

        // Mid-wall pillar accents (second floor)
        p.fill(chunk, &box_limit, 10, 8, 0, 10, 13, 0, dark_oak_log);
        p.fill(chunk, &box_limit, 10, 8, 20, 10, 13, 20, dark_oak_log);
        p.fill(chunk, &box_limit, 0, 8, 10, 0, 13, 10, dark_oak_log);
        p.fill(chunk, &box_limit, 20, 8, 10, 20, 13, 10, dark_oak_log);

        // Windows - ground floor (front/back walls)
        for &x in &[4, 7, 13, 16] {
            p.add_block(chunk, glass_pane, x, 3, 0, &box_limit);
            p.add_block(chunk, glass_pane, x, 4, 0, &box_limit);
            p.add_block(chunk, glass_pane, x, 3, 20, &box_limit);
            p.add_block(chunk, glass_pane, x, 4, 20, &box_limit);
        }

        // Windows - ground floor (side walls)
        for &z in &[4, 7, 13, 16] {
            p.add_block(chunk, glass_pane, 0, 3, z, &box_limit);
            p.add_block(chunk, glass_pane, 0, 4, z, &box_limit);
            p.add_block(chunk, glass_pane, 20, 3, z, &box_limit);
            p.add_block(chunk, glass_pane, 20, 4, z, &box_limit);
        }

        // Windows - second floor (front/back walls)
        for &x in &[4, 7, 13, 16] {
            p.add_block(chunk, glass_pane, x, 10, 0, &box_limit);
            p.add_block(chunk, glass_pane, x, 11, 0, &box_limit);
            p.add_block(chunk, glass_pane, x, 10, 20, &box_limit);
            p.add_block(chunk, glass_pane, x, 11, 20, &box_limit);
        }

        // Windows - second floor (side walls)
        for &z in &[4, 7, 13, 16] {
            p.add_block(chunk, glass_pane, 0, 10, z, &box_limit);
            p.add_block(chunk, glass_pane, 0, 11, z, &box_limit);
            p.add_block(chunk, glass_pane, 20, 10, z, &box_limit);
            p.add_block(chunk, glass_pane, 20, 11, z, &box_limit);
        }

        // Entrance - front door
        p.fill(chunk, &box_limit, 9, 1, 0, 11, 4, 0, air);

        // Interior dividing wall (ground floor)
        p.fill(chunk, &box_limit, 10, 1, 1, 10, 6, 19, dark_oak_planks);
        // Doorway in dividing wall
        p.fill(chunk, &box_limit, 10, 1, 9, 10, 3, 11, air);

        // Interior dividing wall (second floor)
        p.fill(chunk, &box_limit, 10, 8, 1, 10, 13, 19, dark_oak_planks);
        p.fill(chunk, &box_limit, 10, 8, 9, 10, 10, 11, air);

        // Stairs (birch planks staircase)
        for step in 0..7 {
            let y = 1 + step;
            let z = 2 + step;
            p.fill(chunk, &box_limit, 2, y, z, 3, y, z, birch_planks);
        }

        // Roof - peaked
        for i in 0..=5 {
            let y = 14 + i;
            let x_min = i;
            let x_max = 20 - i;
            let z_min = i;
            let z_max = 20 - i;
            if x_min <= x_max && z_min <= z_max {
                p.fill(
                    chunk,
                    &box_limit,
                    x_min,
                    y,
                    z_min,
                    x_max,
                    y,
                    z_max,
                    dark_oak_planks,
                );
            }
        }

        // Carpet in main hall (birch planks as floor accent)
        p.fill(chunk, &box_limit, 3, 1, 3, 8, 1, 17, birch_planks);

        // Chest
        p.add_block(
            chunk,
            Block::CHEST.default_state,
            15,
            1,
            15,
            &box_limit,
        );

        // Fill downward from corners
        for &x in &[0, 20] {
            for &z in &[0, 20] {
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
