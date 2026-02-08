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

pub struct AncientCityGenerator;

impl StructureGenerator for AncientCityGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Ancient Cities generate deep underground, Y -51 to -30
        let y = -51;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(AncientCityPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::AncientCity,
                x,
                y,
                z,
                25,
                12,
                25,
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
pub struct AncientCityPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for AncientCityPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let deepslate_bricks = Block::DEEPSLATE_BRICKS.default_state;
        let deepslate_tiles = Block::DEEPSLATE_TILES.default_state;
        let sculk = Block::SCULK.default_state;
        let soul_lantern = Block::SOUL_LANTERN.default_state;
        let dark_oak_planks = Block::DARK_OAK_PLANKS.default_state;
        let dark_oak_log = Block::DARK_OAK_LOG.default_state;
        let air = Block::AIR.default_state;
        let soul_sand = Block::SOUL_SAND.default_state;

        // Ground layer — sculk-covered floor
        p.fill(chunk, &box_limit, 0, 0, 0, 24, 0, 24, sculk);

        // Central plaza floor (deepslate tiles)
        p.fill(chunk, &box_limit, 5, 0, 5, 19, 0, 19, deepslate_tiles);

        // Perimeter walls
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 8, 24, deepslate_bricks);
        p.fill(chunk, &box_limit, 24, 1, 0, 24, 8, 24, deepslate_bricks);
        p.fill(chunk, &box_limit, 1, 1, 0, 23, 8, 0, deepslate_bricks);
        p.fill(chunk, &box_limit, 1, 1, 24, 23, 8, 24, deepslate_bricks);

        // Interior is air
        p.fill(chunk, &box_limit, 1, 1, 1, 23, 8, 23, air);

        // Corner towers
        for &(cx, cz) in &[(1, 1), (1, 21), (21, 1), (21, 21)] {
            p.fill(
                chunk,
                &box_limit,
                cx,
                1,
                cz,
                cx + 2,
                10,
                cz + 2,
                deepslate_bricks,
            );
            // Hollow interior
            p.fill(
                chunk,
                &box_limit,
                cx + 1,
                1,
                cz + 1,
                cx + 1,
                9,
                cz + 1,
                air,
            );
            // Soul lantern on top
            p.add_block(chunk, soul_lantern, cx + 1, 10, cz + 1, &box_limit);
        }

        // Central structure — memorial/portal frame
        p.fill(chunk, &box_limit, 10, 1, 10, 14, 1, 14, deepslate_tiles);
        p.fill(chunk, &box_limit, 10, 1, 10, 10, 6, 10, deepslate_bricks);
        p.fill(chunk, &box_limit, 14, 1, 10, 14, 6, 10, deepslate_bricks);
        p.fill(chunk, &box_limit, 10, 1, 14, 10, 6, 14, deepslate_bricks);
        p.fill(chunk, &box_limit, 14, 1, 14, 14, 6, 14, deepslate_bricks);
        // Top arch
        p.fill(chunk, &box_limit, 10, 6, 10, 14, 6, 14, deepslate_bricks);
        p.fill(chunk, &box_limit, 11, 6, 11, 13, 6, 13, air);
        // Reinforced deepslate in center (represents the sculk catalyst area)
        p.add_block(
            chunk,
            Block::REINFORCED_DEEPSLATE.default_state,
            12,
            1,
            12,
            &box_limit,
        );

        // Walkways from corners to center
        // North-south walkway
        p.fill(chunk, &box_limit, 11, 1, 1, 13, 1, 9, dark_oak_planks);
        p.fill(chunk, &box_limit, 11, 1, 15, 13, 1, 23, dark_oak_planks);
        // East-west walkway
        p.fill(chunk, &box_limit, 1, 1, 11, 9, 1, 13, dark_oak_planks);
        p.fill(chunk, &box_limit, 15, 1, 11, 23, 1, 13, dark_oak_planks);

        // Dark oak pillar accents along walkways
        for &z in &[3, 7, 17, 21] {
            p.fill(chunk, &box_limit, 11, 1, z, 11, 4, z, dark_oak_log);
            p.fill(chunk, &box_limit, 13, 1, z, 13, 4, z, dark_oak_log);
        }
        for &x in &[3, 7, 17, 21] {
            p.fill(chunk, &box_limit, x, 1, 11, x, 4, 11, dark_oak_log);
            p.fill(chunk, &box_limit, x, 1, 13, x, 4, 13, dark_oak_log);
        }

        // Soul sand and soul lanterns along walkways
        for &z in &[5, 19] {
            p.add_block(chunk, soul_sand, 12, 1, z, &box_limit);
            p.add_block(chunk, soul_lantern, 12, 5, z, &box_limit);
        }
        for &x in &[5, 19] {
            p.add_block(chunk, soul_sand, x, 1, 12, &box_limit);
            p.add_block(chunk, soul_lantern, x, 5, 12, &box_limit);
        }

        // Ceiling
        p.fill(chunk, &box_limit, 0, 11, 0, 24, 11, 24, deepslate_bricks);
        // Open ceiling center
        p.fill(chunk, &box_limit, 5, 11, 5, 19, 11, 19, air);

        // Chest loot locations
        p.add_block(chunk, Block::CHEST.default_state, 2, 1, 2, &box_limit);
        p.add_block(chunk, Block::CHEST.default_state, 22, 1, 22, &box_limit);
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
