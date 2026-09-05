use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{Axis, CreakingHeartLikeProperties, CreakingHeartState},
    tag,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct CreakingHeartTreeDecorator {
    pub probability: f32,
}

impl CreakingHeartTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
    ) {
        if log_positions.is_empty() || random.next_f32() >= self.probability {
            return;
        }

        let mut shuffled = log_positions.to_vec();
        for i in (1..shuffled.len()).rev() {
            let j = random.next_bounded_i32((i + 1) as i32) as usize;
            shuffled.swap(i, j);
        }

        for pos in shuffled {
            let mut all_logs = true;
            for dir in BlockDirection::all() {
                let neighbor = pos.offset(dir.to_offset());
                let block_id = GenerationCache::get_block_state(chunk, &neighbor.0).to_block_id();
                if !block_id.has_tag(tag::Block::MINECRAFT_LOGS) {
                    all_logs = false;
                    break;
                }
            }

            if all_logs {
                let props = CreakingHeartLikeProperties {
                    r#axis: Axis::Y,
                    r#creaking_heart_state: CreakingHeartState::Dormant,
                    r#natural: true,
                };
                let state_id = props.to_state_id(&Block::CREAKING_HEART);
                chunk.set_block_state(&pos.0, BlockState::from_id(state_id));
                break;
            }
        }
    }
}
