use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::{generation::feature::placed_features::PlacedFeatureWrapper, world::WorldPortalExt};

pub struct WeightedRandomFeature {
    pub features: Vec<WeightedRandomFeatureEntry>,
    pub total_weight: i32,
}

pub struct WeightedRandomFeatureEntry {
    pub feature: PlacedFeatureWrapper,
    pub weight: i32,
}

impl WeightedRandomFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        min_y: i8,
        height: u16,
        feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if self.total_weight <= 0 {
            return false;
        }
        let mut roll = random.next_bounded_i32(self.total_weight);
        for entry in &self.features {
            roll -= entry.weight;
            if roll < 0 {
                if let Some(feature) = entry.feature.get() {
                    return feature.generate(
                        chunk,
                        block_registry,
                        min_y,
                        height,
                        feature_name,
                        random,
                        pos,
                    );
                }
                return false;
            }
        }
        false
    }
}
