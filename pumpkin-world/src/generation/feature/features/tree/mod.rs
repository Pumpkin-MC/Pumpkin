use foliage::FoliagePlacer;
use pumpkin_data::{Block, tag::Tagable};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;
use trunk::TrunkPlacer;

use crate::{
    ProtoChunk,
    generation::{block_state_provider::BlockStateProvider, feature::size::FeatureSize},
};

mod foliage;
mod trunk;

#[derive(Deserialize)]
pub struct TreeFeature {
    dirt_provider: BlockStateProvider,
    trunk_provider: BlockStateProvider,
    trunk_placer: TrunkPlacer,
    foliage_provider: BlockStateProvider,
    foliage_placer: FoliagePlacer,
    minimum_size: FeatureSize,
    ignore_vines: bool,
    force_dirt: bool,
}

pub struct TreeNode {
    center: BlockPos,
    foliage_radius: i32,
    giant_trunk: bool,
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
        self.generate_main(chunk, min_y, height, feature_name, random, pos);
        true
    }

    pub fn can_replace_or_log(chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let block = chunk.get_block_state(&pos.0).to_block();

        Self::can_replace(chunk, pos) || block.is_tagged_with("minecraft:logs").unwrap()
    }

    pub fn can_replace(chunk: &ProtoChunk, pos: &BlockPos) -> bool {
        let state = chunk.get_block_state(&pos.0);
        let block = state.to_block();
        let state = state.to_state();

        state.is_air()
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
        let nodes =
            self.trunk_placer
                .generate(top, pos, chunk, &self.trunk_provider.get(random, pos));

        let foliage_height = self.foliage_placer.r#type.get_random_height(random);
        let foliage_radius = self.foliage_placer.get_random_radius(random);
        let foliage_state = &self.foliage_provider.get(random, pos);
        for node in nodes {
            self.foliage_placer.generate(
                chunk,
                random,
                &node,
                foliage_height,
                foliage_radius,
                foliage_state,
            );
        }
    }

    fn get_top(&self, height: u32, chunk: &ProtoChunk, init_pos: BlockPos) -> u32 {
        for y in 0..=height + 1 {
            let j = self.minimum_size.r#type.get_radius(height, y as i32);
            for x in -j..=j {
                for z in -j..=j {
                    let pos = BlockPos(init_pos.0.add_raw(x, y as i32, z));
                    let block = chunk.get_block_state(&pos.0).to_block();
                    if Self::can_replace_or_log(chunk, &pos)
                        && (self.ignore_vines || block != Block::VINE)
                    {
                        continue;
                    }
                    return y.saturating_sub(2);
                }
            }
        }
        height
    }
}
