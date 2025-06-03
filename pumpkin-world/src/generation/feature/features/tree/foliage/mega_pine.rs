use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct MegaPineFoliagePlacer {
    crown_height: IntProvider,
}

impl MegaPineFoliagePlacer {
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
        let pos = node.center;
        let mut current = 0;
        for y in pos.0.y - foliage_height + offset..pos.0.y + offset {
            let delta = pos.0.y - y;
            let rad = radius
                + node.foliage_radius
                + (delta as f32 / foliage_height as f32 * 3.5).floor() as i32;
            let radius = if delta > 0 && rad == current && (y & 1) == 0 {
                radius + 1
            } else {
                radius
            };
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                BlockPos::new(pos.0.x, y, pos.0.z),
                radius,
                0,
                node.giant_trunk,
                foliage_provider,
            )
            .await;
            current = rad;
        }
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
