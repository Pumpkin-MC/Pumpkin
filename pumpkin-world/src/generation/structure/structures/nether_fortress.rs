// use std::sync::Arc;

// use pumpkin_data::Block;
// use pumpkin_util::{
//     math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
//     random::RandomGenerator,
// };
// use serde::Deserialize;

// use crate::{
//     ProtoChunk,
//     generation::{
//         positions::chunk_pos,
//         structure::structures::{
//             StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
//             StructurePiecesCollector, StructurePosition,
//         },
//     },
// };

// #[derive(Deserialize, Clone, Debug)]
// pub struct NetherFortressGenerator;

// impl StructureGenerator for NetherFortressGenerator {
//     fn get_structure_position(
//         &self,
//         context: StructureGeneratorContext,
//     ) -> Option<StructurePosition> {
//         let start_x = chunk_pos::start_block_x(context.chunk_x);
//         let start_z = chunk_pos::start_block_z(context.chunk_z);
//         let start_y = 64;

//         let mut collector = StructurePiecesCollector::default();

//         // Add a Bridge Platform piece
//         let box_min = BlockPos::new(start_x, start_y, start_z);
//         let box_max = BlockPos::new(start_x + 8, start_y + 10, start_z + 8);

//         todo!();

//         Some(StructurePosition {
//             start_pos: box_min,
//             collector: Arc::new(collector.into()),
//         })
//     }
// }

// #[derive(Debug, Clone)]
// pub enum NetherFortressPieceType {
//     BridgePlatform,
//     SmallCorridor,
// }

// struct NetherFortressPiece {
//     piece: StructurePiece,
//     piece_type: NetherFortressPieceType,
// }

// impl StructurePieceBase for NetherFortressPiece {
//     fn place(&mut self, chunk: &mut ProtoChunk, _seed: i64) {
//         let bounding_box = self.piece.bounding_box;
//         let origin_x = bounding_box.min.x;
//         let origin_y = bounding_box.min.y;
//         let origin_z = bounding_box.min.z;

//         let offset = |x, y, z| Vector3::new(origin_x + x, origin_y + y, origin_z + z);

//         let fill_rel = |min_x, min_y, min_z, max_x, max_y, max_z, block, c: &mut ProtoChunk| {
//             fill(
//                 origin_x + min_x,
//                 origin_y + min_y,
//                 origin_z + min_z,
//                 origin_x + max_x,
//                 origin_y + max_y,
//                 origin_z + max_z,
//                 block,
//                 c,
//             )
//         };

//         let fill_outline_rel =
//             |min_x, min_y, min_z, max_x, max_y, max_z, outline, inside, c: &mut ProtoChunk| {
//                 fill_with_outline(
//                     origin_x + min_x,
//                     origin_y + min_y,
//                     origin_z + min_z,
//                     origin_x + max_x,
//                     origin_y + max_y,
//                     origin_z + max_z,
//                     outline,
//                     inside,
//                     c,
//                 )
//             };

//         match self.piece_type {
//             NetherFortressPieceType::BridgePlatform => {
//                 fill_rel(0, 2, 0, 6, 7, 7, Block::AIR.default_state, chunk);
//                 fill_rel(1, 0, 0, 5, 1, 7, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 2, 1, 5, 2, 7, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 3, 2, 5, 3, 7, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 4, 3, 5, 4, 7, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 2, 0, 1, 4, 2, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(5, 2, 0, 5, 4, 2, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 5, 2, 1, 5, 3, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(5, 5, 2, 5, 5, 3, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(0, 5, 3, 0, 5, 8, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(6, 5, 3, 6, 5, 8, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(1, 5, 8, 5, 5, 8, Block::NETHER_BRICKS.default_state, chunk);

//                 let fence = &Block::NETHER_BRICK_FENCE.default_state;

//                 chunk.set_block_state(&offset(1, 6, 3), fence);
//                 chunk.set_block_state(&offset(5, 6, 3), fence);
//                 chunk.set_block_state(&offset(0, 6, 3), fence);
//                 chunk.set_block_state(&offset(6, 6, 3), fence);

//                 fill_outline_rel(0, 6, 4, 0, 6, 7, fence, fence, chunk);
//                 fill_outline_rel(6, 6, 4, 6, 6, 7, fence, fence, chunk);

//                 chunk.set_block_state(&offset(0, 6, 8), fence);
//                 chunk.set_block_state(&offset(6, 6, 8), fence);

//                 fill_outline_rel(1, 6, 8, 5, 6, 8, fence, fence, chunk);

//                 chunk.set_block_state(&offset(1, 7, 8), fence);
//                 fill_outline_rel(2, 7, 8, 4, 7, 8, fence, fence, chunk);
//                 chunk.set_block_state(&offset(5, 7, 8), fence);

//                 chunk.set_block_state(&offset(2, 8, 8), fence);
//                 chunk.set_block_state(&offset(3, 8, 8), fence);
//                 chunk.set_block_state(&offset(4, 8, 8), fence);

//                 // Blaze Spawner
//                 chunk.set_block_state(&offset(3, 5, 5), Block::SPAWNER.default_state);

//                 // Pillars down
//                 for x in 0..6 {
//                     for z in 0..6 {
//                         fill_downwards(
//                             origin_x + x,
//                             origin_y - 1,
//                             origin_z + z,
//                             Block::NETHER_BRICKS.default_state,
//                             chunk,
//                         );
//                     }
//                 }
//             }
//             NetherFortressPieceType::SmallCorridor => {
//                 fill_rel(0, 0, 0, 4, 1, 4, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(0, 2, 0, 4, 5, 4, Block::AIR.default_state, chunk);

//                 let fence = &Block::NETHER_BRICK_FENCE.default_state;

//                 fill_rel(0, 2, 0, 0, 5, 4, Block::NETHER_BRICKS.default_state, chunk);
//                 fill_rel(4, 2, 0, 4, 5, 4, Block::NETHER_BRICKS.default_state, chunk);

//                 fill_rel(0, 3, 1, 0, 4, 1, fence, chunk);
//                 fill_rel(0, 3, 3, 0, 4, 3, fence, chunk);
//                 fill_rel(4, 3, 1, 4, 4, 1, fence, chunk);
//                 fill_rel(4, 3, 3, 4, 4, 3, fence, chunk);

//                 fill_rel(0, 6, 0, 4, 6, 4, Block::NETHER_BRICKS.default_state, chunk);

//                 for i in 0..=4 {
//                     for j in 0..=4 {
//                         fill_downwards(
//                             origin_x + i,
//                             origin_y - 1,
//                             origin_z + j,
//                             Block::NETHER_BRICKS.default_state,
//                             chunk,
//                         );
//                     }
//                 }
//             }
//         }
//     }

//     fn get_structure_piece(&self) -> &super::StructurePiece {
//         &self.piece
//     }

//     fn get_structure_piece_mut(&mut self) -> &mut super::StructurePiece {
//         &mut self.piece
//     }
// }

// === New implementation below ===

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

pub struct NetherFortressGenerator;

impl StructureGenerator for NetherFortressGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = get_center_x(context.chunk_x);
        let z = get_center_z(context.chunk_z);

        // Nether Fortress generates at Y 64 in the Nether
        let y = 64;

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(NetherFortressPieceImpl {
            shiftable_structure_piece: ShiftableStructurePiece::new(
                StructurePieceType::NetherFortressStart,
                x,
                y,
                z,
                18,
                10,
                14,
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
pub struct NetherFortressPieceImpl {
    shiftable_structure_piece: ShiftableStructurePiece,
}

impl StructurePieceBase for NetherFortressPieceImpl {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn place(&mut self, chunk: &mut ProtoChunk, _random: &mut RandomGenerator, _seed: i64) {
        let box_limit = self.shiftable_structure_piece.piece.bounding_box;
        let p = &self.shiftable_structure_piece.piece;

        let nether_bricks = Block::NETHER_BRICKS.default_state;
        let nether_brick_fence = Block::NETHER_BRICK_FENCE.default_state;
        let air = Block::AIR.default_state;
        let lava = Block::LAVA.default_state;
        let soul_sand = Block::SOUL_SAND.default_state;
        let nether_wart = Block::NETHER_WART.default_state;

        // Bridge platform with corridor
        // Floor
        p.fill(chunk, &box_limit, 0, 0, 0, 17, 0, 13, nether_bricks);
        // Carve interior
        p.fill(chunk, &box_limit, 1, 1, 1, 16, 8, 12, air);

        // Side walls
        p.fill(chunk, &box_limit, 0, 1, 0, 0, 8, 13, nether_bricks);
        p.fill(chunk, &box_limit, 17, 1, 0, 17, 8, 13, nether_bricks);
        // End walls
        p.fill(chunk, &box_limit, 1, 1, 0, 16, 8, 0, nether_bricks);
        p.fill(chunk, &box_limit, 1, 1, 13, 16, 8, 13, nether_bricks);

        // Ceiling
        p.fill(chunk, &box_limit, 0, 9, 0, 17, 9, 13, nether_bricks);

        // Entrance openings
        p.fill(chunk, &box_limit, 7, 1, 0, 10, 4, 0, air);
        p.fill(chunk, &box_limit, 7, 1, 13, 10, 4, 13, air);

        // Interior pillars
        for &(px, pz) in &[(4, 4), (4, 9), (13, 4), (13, 9)] {
            p.fill(chunk, &box_limit, px, 1, pz, px, 8, pz, nether_bricks);
        }

        // Fence railings on second level
        p.fill(chunk, &box_limit, 1, 5, 1, 1, 5, 12, nether_brick_fence);
        p.fill(chunk, &box_limit, 16, 5, 1, 16, 5, 12, nether_brick_fence);

        // Blaze spawner platform
        p.fill(chunk, &box_limit, 7, 5, 5, 10, 5, 8, nether_bricks);
        p.add_block(chunk, Block::SPAWNER.default_state, 8, 6, 6, &box_limit);

        // Nether wart garden
        p.fill(chunk, &box_limit, 2, 0, 2, 3, 0, 3, soul_sand);
        p.add_block(chunk, nether_wart, 2, 1, 2, &box_limit);
        p.add_block(chunk, nether_wart, 3, 1, 3, &box_limit);

        // Lava well
        p.fill(chunk, &box_limit, 14, 0, 2, 15, 0, 3, lava);

        // Chest
        p.add_block(chunk, Block::CHEST.default_state, 2, 1, 10, &box_limit);

        // Fill downward
        for x in (0..18).step_by(17) {
            for z in (0..14).step_by(13) {
                p.fill_downwards(chunk, nether_bricks, x, -1, z, &box_limit);
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
