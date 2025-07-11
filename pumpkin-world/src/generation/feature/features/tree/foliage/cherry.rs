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
pub struct CherryFoliagePlacer {
    height: IntProvider,
    wide_bottom_layer_hole_chance: f32,
    corner_hole_chance: f32,
    hanging_leaves_chance: f32,
    hanging_leaves_extension_chance: f32,
}

impl CherryFoliagePlacer {
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
        let pos = node.center.up_height(offset);
        let radius = radius + node.foliage_radius - 1;
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            pos,
            radius - 2,
            foliage_height - 3,
            node.giant_trunk,
            foliage_provider,
        );
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            pos,
            radius - 1,
            foliage_height - 4,
            node.giant_trunk,
            foliage_provider,
        );
        for y in foliage_height - 5..0 {
            FoliagePlacer::generate_square(
                self,
                chunk,
                level,
                random,
                pos,
                radius,
                y,
                node.giant_trunk,
                foliage_provider,
            );
        }
        // TODO: generateSquareWithHangingLeaves
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            pos,
            radius,
            -1,
            node.giant_trunk,
            foliage_provider,
        );
        // TODO: generateSquareWithHangingLeaves
        FoliagePlacer::generate_square(
            self,
            chunk,
            level,
            random,
            pos,
            radius - 1,
            -2,
            node.giant_trunk,
            foliage_provider,
        );
    }
    pub fn get_random_height(&self, random: &mut RandomGenerator) -> i32 {
        self.height.get(random)
    }
}

impl LeaveValidator for CherryFoliagePlacer {
    fn is_invalid_for_leaves(
        &self,
        random: &mut pumpkin_util::random::RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        if y == -1
            && (dx == radius || dz == radius)
            && random.next_f32() < self.wide_bottom_layer_hole_chance
        {
            return true;
        }
        let in_radius = dx == radius && dz == radius;
        if radius > 2 {
            return in_radius
                || dx + dz > radius * 2 - 2 && random.next_f32() < self.corner_hole_chance;
        }
        in_radius && random.next_f32() < self.corner_hole_chance
    }
}
