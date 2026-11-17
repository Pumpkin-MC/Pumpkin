use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::block_state_provider::BlockStateProvider;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct BlockPileFeature {
    pub state_provider: BlockStateProvider,
}

impl BlockPileFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if pos.0.y < min_y as i32 + 5 {
            return false;
        }

        let xr = 2 + random.next_bounded_i32(2);
        let zr = 2 + random.next_bounded_i32(2);

        for block_pos in BlockPos::iterate(pos.add(-xr, 0, -zr), pos.add(xr, 1, zr)) {
            let xd = pos.0.x - block_pos.0.x;
            let zd = pos.0.z - block_pos.0.z;
            if (xd * xd + zd * zd) as f32 <= random.next_f32() * 10.0 - random.next_f32() * 6.0
                || random.next_f32() < 0.031
            {
                self.try_place_block(chunk, block_registry, block_pos, random);
            }
        }

        true
    }

    fn may_place_on<T: GenerationCache>(
        chunk: &T,
        block_pos: BlockPos,
        random: &mut RandomGenerator,
    ) -> bool {
        let below = block_pos.down();
        let below_state = GenerationCache::get_block_state(chunk, &below.0).to_state();
        if below_state.id == Block::DIRT_PATH.default_state.id {
            random.next_bool()
        } else {
            below_state.is_side_solid(BlockDirection::Up)
        }
    }

    fn try_place_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        block_pos: BlockPos,
        random: &mut RandomGenerator,
    ) {
        if chunk.is_air(&block_pos.0) && Self::may_place_on(chunk, block_pos, random) {
            let state = self
                .state_provider
                .get(random, block_pos, chunk, block_registry);
            chunk.set_block_state(&block_pos.0, state);
        }
    }
}
