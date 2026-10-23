use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    generation::{
        block_predicate::BlockPredicate,
        block_state_provider::BlockStateProvider,
        feature::placed_features::{PLACED_FEATURES, PlacedFeatureWrapper},
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct SingleBlockPillarFeature {
    pub block: BlockStateProvider,
    pub can_replace: BlockPredicate,
    pub direction: BlockDirection,
    pub chance_to_continue: f32,
    pub cap_feature: Option<PlacedFeatureWrapper>,
}

impl SingleBlockPillarFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        min_y: i8,
        height: u16,
        feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> bool {
        let mut pos = origin;

        while self.can_replace.test(block_registry, chunk, &pos)
            && random.next_f32() < self.chance_to_continue
            && !chunk.out_of_height(pos.0.y as i16)
        {
            let state = self.block.get(random, pos, chunk, block_registry);
            chunk.set_block_state(&pos.0, state);
            pos = pos.offset(self.direction.to_offset());
        }

        pos = pos.offset(self.direction.opposite().to_offset());
        if let Some(cap) = &self.cap_feature {
            let (cap_feature_name, placed_feature) = match cap {
                PlacedFeatureWrapper::Named(name) => (*name, PLACED_FEATURES.get(name)),
                PlacedFeatureWrapper::Direct(feature) => (feature_name, Some(feature)),
            };
            if let Some(feature) = placed_feature {
                feature.generate(
                    chunk,
                    block_registry,
                    min_y,
                    height,
                    cap_feature_name,
                    random,
                    pos,
                );
            }
        }

        true
    }
}
