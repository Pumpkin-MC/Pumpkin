use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct WeepingVinesFeature;

impl WeepingVinesFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let ceiling_pos = pos.up();
        let ceiling_block = GenerationCache::get_block_state(chunk, &ceiling_pos.0).to_block_id();
        if ceiling_block == Block::AIR.id {
            return false;
        }

        let length = random.next_bounded_i32(4) + 3;
        for i in 0..length {
            let target = BlockPos::new(pos.0.x, pos.0.y - i, pos.0.z);
            if chunk.is_air(&target.0) {
                chunk.set_block_state(&target.0, Block::WEEPING_VINES.default_state);
            } else {
                break;
            }
        }
        true
    }
}
