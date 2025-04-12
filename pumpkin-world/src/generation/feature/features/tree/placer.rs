use pumpkin_data::block::{Block, BlockState, get_block_by_state_id};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

use super::{TreeFeature, trunk::TrunkType};

#[derive(Deserialize)]
pub struct TrunkPlacer {
    base_height: u8,
    height_rand_a: u8,
    height_rand_b: u8,
    r#type: TrunkType,
}

impl TrunkPlacer {
    pub fn get_height(&self, random: &mut RandomGenerator) -> u32 {
        self.base_height as u32
            + random.next_bounded_i32(self.height_rand_a as i32 + 1) as u32
            + random.next_bounded_i32(self.height_rand_b as i32 + 1) as u32
    }

    pub fn place(&self, chunk: &mut ProtoChunk, pos: &BlockPos, trunk_block: &BlockState) -> bool {
        if TreeFeature::can_replace(chunk, pos) {
            chunk.set_block_state(&pos.0, trunk_block);
            return true;
        }
        false
    }

    pub fn generate(
        &self,
        height: u32,
        start_pos: BlockPos,
        chunk: &mut ProtoChunk,
        trunk_block: &BlockState,
    ) {
        self.r#type
            .generate(self, height, start_pos, chunk, trunk_block);
    }
}
