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

pub struct PillagerOutpostGenerator;

impl StructureGenerator for PillagerOutpostGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(PillagerOutpostPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::PillagerOutpost,
                x,
                64,
                z,
                11,
                19,
                11,
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
pub struct PillagerOutpostPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for PillagerOutpostPiece {
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
        let dark_oak_planks = Block::DARK_OAK_PLANKS.default_state;
        let dark_oak_log = Block::DARK_OAK_LOG.default_state;
        let dark_oak_slab = Block::DARK_OAK_SLAB.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        // Foundation platform
        p.fill(chunk, &box_limit, 0, 0, 0, 10, 0, 10, cobblestone);

        // Central tower base (4x4 cobblestone pillars at corners)
        p.fill(chunk, &box_limit, 3, 1, 3, 3, 12, 3, cobblestone);
        p.fill(chunk, &box_limit, 7, 1, 3, 7, 12, 3, cobblestone);
        p.fill(chunk, &box_limit, 3, 1, 7, 3, 12, 7, cobblestone);
        p.fill(chunk, &box_limit, 7, 1, 7, 7, 12, 7, cobblestone);

        // Tower walls connecting pillars (lower section)
        p.fill(chunk, &box_limit, 4, 1, 3, 6, 1, 3, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 1, 7, 6, 1, 7, dark_oak_planks);
        p.fill(chunk, &box_limit, 3, 1, 4, 3, 1, 6, dark_oak_planks);
        p.fill(chunk, &box_limit, 7, 1, 4, 7, 1, 6, dark_oak_planks);

        // Floor at level 1
        p.fill(chunk, &box_limit, 4, 1, 4, 6, 1, 6, dark_oak_planks);

        // Tower walls (level 5 - second floor)
        p.fill(chunk, &box_limit, 4, 5, 3, 6, 5, 3, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 5, 7, 6, 5, 7, dark_oak_planks);
        p.fill(chunk, &box_limit, 3, 5, 4, 3, 5, 6, dark_oak_planks);
        p.fill(chunk, &box_limit, 7, 5, 4, 7, 5, 6, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 5, 4, 6, 5, 6, dark_oak_planks);

        // Tower walls (level 9 - third floor)
        p.fill(chunk, &box_limit, 4, 9, 3, 6, 9, 3, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 9, 7, 6, 9, 7, dark_oak_planks);
        p.fill(chunk, &box_limit, 3, 9, 4, 3, 9, 6, dark_oak_planks);
        p.fill(chunk, &box_limit, 7, 9, 4, 7, 9, 6, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 9, 4, 6, 9, 6, dark_oak_planks);

        // Interior air for each level
        p.fill(chunk, &box_limit, 4, 2, 4, 6, 4, 6, air);
        p.fill(chunk, &box_limit, 4, 6, 4, 6, 8, 6, air);
        p.fill(chunk, &box_limit, 4, 10, 4, 6, 12, 6, air);

        // Windows on each level
        p.add_block(chunk, air, 5, 3, 3, &box_limit);
        p.add_block(chunk, air, 5, 3, 7, &box_limit);
        p.add_block(chunk, air, 3, 3, 5, &box_limit);
        p.add_block(chunk, air, 7, 3, 5, &box_limit);
        p.add_block(chunk, air, 5, 7, 3, &box_limit);
        p.add_block(chunk, air, 5, 7, 7, &box_limit);
        p.add_block(chunk, air, 3, 7, 5, &box_limit);
        p.add_block(chunk, air, 7, 7, 5, &box_limit);
        p.add_block(chunk, air, 5, 11, 3, &box_limit);
        p.add_block(chunk, air, 5, 11, 7, &box_limit);
        p.add_block(chunk, air, 3, 11, 5, &box_limit);
        p.add_block(chunk, air, 7, 11, 5, &box_limit);

        // Roof platform
        p.fill(chunk, &box_limit, 2, 13, 2, 8, 13, 8, dark_oak_planks);
        // Roof border - slabs
        p.fill(chunk, &box_limit, 2, 13, 2, 2, 13, 8, dark_oak_slab);
        p.fill(chunk, &box_limit, 8, 13, 2, 8, 13, 8, dark_oak_slab);
        p.fill(chunk, &box_limit, 3, 13, 2, 7, 13, 2, dark_oak_slab);
        p.fill(chunk, &box_limit, 3, 13, 8, 7, 13, 8, dark_oak_slab);

        // Top roof peak
        p.fill(chunk, &box_limit, 3, 14, 3, 7, 14, 7, dark_oak_planks);
        p.fill(chunk, &box_limit, 4, 15, 4, 6, 15, 6, dark_oak_planks);
        p.add_block(chunk, dark_oak_planks, 5, 16, 5, &box_limit);

        // Fence railing on roof
        p.add_block(chunk, oak_fence, 2, 14, 2, &box_limit);
        p.add_block(chunk, oak_fence, 8, 14, 2, &box_limit);
        p.add_block(chunk, oak_fence, 2, 14, 8, &box_limit);
        p.add_block(chunk, oak_fence, 8, 14, 8, &box_limit);

        // Ladder inside
        p.fill(
            chunk,
            &box_limit,
            4,
            2,
            4,
            4,
            12,
            4,
            Block::LADDER.default_state,
        );

        // Entrance at ground level
        p.fill(chunk, &box_limit, 5, 1, 3, 5, 2, 3, air);

        // Chest inside
        p.add_block(chunk, Block::CHEST.default_state, 6, 2, 6, &box_limit);

        // Dark oak log accents
        p.add_block(chunk, dark_oak_log, 3, 0, 3, &box_limit);
        p.add_block(chunk, dark_oak_log, 7, 0, 3, &box_limit);
        p.add_block(chunk, dark_oak_log, 3, 0, 7, &box_limit);
        p.add_block(chunk, dark_oak_log, 7, 0, 7, &box_limit);

        // Fill downward from corners
        for &x in &[0, 10] {
            for &z in &[0, 10] {
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
