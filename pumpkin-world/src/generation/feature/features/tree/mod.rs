use decorator::TreeDecorator;
use foliage::FoliagePlacer;
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockState, tag::Taggable};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;
use trunk::TrunkPlacer;

use crate::generation::proto_chunk::GenerationCache;
use crate::generation::{block_state_provider::BlockStateProvider, feature::size::FeatureSize};

mod decorator;
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
    decorators: Vec<TreeDecorator>,
}

pub struct TreeNode {
    center: BlockPos,
    foliage_radius: i32,
    giant_trunk: bool,
}

impl TreeFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // TODO
        let log_positions = self.generate_main(chunk, min_y, height, feature_name, random, pos);

        for decorator in &self.decorators {
            decorator.generate(chunk, random, &[], &log_positions);
        }
        true
    }

    pub fn can_replace_or_log(state: &BlockState, block: &Block) -> bool {
        Self::can_replace(state, block) || block.has_tag(&tag::Block::MINECRAFT_LOGS)
    }

    pub fn is_air_or_leaves(state: &BlockState, block: &Block) -> bool {
        state.is_air() || block.has_tag(&tag::Block::MINECRAFT_LEAVES)
    }

    pub fn can_replace(state: &BlockState, block: &Block) -> bool {
        state.is_air() || block.has_tag(&tag::Block::MINECRAFT_REPLACEABLE_BY_TREES)
    }

    fn generate_main<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> Vec<BlockPos> {
        let height = self.trunk_placer.get_height(random);

        let clipped_height = self.minimum_size.min_clipped_height;
        let top = self.get_top(height, chunk, pos); // TODO: roots   
        if top < height && (clipped_height.is_none() || top < clipped_height.unwrap() as u32) {
            return vec![];
        }
        let trunk_state = self.trunk_provider.get(random, pos);
        let dirt_state = self.dirt_provider.get(random, pos);

        let (nodes, logs) = self.trunk_placer.generate(
            top,
            pos,
            chunk,
            random,
            self.force_dirt,
            dirt_state,
            trunk_state,
        );

        let foliage_height = self
            .foliage_placer
            .r#type
            .get_random_height(random, height as i32);
        let base_height = height as i32 - foliage_height;
        let foliage_radius = self.foliage_placer.get_random_radius(random, base_height);
        let foliage_state = self.foliage_provider.get(random, pos);
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
        logs
    }

    fn get_top<T: GenerationCache>(&self, height: u32, chunk: &T, init_pos: BlockPos) -> u32 {
        for y in 0..=height + 1 {
            let j = self.minimum_size.r#type.get_radius(height, y as i32);
            for x in -j..=j {
                for z in -j..=j {
                    let pos = BlockPos(init_pos.0.add_raw(x, y as i32, z));
                    let rstate = GenerationCache::get_block_state(chunk, &pos.0);
                    let block = rstate.to_block();
                    if Self::can_replace_or_log(rstate.to_state(), block)
                        && (self.ignore_vines || block != &Block::VINE)
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
