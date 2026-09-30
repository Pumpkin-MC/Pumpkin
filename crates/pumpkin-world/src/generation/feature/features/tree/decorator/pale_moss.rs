use pumpkin_data::{Block, BlockState, block_properties::PaleHangingMossLikeProperties};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct PaleMossTreeDecorator {
    pub leaves_probability: f32,
    pub trunk_probability: f32,
    pub ground_probability: f32,
}

impl PaleMossTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        log_positions: &[BlockPos],
        foliage_positions: &[BlockPos],
    ) {
        if log_positions.is_empty() {
            return;
        }

        for &pos in log_positions {
            if random.next_f32() < self.trunk_probability {
                let down = pos.down();
                if chunk.is_air(&down.0) {
                    Self::add_moss_hanger(chunk, random, down);
                }
            }
        }

        for &pos in foliage_positions {
            if random.next_f32() < self.leaves_probability {
                let down = pos.down();
                if chunk.is_air(&down.0) {
                    Self::add_moss_hanger(chunk, random, down);
                }
            }
        }
    }

    fn add_moss_hanger<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        mut pos: BlockPos,
    ) {
        while chunk.is_air(&pos.down().0) && random.next_f32() >= 0.5 {
            let props = PaleHangingMossLikeProperties { tip: false };
            let state_id = props.to_state_id(&Block::PALE_HANGING_MOSS);
            chunk.set_block_state(&pos.0, BlockState::from_id(state_id));
            pos = pos.down();
        }

        let props = PaleHangingMossLikeProperties { tip: true };
        let state_id = props.to_state_id(&Block::PALE_HANGING_MOSS);
        chunk.set_block_state(&pos.0, BlockState::from_id(state_id));
    }
}
