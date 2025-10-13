use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::{math::square, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct LargeOakFoliagePlacer {
    height: i32,
}

impl LargeOakFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        for y in (offset - foliage_height..=offset).rev() {
            let radius = radius
                + if y == offset || y == offset - foliage_height {
                    0
                } else {
                    1
                };
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                node.center,
                radius,
                y,
                node.giant_trunk,
                foliage_provider,
            );
        }
    }

    pub fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
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
