use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::feature::placed_features::PlacedFeatureWrapper, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct RandomBooleanFeature {
    feature_true: Box<PlacedFeatureWrapper>,
    feature_false: Box<PlacedFeatureWrapper>,
}

impl RandomBooleanFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn BlockRegistryExt,
        min_y: i8,
        height: u16,
        feature_name: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let val = random.next_bool();
        let feature = if val {
            &self.feature_true
        } else {
            &self.feature_false
        };
        feature.get().generate(
            chunk,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        )
    }
}
