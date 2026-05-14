use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, VineLikeProperties},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

pub struct LeavesVineTreeDecorator {
    pub probability: f32,
}

impl LeavesVineTreeDecorator {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        foliage_positions: &[BlockPos],
    ) {
        for pos in foliage_positions {
            if random.next_f32() < self.probability {
                let target = pos.west();
                if chunk.is_air(&target.0) {
                    Self::place_vines(chunk, target, |vine| vine.east = true);
                }
            }
            if random.next_f32() < self.probability {
                let target = pos.east();
                if chunk.is_air(&target.0) {
                    Self::place_vines(chunk, target, |vine| vine.west = true);
                }
            }
            if random.next_f32() < self.probability {
                let target = pos.north();
                if chunk.is_air(&target.0) {
                    Self::place_vines(chunk, target, |vine| vine.south = true);
                }
            }
            if random.next_f32() < self.probability {
                let target = pos.south();
                if chunk.is_air(&target.0) {
                    Self::place_vines(chunk, target, |vine| vine.north = true);
                }
            }
        }
    }

    fn place_vines<T: GenerationCache>(
        chunk: &mut T,
        start: BlockPos,
        configure_face: impl Fn(&mut VineLikeProperties),
    ) {
        let mut vine = VineLikeProperties::default(&Block::VINE);
        configure_face(&mut vine);
        let state = BlockState::from_id(vine.to_state_id(&Block::VINE));
        chunk.set_block_state(&start.0, state);

        let mut current = start.down();
        for _ in 0..4 {
            if !chunk.is_air(&current.0) {
                break;
            }
            chunk.set_block_state(&current.0, state);
            current = current.down();
        }
    }
}
