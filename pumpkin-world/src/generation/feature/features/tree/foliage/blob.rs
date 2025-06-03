use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::random::{RandomGenerator, RandomImpl};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct BlobFoliagePlacer {
    height: i32,
}

impl BlobFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
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
            let radius = (radius + node.foliage_radius - 1 - y / 2).max(0);
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
            )
            .await;
        }
    }

    pub fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
        self.height
    }
}

impl LeaveValidator for BlobFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        dx == radius && dz == radius && (random.next_bounded_i32(2) == 0 || y == 0)
    }
}
