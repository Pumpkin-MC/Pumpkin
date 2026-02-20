use pumpkin_data::BlockState;
use pumpkin_util::{math::square, random::RandomGenerator};
use serde::Deserialize;

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

#[derive(Deserialize)]
pub struct LargeOakFoliagePlacer {
    pub height: i32,
}

impl LargeOakFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        for y in (offset - foliage_height..=offset).rev() {
            let radius = radius + i32::from(!(y == offset || y == offset - foliage_height));
            FoliagePlacer::generate_square(
                self,
                chunk,
                random,
                node.center,
                radius,
                y,
                node.giant_trunk,
                foliage_provider,
            );
        }
    }

    pub const fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
        self.height
    }
}

impl LeaveValidator for LargeOakFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        square(dx as f32 + 0.5) + square(dz as f32 + 0.5) > (radius * radius) as f32
    }
}
