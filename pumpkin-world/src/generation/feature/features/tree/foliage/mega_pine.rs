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
        let mut current = 0;
        // Vanilla: y from (centerY - foliageHeight + offset) to (centerY + offset) inclusive.
        let y_start = pos.0.y - foliage_height + offset;
        let y_end = pos.0.y + offset;
        for y in y_start..=y_end {
            let delta = pos.0.y - y;
            // Tapered crown radius (vanilla MegaPineFoliagePlacer).
            let rad = radius
                + node.foliage_radius
                + if foliage_height > 0 {
                    (delta as f32 / foliage_height as f32 * 3.5).floor() as i32
                } else {
                    0
                };
            // Place with tapered radius — not the base `radius` (that left bare trunks).
            let place_r = if delta > 0 && rad == current && (y & 1) == 0 {
                rad + 1
            } else {
                rad
            };
            FoliagePlacer::generate_square(
                &mut foliage_positions,
                self,
                chunk,
                random,
                BlockPos::new(pos.0.x, y, pos.0.z),
                place_r,
                0,
                node.giant_trunk,
                foliage_provider,
            );
            current = rad;
        }
        foliage_positions
    }
    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
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
