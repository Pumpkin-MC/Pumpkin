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

pub struct NetherFossilGenerator;

impl StructureGenerator for NetherFossilGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // 4 fossil variants: ribcage, spine, skull, hip
        let variant = context.random.next_bounded_i32(4);
        let (width, height, depth) = match variant {
            0 => (8, 5, 5),  // ribcage
            1 => (12, 3, 3), // spine
            2 => (5, 5, 5),  // skull
            _ => (7, 4, 5),  // hip
        };

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(NetherFossilPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::NetherFossil,
                x,
                32,
                z,
                width,
                height,
                depth,
                StructurePiece::get_random_horizontal_direction(&mut context.random).get_axis(),
            ),
            variant,
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x, 32, z),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct NetherFossilPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
    variant: i32,
}

impl StructurePieceBase for NetherFossilPiece {
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
            0 => Self::place_ribcage(chunk, p, &box_limit),
            1 => Self::place_spine(chunk, p, &box_limit),
            2 => Self::place_skull(chunk, p, &box_limit),
            _ => Self::place_hip(chunk, p, &box_limit),
        }
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}

impl NetherFossilPiece {
    fn place_ribcage(
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let bone = Block::BONE_BLOCK.default_state;

        // Spine running along X-axis
        p.fill(chunk, box_limit, 1, 0, 2, 6, 0, 2, bone);

        // Rib pairs
        for x in [2, 4, 6] {
            // Left rib going up
            p.fill(chunk, box_limit, x, 1, 2, x, 3, 2, bone);
            p.add_block(chunk, bone, x, 3, 1, box_limit);
            p.add_block(chunk, bone, x, 4, 0, box_limit);
            // Right rib going up
            p.fill(chunk, box_limit, x, 1, 2, x, 3, 2, bone);
            p.add_block(chunk, bone, x, 3, 3, box_limit);
            p.add_block(chunk, bone, x, 4, 4, box_limit);
        }
    }

    fn place_spine(
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let bone = Block::BONE_BLOCK.default_state;

        // Long spine segment
        p.fill(chunk, box_limit, 0, 0, 1, 11, 0, 1, bone);
        // Vertebra bumps
        for x in (1..11).step_by(2) {
            p.add_block(chunk, bone, x, 1, 1, box_limit);
        }
        // Slight curve upward at ends
        p.add_block(chunk, bone, 0, 1, 1, box_limit);
        p.add_block(chunk, bone, 11, 1, 1, box_limit);
        p.add_block(chunk, bone, 0, 2, 1, box_limit);
    }

    fn place_skull(
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let bone = Block::BONE_BLOCK.default_state;

        // Skull base
        p.fill(chunk, box_limit, 1, 0, 1, 3, 0, 3, bone);
        // Skull top
        p.fill(chunk, box_limit, 1, 1, 1, 3, 2, 3, bone);
        p.fill(chunk, box_limit, 0, 1, 1, 0, 2, 3, bone);
        p.fill(chunk, box_limit, 4, 1, 1, 4, 2, 3, bone);
        // Top cap
        p.fill(chunk, box_limit, 1, 3, 1, 3, 3, 3, bone);
        // Eye sockets (hollow)
        p.add_block(chunk, Block::AIR.default_state, 1, 2, 1, box_limit);
        p.add_block(chunk, Block::AIR.default_state, 3, 2, 1, box_limit);
        // Jaw
        p.fill(chunk, box_limit, 1, 0, 0, 3, 0, 0, bone);
        p.add_block(chunk, bone, 0, 0, 2, box_limit);
        p.add_block(chunk, bone, 4, 0, 2, box_limit);
    }

    fn place_hip(
        chunk: &mut ProtoChunk,
        p: &StructurePiece,
        box_limit: &pumpkin_util::math::block_box::BlockBox,
    ) {
        let bone = Block::BONE_BLOCK.default_state;

        // Central spine segment
        p.fill(chunk, box_limit, 2, 0, 2, 4, 0, 2, bone);
        // Hip bones - left
        p.fill(chunk, box_limit, 0, 0, 2, 1, 0, 2, bone);
        p.add_block(chunk, bone, 0, 1, 2, box_limit);
        p.add_block(chunk, bone, 0, 2, 1, box_limit);
        p.add_block(chunk, bone, 0, 3, 0, box_limit);
        // Hip bones - right
        p.fill(chunk, box_limit, 5, 0, 2, 6, 0, 2, bone);
        p.add_block(chunk, bone, 6, 1, 2, box_limit);
        p.add_block(chunk, bone, 6, 2, 3, box_limit);
        p.add_block(chunk, bone, 6, 3, 4, box_limit);
        // Sacrum
        p.add_block(chunk, bone, 3, 1, 2, box_limit);
    }
}
