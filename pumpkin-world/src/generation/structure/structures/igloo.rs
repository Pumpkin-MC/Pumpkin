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

pub struct IglooGenerator;

impl StructureGenerator for IglooGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(IglooPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::Igloo,
                x,
                64,
                z,
                7,
                5,
                8,
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
pub struct IglooPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for IglooPiece {
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

        let snow_block = Block::SNOW_BLOCK.default_state;
        let ice = Block::ICE.default_state;
        let packed_ice = Block::PACKED_ICE.default_state;
        let white_carpet = Block::WHITE_CARPET.default_state;
        let air = Block::AIR.default_state;

        // Build the igloo dome using layers of snow blocks
        // Layer 0 (floor) - 7x8 snow block floor
        p.fill(chunk, &box_limit, 0, 0, 0, 6, 0, 7, snow_block);

        // Layer 1 (walls) - snow block walls with air interior
        p.fill_with_outline(chunk, &box_limit, false, 0, 1, 0, 6, 3, 7, snow_block, air);

        // Cut out doorway on front side
        p.fill(chunk, &box_limit, 2, 1, 0, 4, 2, 0, air);

        // Layer 4 (top/ceiling) - snow blocks
        p.fill(chunk, &box_limit, 1, 4, 1, 5, 4, 6, snow_block);

        // Interior furnishings
        p.add_block(chunk, white_carpet, 2, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 3, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 4, 1, 3, &box_limit);
        p.add_block(chunk, white_carpet, 2, 1, 4, &box_limit);
        p.add_block(chunk, white_carpet, 3, 1, 4, &box_limit);
        p.add_block(chunk, white_carpet, 4, 1, 4, &box_limit);

        // Crafting table and furnace inside
        p.add_block(
            chunk,
            Block::CRAFTING_TABLE.default_state,
            1,
            1,
            6,
            &box_limit,
        );
        p.add_block(chunk, Block::FURNACE.default_state, 5, 1, 6, &box_limit);

        // Torch for light
        p.add_block(chunk, Block::TORCH.default_state, 3, 2, 6, &box_limit);

        // Ice windows on sides
        p.add_block(chunk, ice, 0, 2, 3, &box_limit);
        p.add_block(chunk, ice, 0, 2, 4, &box_limit);
        p.add_block(chunk, ice, 6, 2, 3, &box_limit);
        p.add_block(chunk, ice, 6, 2, 4, &box_limit);

        // Packed ice decoration at entrance
        p.add_block(chunk, packed_ice, 1, 0, 0, &box_limit);
        p.add_block(chunk, packed_ice, 5, 0, 0, &box_limit);

        // Fill downwards from corners with snow to blend into terrain
        for &x in &[0, 6] {
            for &z in &[0, 7] {
                p.fill_downwards(chunk, snow_block, x, -1, z, &box_limit);
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
