use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_util::{
    math::int_provider::IntProvider,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, generation::feature::features::tree::TreeNode, level::Level};

use super::{FoliagePlacer, LeaveValidator};

#[derive(Deserialize)]
pub struct SpruceFoliagePlacer {
    trunk_height: IntProvider,
}

impl SpruceFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        iradius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        let mut radius = random.next_bounded_i32(2);
        let mut max = 1;
        let mut next = 0;
        for y in ((-foliage_height)..=offset).rev() {
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
            if radius >= max {
                radius = next;
                next = 1;
                max = (iradius + node.foliage_radius).min(max + 1);
                continue;
            }
            radius += 1;
        }
    }
    pub fn get_random_height(&self, random: &mut RandomGenerator, trunk_height: i32) -> i32 {
        (trunk_height - self.trunk_height.get(random)).max(4)
    }
}

impl LeaveValidator for SpruceFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        _random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        _y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        dx == radius && dz == radius && radius > 0
    }
}
