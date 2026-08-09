use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    math::position::{BlockPos, BlockPosIterator},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{block_state_provider::BlockStateProvider, proto_chunk::GenerationCache},
    world::WorldPortalExt,
};

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
        origin: BlockPos,
    ) -> bool {
        if origin.0.y < i32::from(min_y) + 5 {
            return false;
        }

        let xr = 2 + random.next_bounded_i32(2);
        let zr = 2 + random.next_bounded_i32(2);

        for block_pos in BlockPosIterator::new(
            origin.0.x - xr,
            origin.0.y,
            origin.0.z - zr,
            origin.0.x + xr,
            origin.0.y + 1,
            origin.0.z + zr,
        ) {
            let xd = origin.0.x - block_pos.0.x;
            let zd = origin.0.z - block_pos.0.z;
            let should_place = (xd * xd + zd * zd) as f32
                <= random.next_f32() * 10.0 - random.next_f32() * 6.0
                || random.next_f32() < 0.031;
            if should_place {
                self.try_place_block(chunk, block_registry, random, block_pos);
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
        if below_state.id.to_block_id() == Block::DIRT_PATH.id {
            random.next_bool()
        } else {
            below_state.is_side_solid(BlockDirection::Up)
        }
    }

    fn try_place_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        block_pos: BlockPos,
    ) {
        if chunk.is_air(&block_pos.0) && Self::may_place_on(chunk, block_pos, random) {
            let state = self
                .state_provider
                .get(random, block_pos, chunk, block_registry);
            chunk.set_block_state(&block_pos.0, state);
        }
    }
}
