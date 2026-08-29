use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct MegaPineFoliagePlacer {
    pub crown_height: IntProvider,
}

impl MegaPineFoliagePlacer {
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
        let pos = node.center;
        let mut current_radius = 0;
        for y in (pos.0.y - foliage_height + offset)..=(pos.0.y + offset) {
            let delta = pos.0.y - y;
            let computed_radius = radius
                + node.foliage_radius
                + ((delta as f32 / foliage_height as f32) * 3.5).floor() as i32;
            let r = if delta > 0 && computed_radius == current_radius && (y & 1) == 0 {
                computed_radius + 1
            } else {
                computed_radius
            };
            FoliagePlacer::generate_square(
                &mut foliage_positions,
                self,
                chunk,
                random,
                BlockPos::new(pos.0.x, y, pos.0.z),
                r,
                0,
                node.giant_trunk,
                foliage_provider,
            );
            current_radius = computed_radius;
        }
        foliage_positions
    }
    pub fn get_random_height(&self, random: &mut RandomGenerator) -> i32 {
        self.crown_height.get(random)
    }
}

impl LeaveValidator for MegaPineFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        if dx + dz >= 7 {
            return true;
        }
        dx * dx + dz * dz > radius * radius
    }
}
