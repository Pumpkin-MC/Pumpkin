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

pub struct ShipwreckGenerator;

impl StructureGenerator for ShipwreckGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Shipwrecks come in multiple sizes; pick a random variant
        let variant = context.random.next_bounded_i32(3);
        let (width, height, depth) = match variant {
            0 => (14, 10, 5), // small
            1 => (16, 12, 7), // medium
            _ => (18, 14, 9), // large
        };

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(ShipwreckPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::Shipwreck,
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
pub struct ShipwreckPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    variant: i32,
}

impl StructurePieceBase for ShipwreckPiece {
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
            0 => self.place_small(chunk, p, &box_limit),
            1 => self.place_medium(chunk, p, &box_limit),
            _ => self.place_large(chunk, p, &box_limit),
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}

impl ShipwreckPiece {
    fn place_small(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let oak_planks = Block::OAK_PLANKS.default_state;
        let spruce_planks = Block::SPRUCE_PLANKS.default_state;
        let oak_log = Block::OAK_LOG.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        // Hull - bottom
        p.fill(chunk, box_limit, 0, 0, 0, 13, 0, 4, oak_planks);
        // Hull - walls
        p.fill(chunk, box_limit, 0, 1, 0, 0, 3, 4, oak_planks);
        p.fill(chunk, box_limit, 13, 1, 0, 13, 3, 4, oak_planks);
        p.fill(chunk, box_limit, 1, 1, 0, 12, 3, 0, oak_planks);
        p.fill(chunk, box_limit, 1, 1, 4, 12, 3, 4, oak_planks);
        // Interior air
        p.fill(chunk, box_limit, 1, 1, 1, 12, 3, 3, air);
        // Deck
        p.fill(chunk, box_limit, 1, 4, 0, 12, 4, 4, spruce_planks);
        // Mast
        p.fill(chunk, box_limit, 6, 5, 2, 6, 9, 2, oak_log);
        // Fence railing
        p.add_block(chunk, oak_fence, 0, 5, 0, box_limit);
        p.add_block(chunk, oak_fence, 0, 5, 4, box_limit);
        p.add_block(chunk, oak_fence, 13, 5, 0, box_limit);
        p.add_block(chunk, oak_fence, 13, 5, 4, box_limit);
        // Bow
        p.fill(chunk, box_limit, 12, 1, 1, 12, 2, 3, oak_planks);
        // Chest/barrel
        p.add_block(chunk, Block::CHEST.default_state, 3, 1, 2, box_limit);
        p.add_block(chunk, Block::BARREL.default_state, 10, 1, 2, box_limit);

        // Fill downward from corners
        for &x in &[0, 13] {
            for &z in &[0, 4] {
                p.fill_downwards(chunk, oak_planks, x, -1, z, box_limit);
            }
        }
    }

    fn place_medium(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let oak_planks = Block::OAK_PLANKS.default_state;
        let spruce_planks = Block::SPRUCE_PLANKS.default_state;
        let spruce_log = Block::SPRUCE_LOG.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        // Hull
        p.fill(chunk, box_limit, 0, 0, 0, 15, 0, 6, oak_planks);
        // Walls
        p.fill(chunk, box_limit, 0, 1, 0, 0, 4, 6, oak_planks);
        p.fill(chunk, box_limit, 15, 1, 0, 15, 4, 6, oak_planks);
        p.fill(chunk, box_limit, 1, 1, 0, 14, 4, 0, oak_planks);
        p.fill(chunk, box_limit, 1, 1, 6, 14, 4, 6, oak_planks);
        // Interior
        p.fill(chunk, box_limit, 1, 1, 1, 14, 4, 5, air);
        // Lower deck
        p.fill(chunk, box_limit, 1, 2, 1, 14, 2, 5, spruce_planks);
        // Upper deck
        p.fill(chunk, box_limit, 1, 5, 0, 14, 5, 6, spruce_planks);
        // Mast
        p.fill(chunk, box_limit, 7, 6, 3, 7, 11, 3, spruce_log);
        // Cabin at stern
        p.fill(chunk, box_limit, 0, 5, 0, 3, 7, 6, oak_planks);
        p.fill(chunk, box_limit, 1, 6, 1, 2, 6, 5, air);
        // Fence railing on deck
        for x in [0, 15] {
            p.add_block(chunk, oak_fence, x, 6, 0, box_limit);
            p.add_block(chunk, oak_fence, x, 6, 6, box_limit);
        }
        // Chests
        p.add_block(chunk, Block::CHEST.default_state, 4, 3, 3, box_limit);
        p.add_block(chunk, Block::CHEST.default_state, 12, 3, 3, box_limit);
        p.add_block(chunk, Block::BARREL.default_state, 1, 6, 3, box_limit);

        for &x in &[0, 15] {
            for &z in &[0, 6] {
                p.fill_downwards(chunk, oak_planks, x, -1, z, box_limit);
            }
        }
    }

    fn place_large(
        &self,
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let dark_oak_planks = Block::DARK_OAK_PLANKS.default_state;
        let spruce_planks = Block::SPRUCE_PLANKS.default_state;
        let dark_oak_log = Block::DARK_OAK_LOG.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        // Hull
        p.fill(chunk, box_limit, 0, 0, 0, 17, 0, 8, dark_oak_planks);
        // Walls
        p.fill(chunk, box_limit, 0, 1, 0, 0, 5, 8, dark_oak_planks);
        p.fill(chunk, box_limit, 17, 1, 0, 17, 5, 8, dark_oak_planks);
        p.fill(chunk, box_limit, 1, 1, 0, 16, 5, 0, dark_oak_planks);
        p.fill(chunk, box_limit, 1, 1, 8, 16, 5, 8, dark_oak_planks);
        // Interior
        p.fill(chunk, box_limit, 1, 1, 1, 16, 5, 7, air);
        // Lower deck
        p.fill(chunk, box_limit, 1, 3, 1, 16, 3, 7, spruce_planks);
        // Upper deck
        p.fill(chunk, box_limit, 1, 6, 0, 16, 6, 8, spruce_planks);
        // Mast - fore and aft
        p.fill(chunk, box_limit, 5, 7, 4, 5, 13, 4, dark_oak_log);
        p.fill(chunk, box_limit, 12, 7, 4, 12, 13, 4, dark_oak_log);
        // Stern cabin
        p.fill(chunk, box_limit, 0, 6, 0, 3, 9, 8, dark_oak_planks);
        p.fill(chunk, box_limit, 1, 7, 1, 2, 8, 7, air);
        // Bow
        p.fill(chunk, box_limit, 15, 1, 2, 15, 3, 6, dark_oak_planks);
        // Fence railings
        for x in [0, 17] {
            p.add_block(chunk, oak_fence, x, 7, 0, box_limit);
            p.add_block(chunk, oak_fence, x, 7, 8, box_limit);
        }
        // Chests/barrels
        p.add_block(chunk, Block::CHEST.default_state, 4, 4, 4, box_limit);
        p.add_block(chunk, Block::CHEST.default_state, 13, 4, 4, box_limit);
        p.add_block(chunk, Block::CHEST.default_state, 1, 7, 4, box_limit);
        p.add_block(chunk, Block::BARREL.default_state, 8, 4, 2, box_limit);
        p.add_block(chunk, Block::BARREL.default_state, 8, 4, 6, box_limit);

        for &x in &[0, 17] {
            for &z in &[0, 8] {
                p.fill_downwards(chunk, dark_oak_planks, x, -1, z, box_limit);
            }
        }
    }
}
