use pumpkin_data::{Block, BlockDirection};
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
        feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) {
            return false;
        }

        let mut working_pos = pos;
        if self.place_dirt_and_tree(
            chunk,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            &mut working_pos,
            pos,
        ) {
            self.place_roots(chunk, block_registry, random, pos);
        }

        true
    }

    fn space_for_tree<T: GenerationCache>(&self, chunk: &T, pos: BlockPos) -> bool {
        let mut column_up = pos;
        for i in 1..=self.required_vertical_space_for_tree {
            column_up = column_up.up();
            let state = GenerationCache::get_block_state(chunk, &column_up.0);
            if !self.is_allowed_tree_space(state.to_block_id(), i) {
                return false;
            }
        }
        true
    }

    fn is_allowed_tree_space(
        &self,
        block_id: pumpkin_data::BlockId,
        blocks_above_origin: i32,
    ) -> bool {
        if block_id == Block::AIR.id {
            return true;
        }
        let blocks_above_ground = blocks_above_origin + 1;
        blocks_above_ground <= self.allowed_vertical_water_for_tree && block_id == Block::WATER.id
    }

    #[allow(clippy::too_many_arguments)]
    fn place_dirt_and_tree<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        min_y: i8,
        height: u16,
        feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        working_pos: &mut BlockPos,
        pos: BlockPos,
    ) -> bool {
        for y in 0..self.root_column_max_height {
            *working_pos = working_pos.up();
            if chunk.top_block_height_exclusive(working_pos.0.x, working_pos.0.z) < working_pos.0.y
            {
                return false;
            }

            if self
                .allowed_tree_position
                .test(block_registry, chunk, working_pos)
                && self.space_for_tree(chunk, *working_pos)
            {
                let below_pos = working_pos.down();
                let below_state = GenerationCache::get_block_state(chunk, &below_pos.0);
                if below_state.to_block_id() == Block::LAVA.id
                    || !below_state.to_state().is_side_solid(BlockDirection::Up)
                {
                    return false;
                }

                if self.feature.generate(
                    chunk,
                    block_registry,
                    min_y,
                    height,
                    feature_name,
                    random,
                    *working_pos,
                ) {
                    self.place_dirt(chunk, block_registry, random, pos, pos.0.y + y);
                    return true;
                }
            }
        }

        false
    }

    fn place_dirt<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        origin: BlockPos,
        target_height: i32,
    ) {
        let origin_x = origin.0.x;
        let origin_z = origin.0.z;

        for y in origin.0.y..target_height {
            self.place_rooted_dirt(
                chunk,
                block_registry,
                random,
                origin_x,
                origin_z,
                BlockPos::new(origin_x, y, origin_z),
            );
        }
    }

    fn place_rooted_dirt<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        origin_x: i32,
        origin_z: i32,
        mut working_pos: BlockPos,
    ) {
        let root_radius = self.root_radius.max(1);

        for _ in 0..self.root_placement_attempts {
            working_pos = working_pos.add(
                random.next_bounded_i32(root_radius) - random.next_bounded_i32(root_radius),
                0,
                random.next_bounded_i32(root_radius) - random.next_bounded_i32(root_radius),
            );

            if self
                .root_replaceable
                .test(block_registry, chunk, &working_pos)
            {
                chunk.set_block_state(
                    &working_pos.0,
                    self.root_state_provider
                        .get(random, working_pos, chunk, block_registry),
                );
            }

            working_pos.0.x = origin_x;
            working_pos.0.z = origin_z;
        }
    }

    fn place_roots<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        let root_radius = self.hanging_root_radius.max(1);
        let vertical_span = self.hanging_roots_vertical_span.max(1);

        for _ in 0..self.hanging_root_placement_attempts {
            let working_pos = pos.add(
                random.next_bounded_i32(root_radius) - random.next_bounded_i32(root_radius),
                random.next_bounded_i32(vertical_span) - random.next_bounded_i32(vertical_span),
                random.next_bounded_i32(root_radius) - random.next_bounded_i32(root_radius),
            );

            if chunk.is_air(&working_pos.0) {
                let above = working_pos.up();
                let above_state = GenerationCache::get_block_state(chunk, &above.0);
                if above_state.to_state().is_side_solid(BlockDirection::Down) {
                    chunk.set_block_state(
                        &working_pos.0,
                        self.hanging_root_state_provider.get(
                            random,
                            working_pos,
                            chunk,
                            block_registry,
                        ),
                    );
                }
            }
        }
    }
}
