use pumpkin_data::BlockState;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use super::FoliagePlacer;
use crate::generation::feature::features::tree::TreeNode;
use crate::generation::proto_chunk::GenerationCache;

pub struct PoplarFoliagePlacer {
    pub height: IntProvider,
    pub side_hole_chance: f32,
}

impl PoplarFoliagePlacer {
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
    ) -> Vec<BlockPos> {
        let mut foliage_positions = Vec::new();
        let pos = node.center.up_height(offset);
        let current_radius = leaf_radius + node.foliage_radius - 1;
        let flipped = random.next_bool();

        self.generate_rhombus(
            &mut foliage_positions,
            chunk,
            random,
            pos,
            current_radius - 2,
            foliage_height - 1,
            node.giant_trunk,
            foliage_provider,
            foliage_height,
            flipped,
        );
        self.generate_rhombus(
            &mut foliage_positions,
            chunk,
            random,
            pos,
            current_radius - 1,
            foliage_height - 2,
            node.giant_trunk,
            foliage_provider,
            foliage_height,
            flipped,
        );
        self.generate_rhombus(
            &mut foliage_positions,
            chunk,
            random,
            pos,
            current_radius - 1,
            foliage_height - 3,
            node.giant_trunk,
            foliage_provider,
            foliage_height,
            flipped,
        );
        for y in (1..=foliage_height - 4).rev() {
            self.generate_rhombus(
                &mut foliage_positions,
                chunk,
                random,
                pos,
                current_radius,
                y,
                node.giant_trunk,
                foliage_provider,
                foliage_height,
                flipped,
            );
        }
        self.generate_rhombus(
            &mut foliage_positions,
            chunk,
            random,
            pos,
            current_radius - 1,
            0,
            node.giant_trunk,
            foliage_provider,
            foliage_height,
            flipped,
        );
        self.generate_rhombus(
            &mut foliage_positions,
            chunk,
            random,
            pos,
            (current_radius - 2).clamp(1, 2),
            -1,
            node.giant_trunk,
            foliage_provider,
            foliage_height,
            flipped,
        );
        foliage_positions
    }

    pub fn get_random_height(&self, random: &mut RandomGenerator) -> i32 {
        self.height.get(random)
    }

    // Not `FoliagePlacer::generate_square`, which folds away the sign of the offsets
    #[expect(clippy::too_many_arguments)]
    fn generate_rhombus<T: GenerationCache>(
        &self,
        foliage_positions: &mut Vec<BlockPos>,
        chunk: &mut T,
        random: &mut RandomGenerator,
        center_pos: BlockPos,
        radius: i32,
        y: i32,
        giant_trunk: bool,
        foliage_provider: &BlockState,
        foliage_height: i32,
        flipped: bool,
    ) {
        let i = i32::from(giant_trunk);

        for x in -radius..=(radius + i) {
            for z in -radius..=(radius + i) {
                if self.is_invalid_for_leaves(random, x, y, z, radius, foliage_height, flipped) {
                    continue;
                }
                let pos = BlockPos(center_pos.0.add_raw(x, y, z));
                if FoliagePlacer::place_foliage_block(chunk, pos, foliage_provider) {
                    foliage_positions.push(pos);
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn is_invalid_for_leaves(
        &self,
        random: &mut RandomGenerator,
        dx: i32,
        y: i32,
        dz: i32,
        current_radius: i32,
        foliage_height: i32,
        flipped: bool,
    ) -> bool {
        // The top two rows are pulled in
        let partial = foliage_height - 1 == y || foliage_height - 2 == y;
        let abs_dx = dx.abs();
        let abs_dz = dz.abs();
        if partial && (abs_dx == current_radius || abs_dz == current_radius) {
            return true;
        }

        let corner = Self::corner_blocks_to_cut(dx, dz, current_radius, partial, flipped);
        let side_hole = i32::from(random.next_f32() <= self.side_hole_chance);
        abs_dx + abs_dz > current_radius * 2 - (corner + side_hole)
    }

    // Two opposite corners are cut back further, giving the canopy its lopsided outline
    const fn corner_blocks_to_cut(
        dx: i32,
        dz: i32,
        current_radius: i32,
        partial: bool,
        flipped: bool,
    ) -> i32 {
        let small_corner = if flipped {
            dx > 0 && dz > 0 || dz < 0 && dx < 0
        } else {
            dx > 0 && dz < 0 || dz > 0 && dx < 0
        };

        if small_corner {
            current_radius - 1
        } else if partial {
            current_radius + 1
        } else {
            current_radius
        }
    }
}
