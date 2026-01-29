use pumpkin_data::Block;
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::generation::proto_chunk::GenerationCache;
use crate::{block::BlockStateCodec, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct ReplaceBlobsFeature {
    target: BlockStateCodec,
    state: BlockStateCodec,
    radius: IntProvider,
}

impl ReplaceBlobsFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let target = self.target.get_block();
        let state = self.state.get_state();
        let Some(pos) = Self::move_down_to_target(pos, chunk, target) else {
            return false;
        };
        let x = self.radius.get(random);
        let y = self.radius.get(random);
        let z = self.radius.get(random);
        let distance = x.max(y.max(z));

        let mut result = false;

        for iter_pos in BlockPos::iterate_outwards(pos, x, y, z) {
            if iter_pos.manhattan_distance(pos) > distance {
                break;
            }
            let current_state = GenerationCache::get_block_state(chunk, &iter_pos.0);
            if current_state.to_block() != target {
                continue;
            }
            chunk.set_block_state(&iter_pos.0, state);
            result = true;
        }

        result
    }

    fn move_down_to_target<T: GenerationCache>(
        mut pos: BlockPos,
        chunk: &T,
        target: &'static Block,
    ) -> Option<BlockPos> {
        while pos.0.y > chunk.bottom_y() as i32 + 1 {
            let state = GenerationCache::get_block_state(chunk, &pos.0);
            if state.to_block() == target {
                return Some(pos);
            }

            pos = pos.down();
        }
        None
    }
}
