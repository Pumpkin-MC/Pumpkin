use pumpkin_data::Block;
use pumpkin_util::math::{block_box::BlockBox, position::BlockPos, vector3::Vector3};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos,
        structure::structures::{
            StructureGenerator, StructurePiecesCollector, StructurePosition, fill_with_outline,
        },
    },
};

#[derive(Deserialize, Clone, Debug)]
pub struct NetherFortressGenerator;

impl StructureGenerator for NetherFortressGenerator {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> StructurePosition {
        let start_x = chunk_pos::start_block_x(chunk.x);
        let start_z = chunk_pos::start_block_z(chunk.z);
        let generator = StructurePiecesCollector {
            pieces_positions: vec![], // TODO
        };

        StructurePosition {
            position: BlockPos::new(start_x, 64, start_z),
            generator,
        }
    }
    fn generate(&self, position: BlockBox, chunk: &mut crate::ProtoChunk) {
        BridgePlatform::generate(&BridgePlatform, position, chunk);
    }
}

pub struct BridgePlatform;

impl BridgePlatform {
    fn generate(&self, _box: BlockBox, chunk: &mut crate::ProtoChunk) {
        super::fill(0, 2, 0, 6, 7, 7, Block::AIR.default_state, chunk);
        super::fill(1, 0, 0, 5, 1, 7, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 2, 1, 5, 2, 7, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 3, 2, 5, 3, 7, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 4, 3, 5, 4, 7, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 2, 0, 1, 4, 2, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(5, 2, 0, 5, 4, 2, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 5, 2, 1, 5, 3, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(5, 5, 2, 5, 5, 3, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(0, 5, 3, 0, 5, 8, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(6, 5, 3, 6, 5, 8, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(1, 5, 8, 5, 5, 8, Block::NETHER_BRICKS.default_state, chunk);
        // TODO
        let block_state = Block::NETHER_BRICK_FENCE.default_state;
        let block_state2 = Block::NETHER_BRICK_FENCE.default_state;

        chunk.set_block_state(
            &Vector3::new(1, 6, 3),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(5, 6, 3),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(0, 6, 3),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(6, 6, 3),
            Block::NETHER_BRICK_FENCE.default_state,
        );

        fill_with_outline(0, 6, 4, 0, 6, 7, block_state2, block_state2, chunk);
        fill_with_outline(6, 6, 4, 6, 6, 7, block_state2, block_state2, chunk);

        chunk.set_block_state(
            &Vector3::new(0, 6, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(6, 6, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );

        fill_with_outline(1, 6, 8, 5, 6, 8, block_state, block_state, chunk);

        chunk.set_block_state(
            &Vector3::new(1, 7, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );

        fill_with_outline(2, 7, 8, 4, 7, 8, block_state, block_state, chunk);

        chunk.set_block_state(
            &Vector3::new(5, 7, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(
            &Vector3::new(2, 8, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );
        chunk.set_block_state(&Vector3::new(3, 8, 8), block_state);
        chunk.set_block_state(
            &Vector3::new(4, 8, 8),
            Block::NETHER_BRICK_FENCE.default_state,
        );

        // Blaze Spawner
        chunk.set_block_state(
            &Vector3::new(3, 5, 5), // ?
            Block::SPAWNER.default_state,
        );

        for x in 0..6 {
            for z in 0..6 {
                super::fill_downwards(x, -1, z, Block::NETHER_BRICKS.default_state, chunk);
            }
        }
    }
}

pub struct SmallCorridor;

impl SmallCorridor {
    fn generate(&self, _position: BlockPos, chunk: &mut crate::ProtoChunk) {
        super::fill(0, 0, 0, 4, 1, 4, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(0, 2, 0, 4, 5, 4, Block::AIR.default_state, chunk);

        // TODO
        let block_state = Block::NETHER_BRICK_FENCE.default_state;

        super::fill(0, 2, 0, 0, 5, 4, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(4, 2, 0, 4, 5, 4, Block::NETHER_BRICKS.default_state, chunk);
        super::fill(0, 3, 1, 0, 4, 1, block_state, chunk);
        super::fill(0, 3, 3, 0, 4, 3, block_state, chunk);
        super::fill(4, 3, 1, 4, 4, 1, block_state, chunk);
        super::fill(4, 3, 3, 4, 4, 3, block_state, chunk);
        super::fill(0, 6, 0, 4, 6, 4, Block::NETHER_BRICKS.default_state, chunk);

        for i in 0..=4 {
            for j in 0..=4 {
                super::fill_downwards(i, -1, j, Block::NETHER_BRICKS.default_state, chunk);
            }
        }
    }
}
