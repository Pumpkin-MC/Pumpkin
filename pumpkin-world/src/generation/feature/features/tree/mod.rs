use placer::TrunkPlacer;
use pumpkin_data::{
    block::{Block, get_block_by_id, get_state_by_state_id},
    tag::Tagable,
};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{block_state_provider::BlockStateProvider, feature::size::FeatureSize},
};

mod placer;
mod trunk;

#[derive(Deserialize)]
pub struct TreeFeature {
    trunk_provider: BlockStateProvider,
    trunk_placer: TrunkPlacer,
    minimum_size: FeatureSize,
    ignore_vines: bool,
}

impl TreeFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // TODO
        return false;
        self.generate_main(chunk, min_y, height, feature_name, random, pos);
        true
    }

    pub fn can_replace_or_log(chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = chunk.get_block_state(&pos.0);
        let block = get_block_by_id(state.block_id).unwrap();

        Self::can_replace(chunk, pos) || block.is_tagged_with("minecraft:logs").unwrap()
    }

    pub fn can_replace(chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = chunk.get_block_state(&pos.0);
        let block = get_block_by_id(state.block_id).unwrap();
        let state = get_state_by_state_id(state.state_id).unwrap();

        state.air
            || block
                .is_tagged_with("minecraft:replaceable_by_trees")
                .unwrap()
    }

    fn generate_main(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        let height = self.trunk_placer.get_height(random);

        let clipped_height = self.minimum_size.min_clipped_height;
        let top = self.get_top(height, chunk, pos); // TODO: roots   
        if top < height && (clipped_height.is_none() || top < clipped_height.unwrap() as u32) {
            return;
        }
        self.trunk_placer
            .generate(top, pos, chunk, &self.trunk_provider.get(random, pos));
    }

    fn get_top(&self, height: u32, chunk: &ProtoChunk, init_pos: BlockPos) -> u32 {
        let mut pos = BlockPos::new(0, 0, 0);
        for y in 0..=height + 1 {
            let j = self.minimum_size.r#type.get_radius(height, y as i32);
            for x in -j..=j {
                for z in -j..=j {
                    pos = BlockPos(init_pos.0.add_raw(x, y as i32, z));
                    let state = chunk.get_block_state(&pos.0);
                    let block = get_block_by_id(state.block_id).unwrap();
                    if Self::can_replace_or_log(chunk, &pos)
                        && (self.ignore_vines || block != Block::VINE)
                    {
                        continue;
                    }
                    return y - 2;
                }
            }
        }
        height
    }
}
