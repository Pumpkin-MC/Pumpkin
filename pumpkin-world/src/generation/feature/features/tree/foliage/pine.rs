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
pub struct PineFoliagePlacer {
    height: IntProvider,
}

impl PineFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
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
        let mut radius = 0;
        for y in (offset - foliage_height)..offset {
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
            if radius >= 1 && y == offset - foliage_height + 1 {
                radius -= 1;
            } else if radius < iradius + node.foliage_radius {
                radius += 1;
            }
        }
    }

    pub fn get_random_radius(
        placer: &FoliagePlacer,
        random: &mut RandomGenerator,
        base_height: i32,
    ) -> i32 {
        placer.radius.get(random) + random.next_bounded_i32((base_height + 1).max(1))
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator, _trunk_height: i32) -> i32 {
        self.height.get(random)
    }
}

impl LeaveValidator for PineFoliagePlacer {
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
