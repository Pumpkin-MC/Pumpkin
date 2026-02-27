use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos},
    random::RandomGenerator,
};
use serde::Deserialize;

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

#[derive(Deserialize)]
pub struct SwampHutGenerator;

impl StructureGenerator for SwampHutGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(SwampHutPiece {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::SwampHut,
                x,
                64,
                z,
                7,
                7,
                9,
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
pub struct SwampHutPiece {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for SwampHutPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if !self
            .shiftable_structure_piece
            .adjust_to_average_height(chunk)
        {
            return;
        }

        let box_limit = *chunk_box;
        let p = &self.shiftable_structure_piece.piece;

        let spruce_planks = Block::SPRUCE_PLANKS.default_state;
        let oak_log = Block::OAK_LOG.default_state;
        let oak_fence = Block::OAK_FENCE.default_state;
        let air = Block::AIR.default_state;

        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            1,
            1,
            5,
            1,
            7,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            4,
            2,
            5,
            4,
            7,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            1,
            0,
            4,
            1,
            0,
            spruce_planks,
            spruce_planks,
        );

        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            2,
            2,
            3,
            3,
            2,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            1,
            2,
            3,
            1,
            3,
            6,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            5,
            2,
            3,
            5,
            3,
            6,
            spruce_planks,
            spruce_planks,
        );
        p.fill_with_outline(
            chunk,
            &box_limit,
            false,
            2,
            2,
            7,
            4,
            3,
            7,
            spruce_planks,
            spruce_planks,
        );

        p.fill_with_outline(chunk, &box_limit, false, 1, 0, 2, 1, 3, 2, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 5, 0, 2, 5, 3, 2, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 1, 0, 7, 1, 3, 7, oak_log, oak_log);
        p.fill_with_outline(chunk, &box_limit, false, 5, 0, 7, 5, 3, 7, oak_log, oak_log);

        p.add_block(chunk, oak_fence, 2, 3, 2, &box_limit);
        p.add_block(chunk, oak_fence, 3, 3, 7, &box_limit);
        p.add_block(chunk, air, 1, 3, 4, &box_limit);
        p.add_block(chunk, air, 5, 3, 4, &box_limit);
        p.add_block(chunk, air, 5, 3, 5, &box_limit);
        p.add_block(
            chunk,
            Block::POTTED_RED_MUSHROOM.default_state,
            1,
            3,
            5,
            &box_limit,
        );
        p.add_block(
            chunk,
            Block::CRAFTING_TABLE.default_state,
            3,
            2,
            6,
            &box_limit,
        );
        p.add_block(chunk, Block::CAULDRON.default_state, 4, 2, 6, &box_limit);
        p.add_block(chunk, oak_fence, 1, 2, 1, &box_limit);
        p.add_block(chunk, oak_fence, 5, 2, 1, &box_limit);

        // let stairs_n = Block::SPRUCE_STAIRS.default_state.with("facing", "north");
        // let stairs_e = Block::SPRUCE_STAIRS.default_state.with("facing", "east");
        // let stairs_w = Block::SPRUCE_STAIRS.default_state.with("facing", "west");
        // let stairs_s = Block::SPRUCE_STAIRS.default_state.with("facing", "south");

        // p.fill_with_outline(chunk, &box_limit, false, 0, 4, 1, 6, 4, 1, &stairs_n, &stairs_n);
        // p.fill_with_outline(chunk, &box_limit, false, 0, 4, 2, 0, 4, 7, &stairs_e, &stairs_e);
        // p.fill_with_outline(chunk, &box_limit, false, 6, 4, 2, 6, 4, 7, &stairs_w, &stairs_w);
        // p.fill_with_outline(chunk, &box_limit, false, 0, 4, 8, 6, 4, 8, &stairs_s, &stairs_s);

        // p.add_block(chunk, &stairs_n.with("shape", "outer_right"), 0, 4, 1, &box_limit);
        // p.add_block(chunk, &stairs_n.with("shape", "outer_left"), 6, 4, 1, &box_limit);
        // p.add_block(chunk, &stairs_s.with("shape", "outer_left"), 0, 4, 8, &box_limit);
        // p.add_block(chunk, &stairs_s.with("shape", "outer_right"), 6, 4, 8, &box_limit);

        for i in [2, 7] {
            for j in [1, 5] {
                p.fill_downwards(chunk, oak_log, j, -1, i, &box_limit);
            }
        }

        // if !self.has_witch {
        //     let world_coords = p.get_world_coords(2, 2, 5);
        //     if box_limit.contains(world_coords.0, world_coords.1, world_coords.2) {
        //         self.has_witch = true;
        //         // TODO: chunk.add_entity(Witch, world_coords)
        //     }
        // }
        // TODO: self.spawn_cat(chunk, &box_limit);
    }
    fn get_structure_piece(&self) -> &super::StructurePiece {
        &self.shiftable_structure_piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut super::StructurePiece {
        &mut self.shiftable_structure_piece.piece
    }
}
