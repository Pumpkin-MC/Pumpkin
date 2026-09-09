use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::feature::features::ore::OreTarget;
use crate::generation::proto_chunk::GenerationCache;

pub struct ReplaceSingleBlockFeature {
    pub targets: Vec<OreTarget>,
}

impl ReplaceSingleBlockFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let block_state = GenerationCache::get_block_state(chunk, &pos.0);
        for target in &self.targets {
            if target.target.test(block_state, random) {
                chunk.set_block_state(&pos.0, target.state);
                break;
            }
        }
        true
    }
}
