use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::FoliagePlacer;

#[derive(Deserialize)]
pub struct RandomSpreadFoliagePlacer {
    foliage_height: IntProvider,
    leaf_placement_attempts: i32,
}

impl RandomSpreadFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        _node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        _offset: i32,
        foliage_provider: &BlockState,
    ) {
        for _ in 0..self.leaf_placement_attempts {
            let pos = BlockPos::new(
                random.next_bounded_i32(radius) - random.next_bounded_i32(radius),
                random.next_bounded_i32(foliage_height) - random.next_bounded_i32(foliage_height),
                random.next_bounded_i32(radius) - random.next_bounded_i32(radius),
            );
            FoliagePlacer::place_foliage_block(chunk, level, pos, foliage_provider);
        }
    }
    // TODO: getRandomRadius
    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.foliage_height.get(random)
    }
}
