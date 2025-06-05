use std::sync::Arc;

use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{
    ProtoChunk, generation::feature::placed_features::PlacedFeatureWrapper, level::Level,
    world::BlockRegistryExt,
};

#[derive(Deserialize)]
pub struct RandomBooleanFeature {
    feature_true: Box<PlacedFeatureWrapper>,
    feature_false: Box<PlacedFeatureWrapper>,
}

impl RandomBooleanFeature {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        level: &Arc<Level>,
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
        Box::pin(feature.get().generate(
            chunk,
            level,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        ))
        .await
    }
}
