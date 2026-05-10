use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::block_predicate::BlockPredicate;
use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct RootSystemFeature {
    pub feature: Box<crate::generation::feature::placed_features::PlacedFeature>,
    pub required_vertical_space_for_tree: i32,
    pub root_radius: i32,
    pub root_replaceable: BlockPredicate,
    pub root_state_provider: BlockStateProvider,
    pub root_placement_attempts: i32,
    pub root_column_max_height: i32,
    pub hanging_root_radius: i32,
    pub hanging_roots_vertical_span: i32,
    pub hanging_root_state_provider: BlockStateProvider,
    pub hanging_root_placement_attempts: i32,
    pub allowed_vertical_water_for_tree: i32,
    pub allowed_tree_position: BlockPredicate,
}

impl RootSystemFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        min_y: i8,
        height: u16,
        feature_name: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !self.allowed_tree_position.test(block_registry, chunk, &pos) {
            return false;
        }

        if !self.can_place_tree(chunk, pos) {
            return false;
        }

        if !self.feature.generate(
            chunk,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        ) {
            return false;
        }

        self.place_roots(chunk, block_registry, random, pos);
        true
    }

    fn can_place_tree<T: GenerationCache>(&self, chunk: &T, pos: BlockPos) -> bool {
        let mut mutable_pos = pos;
        for i in 1..=self.required_vertical_space_for_tree {
            mutable_pos = mutable_pos.add(0, 1, 0);
            if !self.is_allowed_tree_space(chunk, mutable_pos, i) {
                return false;
            }
        }
        true
    }

    fn is_allowed_tree_space<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        vertical_space: i32,
    ) -> bool {
        if chunk.is_air(&pos.0) {
            return true;
        }
        // TODO: check for water.
        vertical_space <= self.allowed_vertical_water_for_tree
    }

    fn place_roots<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        self.place_rooted_dirt(chunk, block_registry, random, pos);
        self.place_hanging_roots(chunk, block_registry, random, pos);
    }

    fn place_rooted_dirt<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        for _ in 0..self.root_placement_attempts {
            let mut mutable_pos = pos.add(
                random.next_bounded_i32(self.root_radius.max(1))
                    - random.next_bounded_i32(self.root_radius.max(1)),
                0,
                random.next_bounded_i32(self.root_radius.max(1))
                    - random.next_bounded_i32(self.root_radius.max(1)),
            );

            if self
                .root_replaceable
                .test(block_registry, chunk, &mutable_pos)
            {
                chunk.set_block_state(
                    &mutable_pos.0,
                    self.root_state_provider.get_with_context(
                        block_registry,
                        chunk,
                        random,
                        mutable_pos,
                    ),
                );
            }

            for _ in 0..self.root_column_max_height {
                mutable_pos = mutable_pos.add(0, -1, 0);
                if chunk.out_of_height(mutable_pos.0.y as i16)
                    || !self
                        .root_replaceable
                        .test(block_registry, chunk, &mutable_pos)
                {
                    break;
                }
                chunk.set_block_state(
                    &mutable_pos.0,
                    self.root_state_provider.get_with_context(
                        block_registry,
                        chunk,
                        random,
                        mutable_pos,
                    ),
                );
            }
        }
    }

    fn place_hanging_roots<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        for _ in 0..self.hanging_root_placement_attempts {
            let mutable_pos = pos.add(
                random.next_bounded_i32(self.hanging_root_radius.max(1))
                    - random.next_bounded_i32(self.hanging_root_radius.max(1)),
                random.next_bounded_i32(self.hanging_roots_vertical_span.max(1))
                    - random.next_bounded_i32(self.hanging_roots_vertical_span.max(1)),
                random.next_bounded_i32(self.hanging_root_radius.max(1))
                    - random.next_bounded_i32(self.hanging_root_radius.max(1)),
            );

            if chunk.is_air(&mutable_pos.0) {
                let state = self.hanging_root_state_provider.get_with_context(
                    block_registry,
                    chunk,
                    random,
                    mutable_pos,
                );
                // In vanilla, it checks if it can survive and if the block above is sturdy.
                // For now, let's just check if the block above is NOT air.
                let above = mutable_pos.add(0, 1, 0);
                if !chunk.is_air(&above.0) {
                    chunk.set_block_state(&mutable_pos.0, state);
                }
            }
        }
    }
}
