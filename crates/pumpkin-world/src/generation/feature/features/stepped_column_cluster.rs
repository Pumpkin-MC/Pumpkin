use pumpkin_data::tag;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{
        block_predicate::BlockPredicate, block_state_provider::BlockStateProvider,
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct SteppedColumnClusterFeature {
    pub block: BlockStateProvider,
    pub continue_through: BlockPredicate,
    pub can_replace: BlockPredicate,
    pub cannot_place_on: tag::Tag,
    pub cluster_reach: IntProvider,
    pub column_count: IntProvider,
    pub column_reach: IntProvider,
    pub height: IntProvider,
}

impl SteppedColumnClusterFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> bool {
        let lava_sea_level = chunk.get_sea_level();
        if !self.can_place_at(block_registry, chunk, origin) {
            return false;
        }

        let column_height = self.height.get(random);
        let cluster_reach = column_height.min(self.cluster_reach.get(random));
        let count = self.column_count.get(random);
        let mut placed = false;

        let width = 2 * cluster_reach + 1;
        let depth = 2 * cluster_reach + 1;

        for _ in 0..count {
            let rx = random.next_bounded_i32(width);
            let _ry = random.next_bounded_i32(1);
            let rz = random.next_bounded_i32(depth);
            let pos = BlockPos::new(
                origin.0.x - cluster_reach + rx,
                origin.0.y,
                origin.0.z - cluster_reach + rz,
            );
            let blocks_to_place_y =
                column_height - (pos.0.x - origin.0.x).abs() - (pos.0.z - origin.0.z).abs();
            if blocks_to_place_y >= 0 {
                let reach = self.column_reach.get(random);
                placed |= self.place_column(
                    chunk,
                    block_registry,
                    random,
                    lava_sea_level,
                    pos,
                    blocks_to_place_y,
                    reach,
                );
            }
        }

        placed
    }

    #[expect(clippy::too_many_arguments)]
    fn place_column<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        lava_sea_level: i32,
        origin: BlockPos,
        column_height: i32,
        reach: i32,
    ) -> bool {
        let mut placed_any = false;

        for z in (origin.0.z - reach)..=(origin.0.z + reach) {
            for x in (origin.0.x - reach)..=(origin.0.x + reach) {
                let pos = BlockPos::new(x, origin.0.y, z);
                let step_limit = (x - origin.0.x).abs() + (z - origin.0.z).abs();

                let column_pos = if self.can_replace.test(block_registry, chunk, &pos) {
                    self.find_surface(block_registry, chunk, lava_sea_level, pos, step_limit)
                } else {
                    self.find_air(chunk, pos, step_limit)
                };

                if let Some(column_pos) = column_pos {
                    let mut blocks_y = column_height - step_limit / 2;
                    let mut cursor = column_pos;

                    while blocks_y >= 0 {
                        if self.can_replace.test(block_registry, chunk, &cursor) {
                            let state = self.block.get(random, cursor, chunk, block_registry);
                            chunk.set_block_state(&cursor.0, state);
                            cursor = cursor.up();
                            placed_any = true;
                        } else {
                            if !self.continue_through.test(block_registry, chunk, &cursor) {
                                break;
                            }

                            cursor = cursor.up();
                        }

                        blocks_y -= 1;
                    }
                }
            }
        }

        placed_any
    }

    fn find_surface<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        _lava_sea_level: i32,
        pos: BlockPos,
        mut limit: i32,
    ) -> Option<BlockPos> {
        let min_y = chunk.bottom_y() as i32;
        let mut cursor = pos;
        while cursor.0.y > min_y + 1 && limit > 0 {
            limit -= 1;
            if self.can_place_at(block_registry, chunk, cursor) {
                return Some(cursor);
            }

            cursor = cursor.down();
        }

        None
    }

    fn can_place_at<T: GenerationCache>(
        &self,
        block_registry: &dyn WorldPortalExt,
        chunk: &T,
        cursor: BlockPos,
    ) -> bool {
        if !self.can_replace.test(block_registry, chunk, &cursor) {
            return false;
        }

        let below_pos = cursor.down();
        let block_state_id = GenerationCache::get_block_state(chunk, &below_pos.0);
        let block_state = block_state_id.to_state();
        !block_state.is_air() && !block_state_id.to_block_id().has_tag(self.cannot_place_on)
    }

    fn find_air<T: GenerationCache>(
        &self,
        chunk: &T,
        pos: BlockPos,
        mut limit: i32,
    ) -> Option<BlockPos> {
        let max_y = chunk.bottom_y() as i32 + chunk.height() as i32 - 1;
        let mut cursor = pos;
        while cursor.0.y <= max_y && limit > 0 {
            limit -= 1;
            let block_state_id = GenerationCache::get_block_state(chunk, &cursor.0);
            if block_state_id.to_block_id().has_tag(self.cannot_place_on) {
                return None;
            }

            if block_state_id.to_state().is_air() {
                return Some(cursor);
            }

            cursor = cursor.up();
        }

        None
    }
}
