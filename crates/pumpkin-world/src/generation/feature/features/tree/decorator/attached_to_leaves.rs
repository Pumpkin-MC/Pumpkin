use std::collections::HashSet;

use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{block_state_provider::BlockStateProvider, proto_chunk::GenerationCache},
    world::WorldPortalExt,
};

pub struct AttachedToLeavesTreeDecorator {
    pub probability: f32,
    pub exclusion_radius_xz: i32,
    pub exclusion_radius_y: i32,
    pub block_provider: BlockStateProvider,
    pub required_empty_blocks: i32,
    pub directions: Vec<BlockDirection>,
}

impl AttachedToLeavesTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        foliage_positions: &[BlockPos],
    ) {
        if self.directions.is_empty() || foliage_positions.is_empty() {
            return;
        }

        let mut blacklist: HashSet<BlockPos> = HashSet::new();

        let mut shuffled = foliage_positions.to_vec();
        for i in (1..shuffled.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            shuffled.swap(i, j);
        }

        for leaf_pos in shuffled {
            let dir_idx = random.next_bounded_i32(self.directions.len() as i32) as usize;
            let direction = self.directions[dir_idx];
            let placement_pos = leaf_pos.offset(direction.to_offset());

            if !blacklist.contains(&placement_pos)
                && random.next_f32() < self.probability
                && self.has_required_empty_blocks(chunk, leaf_pos, direction)
            {
                for dx in -self.exclusion_radius_xz..=self.exclusion_radius_xz {
                    for dy in -self.exclusion_radius_y..=self.exclusion_radius_y {
                        for dz in -self.exclusion_radius_xz..=self.exclusion_radius_xz {
                            blacklist.insert(placement_pos.add(dx, dy, dz));
                        }
                    }
                }

                chunk.set_block_state(
                    &placement_pos.0,
                    self.block_provider
                        .get(random, placement_pos, chunk, block_registry),
                );
            }
        }
    }

    fn has_required_empty_blocks<T: GenerationCache>(
        &self,
        chunk: &T,
        leaf_pos: BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let offset = direction.to_offset();
        for i in 1..=self.required_empty_blocks {
            let check_pos = leaf_pos.offset(offset * i);
            if !chunk.is_air(&check_pos.0) {
                return false;
            }
        }
        true
    }
}
