use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::{
    generation::{
        feature::placed_features::{PLACED_FEATURES, PlacedFeatureWrapper},
        proto_chunk::GenerationCache,
    },
    world::WorldPortalExt,
};

pub struct OverlayFeature {
    pub features: Vec<PlacedFeatureWrapper>,
}

impl OverlayFeature {
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
        let mut placed_any = false;
        for feature_wrapper in &self.features {
            let (target_feature_name, placed_feature) = match feature_wrapper {
                PlacedFeatureWrapper::Named(name) => (*name, PLACED_FEATURES.get(name)),
                PlacedFeatureWrapper::Direct(feature) => (feature_name, Some(feature)),
            };
            if let Some(placed_feature) = placed_feature {
                placed_any |= placed_feature.generate(
                    chunk,
                    block_registry,
                    min_y,
                    height,
                    target_feature_name,
                    random,
                    origin,
                );
            }
        }
        placed_any
    }
}
