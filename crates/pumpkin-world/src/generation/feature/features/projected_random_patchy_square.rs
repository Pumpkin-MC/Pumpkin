use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{
        block_predicate::BlockPredicate, block_state_provider::BlockStateProvider,
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct ProjectedRandomPatchySquareFeature {
    pub block: BlockStateProvider,
    pub project_through: BlockPredicate,
    pub size: IntProvider,
    pub max_projection_height: i32,
}

impl ProjectedRandomPatchySquareFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let size = self.size.get(random);
        let bound = size * size + 1;

        for dx in -size..=size {
            for dz in -size..=size {
                let probability = dx.abs() * dz.abs();
                if random.next_bounded_i32(bound) < bound - probability {
                    let mut base_pos = pos.add(dx, 0, dz);
                    let mut drop = self.max_projection_height;

                    while self
                        .project_through
                        .test(block_registry, chunk, &base_pos.down())
                    {
                        base_pos = base_pos.down();
                        drop -= 1;
                        if drop <= 0 {
                            break;
                        }
                    }

                    if let Some(state) =
                        self.block
                            .get_optional(block_registry, chunk, random, base_pos)
                    {
                        chunk.set_block_state(&base_pos.0, state);
                    }
                }
            }
        }

        true
    }
}
