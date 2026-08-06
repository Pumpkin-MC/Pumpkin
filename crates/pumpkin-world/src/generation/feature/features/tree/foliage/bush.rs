use pumpkin_data::BlockState;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct BushFoliagePlacer {
    pub height: i32,
}

impl BushFoliagePlacer {
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
    ) -> Vec<BlockPos> {
        let mut foliage_positions = Vec::new();
        for y in (offset - foliage_height..=offset).rev() {
            let radius = radius + node.foliage_radius - 1 - y;
            FoliagePlacer::generate_square(
                &mut foliage_positions,
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
        foliage_positions
    }

    pub const fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
        self.height
    }
}

impl LeaveValidator for BushFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        dx == radius && dz == radius && random.next_bounded_i32(2) == 0
    }
}
