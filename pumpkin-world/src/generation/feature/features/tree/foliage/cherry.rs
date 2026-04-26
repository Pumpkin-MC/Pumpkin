use pumpkin_data::BlockState;
use pumpkin_util::{
    math::int_provider::IntProvider,
    random::{RandomGenerator, RandomImpl},
};

use super::{FoliagePlacer, LeaveValidator};
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct CherryFoliagePlacer {
    pub height: IntProvider,
    pub wide_bottom_layer_hole_chance: f32,
    pub corner_hole_chance: f32,
    pub hanging_leaves_chance: f32,
    pub hanging_leaves_extension_chance: f32,
}

impl CherryFoliagePlacer {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        node: &TreeNode,
        foliage_height: i32,
        leaf_radius: i32,
        offset: i32,
        foliage_provider: &BlockState,
    ) {
        let pos = node.center.up_height(offset);
        let current_radius = leaf_radius + node.foliage_radius - 1;
        FoliagePlacer::generate_square(
            self,
            chunk,
            random,
            pos,
            current_radius - 2,
            foliage_height - 3,
            node.giant_trunk,
            foliage_provider,
        );
        FoliagePlacer::generate_square(
            self,
            chunk,
            random,
            pos,
            current_radius - 1,
            foliage_height - 4,
            node.giant_trunk,
            foliage_provider,
        );
        for y in (0..=foliage_height - 5).rev() {
            FoliagePlacer::generate_square(
                self,
                chunk,
                random,
                pos,
                current_radius,
                y,
                node.giant_trunk,
                foliage_provider,
            );
        }

        FoliagePlacer::generate_square_with_hanging_leaves(
            self,
            chunk,
            random,
            pos,
            current_radius,
            -1,
            node.giant_trunk,
            foliage_provider,
            self.hanging_leaves_chance,
            self.hanging_leaves_extension_chance,
        );

        FoliagePlacer::generate_square_with_hanging_leaves(
            self,
            chunk,
            random,
            pos,
            current_radius - 1,
            -2,
            node.giant_trunk,
            foliage_provider,
            self.hanging_leaves_chance,
            self.hanging_leaves_extension_chance,
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
        current_radius: i32,
        _giant_trunk: bool,
    ) -> bool {
        if y == -1
            && (dx == current_radius || dz == current_radius)
            && random.next_f32() < self.wide_bottom_layer_hole_chance
        {
            return true;
        }

        let corner = dx == current_radius && dz == current_radius;
        let wide_layer = current_radius > 2;

        if wide_layer {
            corner
                || dx + dz > current_radius * 2 - 2 && random.next_f32() < self.corner_hole_chance
        } else {
            corner && random.next_f32() < self.corner_hole_chance
        }
    }
}
