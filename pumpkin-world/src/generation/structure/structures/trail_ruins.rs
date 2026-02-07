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

pub struct TrailRuinsGenerator;

impl StructureGenerator for TrailRuinsGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(TrailRuinsPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::TrailRuins,
                x,
                64,
                z,
                10,
                5,
                10,
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
pub struct TrailRuinsPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for TrailRuinsPiece {
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

        let mud_bricks = Block::MUD_BRICKS.default_state;
        let gravel = Block::GRAVEL.default_state;
        let suspicious_gravel = Block::SUSPICIOUS_GRAVEL.default_state;
        let cobblestone = Block::COBBLESTONE.default_state;
        let terracotta = Block::TERRACOTTA.default_state;
        let air = Block::AIR.default_state;

        // Trail ruins are partially buried — lower half is underground
        // Foundation ring
        p.fill(chunk, &box_limit, 0, -2, 0, 9, -1, 9, gravel);
        p.fill(chunk, &box_limit, 1, -2, 1, 8, -1, 8, mud_bricks);

        // Ground floor — partially ruined walls
        // North wall (partial)
        p.fill(chunk, &box_limit, 1, 0, 0, 5, 2, 0, mud_bricks);
        // East wall (partial)
        p.fill(chunk, &box_limit, 9, 0, 1, 9, 1, 6, cobblestone);
        // South wall (partial)
        p.fill(chunk, &box_limit, 4, 0, 9, 8, 2, 9, terracotta);
        // West wall (partial)
        p.fill(chunk, &box_limit, 0, 0, 3, 0, 2, 8, mud_bricks);

        // Interior floor
        p.fill(chunk, &box_limit, 1, 0, 1, 8, 0, 8, mud_bricks);

        // Suspicious gravel patches (archaeology dig sites)
        p.add_block(chunk, suspicious_gravel, 3, 0, 3, &box_limit);
        p.add_block(chunk, suspicious_gravel, 5, 0, 5, &box_limit);
        p.add_block(chunk, suspicious_gravel, 7, 0, 2, &box_limit);
        p.add_block(chunk, suspicious_gravel, 2, 0, 7, &box_limit);

        // Gravel fill (partially collapsed areas)
        p.add_block(chunk, gravel, 6, 0, 4, &box_limit);
        p.add_block(chunk, gravel, 4, 0, 6, &box_limit);
        p.add_block(chunk, gravel, 7, 1, 7, &box_limit);

        // Terracotta decorative pillar (surviving)
        p.fill(chunk, &box_limit, 2, 0, 2, 2, 3, 2, terracotta);

        // Interior is air above floor
        p.fill(chunk, &box_limit, 1, 1, 1, 8, 4, 8, air);
        // Re-place walls that were inside (they were cleared by the air fill)
        p.fill(chunk, &box_limit, 1, 0, 0, 5, 2, 0, mud_bricks);
        p.fill(chunk, &box_limit, 9, 0, 1, 9, 1, 6, cobblestone);
        p.fill(chunk, &box_limit, 4, 0, 9, 8, 2, 9, terracotta);
        p.fill(chunk, &box_limit, 0, 0, 3, 0, 2, 8, mud_bricks);
        p.fill(chunk, &box_limit, 2, 0, 2, 2, 3, 2, terracotta);

        // Fill downward from corners
        for &x in &[0, 9] {
            for &z in &[0, 9] {
                p.fill_downwards(chunk, cobblestone, x, -3, z, &box_limit);
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
