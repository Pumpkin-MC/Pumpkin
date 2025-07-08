use pumpkin_data::{BlockDirection, block_properties::get_block_by_state_id, tag::Tagable};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{block_state_provider::BlockStateProvider, height_limit::HeightLimitView},
    world::BlockRegistryExt,
};

#[derive(Deserialize)]
pub struct NetherForestVegetationFeature {
    state_provider: BlockStateProvider,
    spread_width: i32,
    spread_height: i32,
}

impl NetherForestVegetationFeature {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let state = chunk.get_block_state(&pos.down().0);

        if !state.to_block().is_tagged_with("minecraft:nylium").unwrap() {
            return false;
        }
        let mut result = false;

        for _ in 0..self.spread_width * self.spread_width {
            let pos = pos.add(
                random.next_bounded_i32(self.spread_width)
                    - random.next_bounded_i32(self.spread_width),
                random.next_bounded_i32(self.spread_height)
                    - random.next_bounded_i32(self.spread_height),
                random.next_bounded_i32(self.spread_width)
                    - random.next_bounded_i32(self.spread_width),
            );
            let nether_state = self.state_provider.get(random, pos);
            let nether_block = get_block_by_state_id(nether_state.id);
            if !chunk.is_air(&pos.0)
                || pos.0.y <= chunk.bottom_y() as i32
                || block_registry
                    .can_place_at(nether_block, chunk, &pos, BlockDirection::Up)
                    .await
            {
                continue;
            }
            chunk.set_block_state(&pos.0, nether_state);
            result = true;
        }

        result
    }
}
