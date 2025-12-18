use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, DoubleBlockHalf, TallSeagrassLikeProperties},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SeagrassFeature {
    probability: f32,
}

impl SeagrassFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let y = chunk.ocean_floor_height_exclusive(pos.0.x + x, pos.0.z + z);
        let top_pos = BlockPos::new(pos.0.x + x, y, pos.0.z + z);
        if GenerationCache::get_block_state(chunk, &top_pos.0).to_block() == &Block::WATER {
            let tall = random.next_f64() < self.probability as f64;
            if tall {
                let tall_pos = top_pos.up();
                if GenerationCache::get_block_state(chunk, &tall_pos.0).to_block() == &Block::WATER
                {
                    let mut props = TallSeagrassLikeProperties::default(&Block::TALL_SEAGRASS);
                    props.half = DoubleBlockHalf::Upper;
                    chunk.set_block_state(&top_pos.0, Block::TALL_SEAGRASS.default_state);
                    chunk.set_block_state(
                        &tall_pos.0,
                        BlockState::from_id(props.to_state_id(&Block::TALL_SEAGRASS)),
                    );
                }
            } else {
                chunk.set_block_state(&top_pos.0, Block::SEAGRASS.default_state);
            }
            return true;
        }
        false
    }
}
