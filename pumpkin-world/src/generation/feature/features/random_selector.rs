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
pub struct RandomFeature {
    features: Vec<RandomFeatureEntry>,
    default: Box<PlacedFeatureWrapper>,
}

#[derive(Deserialize)]
struct RandomFeatureEntry {
    feature: PlacedFeatureWrapper,
    chance: f32,
}

impl RandomFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
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
        for feature in &self.features {
            if random.next_f32() >= feature.chance {
                continue;
            }
            return feature.feature.get().generate(
                chunk,
                level,
                block_registry,
                min_y,
                height,
                feature_name,
                random,
                pos,
            );
        }
        self.default.get().generate(
            chunk,
            level,
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        )
    }
}
