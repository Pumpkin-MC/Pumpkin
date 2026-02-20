use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use super::FoliagePlacer;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

#[derive(Deserialize)]
pub struct RandomSpreadFoliagePlacer {
    pub foliage_height: IntProvider,
    pub leaf_placement_attempts: i32,
}

impl RandomSpreadFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
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
            FoliagePlacer::place_foliage_block(chunk, pos, foliage_provider);
        }
    }
    // TODO: getRandomRadius
    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.foliage_height.get(random)
    }
}
