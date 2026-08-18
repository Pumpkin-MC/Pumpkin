use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct MultifaceGrowthFeature;

impl MultifaceGrowthFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut generated = false;
        let directions = [
            (1, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ];
        for &(dx, dy, dz) in &directions {
            let neighbor = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
            let state = GenerationCache::get_block_state(chunk, &neighbor.0);
            if state.to_block_id() != Block::AIR.id && chunk.is_air(&pos.0) {
                chunk.set_block_state(&pos.0, Block::GLOW_LICHEN.default_state);
                generated = true;
                break;
            }
        }
        generated
    }
}
