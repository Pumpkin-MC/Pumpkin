use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::random::RandomGenerator;
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct AcaciaFoliagePlacer;

impl AcaciaFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        radius: i32,
        _offset: i32,
        foliage_provider: &BlockState,
    ) {
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            node.center,
            radius + node.foliage_radius,
            -1,
            node.giant_trunk,
            foliage_provider,
        )
        .await;
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            node.center,
            radius - 1,
            -foliage_height,
            node.giant_trunk,
            foliage_provider,
        )
        .await;
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            node.center,
            radius + node.foliage_radius - 1,
            0,
            node.giant_trunk,
            foliage_provider,
        )
        .await;
    }

    pub fn get_random_height(&self, _random: &mut RandomGenerator) -> i32 {
        0
    }
}

impl LeaveValidator for AcaciaFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        if y == 0 {
            return (dx > 1 || dz > 1) && dx != 0 && dz != 0;
        }
        dx == radius && dz == radius && radius > 0
    }
}
