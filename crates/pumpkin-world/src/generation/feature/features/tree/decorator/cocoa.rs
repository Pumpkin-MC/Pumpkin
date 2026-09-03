use pumpkin_data::{Block, BlockDirection, BlockState, block_properties::CocoaLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct CocoaTreeDecorator {
    pub probability: f32,
}

impl CocoaTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
    ) {
        if log_positions.is_empty() || random.next_f32() >= self.probability {
            return;
        }

        let tree_y = log_positions[0].0.y;
        for pos in log_positions.iter().filter(|p| p.0.y - tree_y <= 2) {
            for facing in BlockDirection::horizontal_worldgen() {
                if random.next_f32() <= 0.25 {
                    let dir = BlockDirection::from_cardinal_direction(facing);
                    let opposite = dir.opposite();
                    let cocoa_pos = pos.offset(opposite.to_offset());
                    if chunk.is_air(&cocoa_pos.0) {
                        let props = CocoaLikeProperties {
                            age: random.next_bounded_i32(3) as u8,
                            facing,
                        };
                        let state_id = props.to_state_id(&Block::COCOA);
                        chunk.set_block_state(&cocoa_pos.0, BlockState::from_id(state_id));
                    }
                }
            }
        }
    }
}
