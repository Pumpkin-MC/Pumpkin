use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::block_state_provider::BlockStateProvider, world::WorldPortalExt};

pub struct AttachedToLogsTreeDecorator {
    pub probability: f32,
    pub block_provider: BlockStateProvider,
    pub directions: Vec<BlockDirection>,
}

impl AttachedToLogsTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
    ) {
        if self.directions.is_empty() || log_positions.is_empty() {
            return;
        }

        let mut shuffled = log_positions.to_vec();
        for i in (1..shuffled.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            shuffled.swap(i, j);
        }

        for log_pos in shuffled {
            let dir_idx = random.next_bounded_i32(self.directions.len() as i32) as usize;
            let direction = self.directions[dir_idx];
            let placement_pos = log_pos.offset(direction.to_offset());

            if random.next_f32() <= self.probability && chunk.is_air(&placement_pos.0) {
                chunk.set_block_state(
                    &placement_pos.0,
                    self.block_provider
                        .get(random, placement_pos, chunk, block_registry),
                );
            }
        }
    }
}
