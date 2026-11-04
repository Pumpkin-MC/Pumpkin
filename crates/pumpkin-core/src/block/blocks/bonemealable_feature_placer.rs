use pumpkin_data::configured_feature::ConfiguredFeature as FeatureKey;
use pumpkin_data::placed_feature::PlacedFeature;
use pumpkin_data::{Block, BlockId};
use pumpkin_util::random::{RandomGenerator, xoroshiro128::Xoroshiro};
use pumpkin_world::generation::feature::configured_features::CONFIGURED_FEATURES;
use pumpkin_world::generation::height_limit::HeightLimitView;

use crate::block::{BlockBehaviour, BlockMetadata, BonemealArgs};
use crate::world::generation_cache::WorldGenerationCache;

/// Moss and pale moss: bone meal places their vegetation patch feature on the block above.
pub struct BonemealableFeaturePlacerBlock;

impl BlockMetadata for BonemealableFeaturePlacerBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::MOSS_BLOCK, BlockId::PALE_MOSS_BLOCK].into()
    }
}

impl BlockBehaviour for BonemealableFeaturePlacerBlock {
    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        args.world.get_block_state(&args.position.up()).is_air()
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        // Only biome filters read the placed feature name, and these patches have none,
        // so the placed feature closest to each patch stands in for it.
        let (key, placed) = if args.block == &Block::PALE_MOSS_BLOCK {
            (
                FeatureKey::PaleMossPatchBonemeal,
                PlacedFeature::PaleMossPatch,
            )
        } else {
            (
                FeatureKey::MossPatchBonemeal,
                PlacedFeature::LushCavesVegetation,
            )
        };
        let Some(feature) = CONFIGURED_FEATURES.get(&key) else {
            return;
        };
        let portal = args.world.level.world_portal.load_full();
        let Some(portal) = portal.as_ref() else {
            return;
        };
        let origin = args.position.up();
        let mut cache = WorldGenerationCache::new(args.world.clone(), &origin);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::random()));
        let (min_y, height) = (cache.bottom_y(), cache.height());
        feature.generate(
            &mut cache,
            &**portal,
            min_y,
            height,
            placed,
            &mut random,
            origin,
        );
        cache.apply();
    }
}
