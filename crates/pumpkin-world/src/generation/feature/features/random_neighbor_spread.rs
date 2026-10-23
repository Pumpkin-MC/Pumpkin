use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::RandomGenerator,
};

use crate::{
    generation::{
        block_predicate::BlockPredicate, block_state_provider::BlockStateProvider,
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct RandomNeighborSpreadFeature {
    pub block: BlockStateProvider,
    pub accepted_neighbors: Vec<BlockId>,
    pub can_replace: BlockPredicate,
    pub attempts: IntProvider,
    pub xz_offset: IntProvider,
    pub y_offset: IntProvider,
}

impl RandomNeighborSpreadFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> bool {
        let origin_state = self.block.get(random, origin, chunk, block_registry);
        chunk.set_block_state(&origin.0, origin_state);

        let attempts = self.attempts.get(random);
        for _ in 0..attempts {
            let dx = self.xz_offset.get(random);
            let dy = self.y_offset.get(random);
            let dz = self.xz_offset.get(random);
            let place_pos = origin.add(dx, dy, dz);

            if self.can_replace.test(block_registry, chunk, &place_pos) {
                let mut neighbours = 0;
                for dir in BlockDirection::all() {
                    let neighbor_pos = place_pos.offset(dir.to_offset());
                    let neighbor_id =
                        GenerationCache::get_block_state(chunk, &neighbor_pos.0).to_block_id();
                    if self.accepted_neighbors.contains(&neighbor_id) {
                        neighbours += 1;
                        if neighbours > 1 {
                            break;
                        }
                    }
                }

                if neighbours == 1 {
                    let state = self.block.get(random, place_pos, chunk, block_registry);
                    chunk.set_block_state(&place_pos.0, state);
                }
            }
        }

        true
    }
}
