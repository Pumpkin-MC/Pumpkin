use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::proto_chunk::GenerationCache;
use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

#[derive(Deserialize)]
pub struct JungleFoliagePlacer {
    height: i32,
}

impl JungleFoliagePlacer {
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
        let height = if node.giant_trunk {
            foliage_height
        } else {
            1 + random.next_bounded_i32(2)
        };
        for y in (offset - height..=offset).rev() {
            let radius = radius + node.foliage_radius + 1 - y;
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
    pub fn get_random_height(&self, _random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.height
    }
}

impl LeaveValidator for JungleFoliagePlacer {
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
