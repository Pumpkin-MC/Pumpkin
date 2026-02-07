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

pub struct TrialChambersGenerator;

impl StructureGenerator for TrialChambersGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Trial Chambers generate underground, Y -20 to 0
        let y = -20;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(TrialChambersPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::TrialChambers,
                x,
                y,
                z,
                20,
                10,
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
pub struct TrialChambersPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for TrialChambersPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let tuff_bricks = Block::TUFF_BRICKS.default_state;
        let copper_grate = Block::COPPER_GRATE.default_state;
        let trial_spawner = Block::TRIAL_SPAWNER.default_state;
        let air = Block::AIR.default_state;
        let tuff = Block::TUFF.default_state;
        let polished_tuff = Block::POLISHED_TUFF.default_state;

        // Floor
        p.fill(chunk, &box_limit, 0, 0, 0, 19, 0, 19, tuff_bricks);

        // Perimeter walls
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 8, 19, tuff_bricks);
        p.fill(chunk, &box_limit, 19, 1, 0, 19, 8, 19, tuff_bricks);
        p.fill(chunk, &box_limit, 1, 1, 0, 18, 8, 0, tuff_bricks);
        p.fill(chunk, &box_limit, 1, 1, 19, 18, 8, 19, tuff_bricks);

        // Interior air
        p.fill(chunk, &box_limit, 1, 1, 1, 18, 8, 18, air);

        // Ceiling
        p.fill(chunk, &box_limit, 0, 9, 0, 19, 9, 19, tuff_bricks);

        // Central arena floor (polished tuff)
        p.fill(chunk, &box_limit, 5, 0, 5, 14, 0, 14, polished_tuff);

        // Copper grate ceiling accents
        p.fill(chunk, &box_limit, 5, 9, 5, 14, 9, 14, copper_grate);

        // Interior pillars
        for &(px, pz) in &[(4, 4), (4, 15), (15, 4), (15, 15)] {
            p.fill(chunk, &box_limit, px, 1, pz, px, 8, pz, tuff_bricks);
        }

        // Mid-pillars
        for &(px, pz) in &[(9, 4), (9, 15), (4, 9), (15, 9)] {
            p.fill(chunk, &box_limit, px, 1, pz, px+1, 8, pz, tuff_bricks);
        }

        // Trial spawners
        p.add_block(chunk, trial_spawner, 7, 1, 7, &box_limit);
        p.add_block(chunk, trial_spawner, 12, 1, 12, &box_limit);
        p.add_block(chunk, trial_spawner, 7, 1, 12, &box_limit);

        // Raised platforms around spawners
        for &(sx, sz) in &[(6, 6), (11, 11), (6, 11)] {
            p.fill(chunk, &box_limit, sx, 0, sz, sx + 2, 0, sz + 2, tuff);
        }

        // Entrance corridor (north side)
        p.fill(chunk, &box_limit, 8, 1, 0, 11, 4, 0, air);
        p.fill(chunk, &box_limit, 8, 0, 0, 11, 0, 0, polished_tuff);

        // Exit corridor (south side)
        p.fill(chunk, &box_limit, 8, 1, 19, 11, 4, 19, air);
        p.fill(chunk, &box_limit, 8, 0, 19, 11, 0, 19, polished_tuff);

        // Reward vault (center back)
        p.add_block(
            chunk,
            Block::VAULT.default_state,
            9,
            1,
            16,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::VAULT.default_state,
            10,
            1,
            16,
            &box_limit,
        );

        // Decorative tuff brick arches at midpoints
        p.fill(chunk, &box_limit, 9, 6, 1, 10, 8, 1, tuff_bricks);
        p.fill(chunk, &box_limit, 9, 6, 18, 10, 8, 18, tuff_bricks);
        p.fill(chunk, &box_limit, 1, 6, 9, 1, 8, 10, tuff_bricks);
        p.fill(chunk, &box_limit, 18, 6, 9, 18, 8, 10, tuff_bricks);

        // Lanterns for lighting
        for &(lx, lz) in &[(4, 4), (4, 15), (15, 4), (15, 15)] {
            p.add_block(
                chunk,
                Block::LANTERN.default_state,
                lx,
                5,
                lz,
                &box_limit,
            );
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
