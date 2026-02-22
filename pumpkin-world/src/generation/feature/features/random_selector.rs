use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::feature::placed_features::PlacedFeatureWrapper, world::BlockRegistryExt};

pub struct RandomFeature {
    pub features: Vec<RandomFeatureEntry>,
    pub default: Box<PlacedFeatureWrapper>,
}

pub struct RandomFeatureEntry {
    pub feature: PlacedFeatureWrapper,
    pub chance: f32,
}

impl RandomFeature {
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
        for feature in &self.features {
            if random.next_f32() >= feature.chance {
                continue;
            }
            return feature.feature.get().generate(
                chunk,
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
            block_registry,
            min_y,
            height,
            feature_name,
            random,
            pos,
        )
    }
}
