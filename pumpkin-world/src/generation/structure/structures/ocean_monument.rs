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

pub struct OceanMonumentGenerator;

impl StructureGenerator for OceanMonumentGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Ocean Monument spawns at sea level in deep ocean
        let y = context.sea_level - 20;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(OceanMonumentPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::OceanMonumentBase,
                x,
                y,
                z,
                23,
                16,
                23,
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
pub struct OceanMonumentPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for OceanMonumentPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let prismarine = Block::PRISMARINE.default_state;
        let dark_prismarine = Block::DARK_PRISMARINE.default_state;
        let sea_lantern = Block::SEA_LANTERN.default_state;
        let water = Block::WATER.default_state;
        let sponge = Block::WET_SPONGE.default_state;
        let gold_block = Block::GOLD_BLOCK.default_state;

        // Base platform
        p.fill(chunk, &box_limit, 0, 0, 0, 22, 0, 22, dark_prismarine);

        // First tier (ground floor)
        p.fill(chunk, &box_limit, 1, 1, 1, 21, 5, 21, water);
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 5, 22, prismarine);
        p.fill(chunk, &box_limit, 22, 1, 0, 22, 5, 22, prismarine);
        p.fill(chunk, &box_limit, 1, 1, 0, 21, 5, 0, prismarine);
        p.fill(chunk, &box_limit, 1, 1, 22, 21, 5, 22, prismarine);
        // Floor
        p.fill(chunk, &box_limit, 1, 1, 1, 21, 1, 21, dark_prismarine);
        // Ceiling
        p.fill(chunk, &box_limit, 0, 5, 0, 22, 5, 22, dark_prismarine);

        // Second tier (setback)
        p.fill(chunk, &box_limit, 3, 6, 3, 19, 10, 19, water);
        p.fill(chunk, &box_limit, 2, 6, 2, 2, 10, 20, prismarine);
        p.fill(chunk, &box_limit, 20, 6, 2, 20, 10, 20, prismarine);
        p.fill(chunk, &box_limit, 3, 6, 2, 19, 10, 2, prismarine);
        p.fill(chunk, &box_limit, 3, 6, 20, 19, 10, 20, prismarine);
        // Floor
        p.fill(chunk, &box_limit, 3, 6, 3, 19, 6, 19, dark_prismarine);
        // Ceiling
        p.fill(chunk, &box_limit, 2, 10, 2, 20, 10, 20, dark_prismarine);

        // Third tier (top)
        p.fill(chunk, &box_limit, 6, 11, 6, 16, 14, 16, water);
        p.fill(chunk, &box_limit, 5, 11, 5, 5, 14, 17, prismarine);
        p.fill(chunk, &box_limit, 17, 11, 5, 17, 14, 17, prismarine);
        p.fill(chunk, &box_limit, 6, 11, 5, 16, 14, 5, prismarine);
        p.fill(chunk, &box_limit, 6, 11, 17, 16, 14, 17, prismarine);
        // Floor
        p.fill(chunk, &box_limit, 6, 11, 6, 16, 11, 16, dark_prismarine);
        // Roof
        p.fill(chunk, &box_limit, 5, 15, 5, 17, 15, 17, dark_prismarine);

        // Corner pillars
        for &(cx, cz) in &[(0, 0), (0, 22), (22, 0), (22, 22)] {
            p.fill(chunk, &box_limit, cx, 1, cz, cx, 15, cz, prismarine);
        }

        // Sea lanterns
        for &(lx, ly, lz) in &[
            (1, 3, 1),
            (21, 3, 1),
            (1, 3, 21),
            (21, 3, 21),
            (3, 8, 3),
            (19, 8, 3),
            (3, 8, 19),
            (19, 8, 19),
            (11, 13, 11),
        ] {
            p.add_block(chunk, sea_lantern, lx, ly, lz, &box_limit);
        }

        // Treasure room (center, ground floor)
        p.fill(chunk, &box_limit, 9, 2, 9, 13, 2, 13, dark_prismarine);
        p.fill(chunk, &box_limit, 10, 2, 10, 12, 4, 12, water);
        // Gold blocks (treasure)
        p.fill(chunk, &box_limit, 10, 2, 10, 12, 2, 12, gold_block);

        // Sponge rooms
        p.add_block(chunk, sponge, 3, 2, 3, &box_limit);
        p.add_block(chunk, sponge, 19, 2, 19, &box_limit);
        p.add_block(chunk, sponge, 3, 2, 19, &box_limit);
        p.add_block(chunk, sponge, 19, 2, 3, &box_limit);

        // Entrance arch (front)
        p.fill(chunk, &box_limit, 9, 1, 0, 13, 4, 0, water);

        // Fill downward from corners
        for &x in &[0, 22] {
            for &z in &[0, 22] {
                p.fill_downwards(chunk, dark_prismarine, x, -1, z, &box_limit);
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
