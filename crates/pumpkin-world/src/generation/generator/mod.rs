use pumpkin_data::BlockState;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::material_rule::MaterialRule;
use pumpkin_data::noise_router::BaseNoiseRouters;
use pumpkin_data::noise_settings::NoiseSettings;

use super::noise::router::proto_noise_router::ProtoNoiseRouters;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed};

pub mod biome_finder;
pub mod structure_finder;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use rustc_hash::FxHashMap;

use std::sync::Arc;

use crate::chunk_system::StagedChunkEnum;
use crate::generation::proto_chunk::ProtoChunk;

pub mod flat;

#[derive(Clone, Debug)]
pub struct FlatLayer {
    pub block: String,
    pub height: i32,
}

pub trait CustomChunkGenerator: Send + Sync {
    fn dimension(&self) -> &Dimension;
    fn seed(&self) -> u64;
    fn default_block(&self) -> &'static BlockState {
        pumpkin_data::Block::AIR.default_state
    }
    fn biome_mixer_seed(&self) -> i64 {
        0
    }
    fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        None
    }

    fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Biomes;
    }

    fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Noise;
    }

    fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Surface;
    }

    fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Carvers;
    }

    fn step_to_features(
        &self,
        cache: &mut crate::chunk_system::generation_cache::Cache,
        _block_registry: &dyn crate::world::WorldPortalExt,
    ) {
        let mid = ((cache.size * cache.size) >> 1) as usize;
        cache.chunks[mid].get_proto_chunk_mut().stage = StagedChunkEnum::Features;
    }

    fn set_structure_starts(&self, _chunk: &mut ProtoChunk) {}
    fn set_structure_references(&self, _chunk: &mut ProtoChunk) {}
}

pub enum WorldGenerator {
    Noise(Box<VanillaGenerator>),
    Flat(Box<flat::FlatGenerator>),
    Custom(Arc<dyn CustomChunkGenerator>),
}

impl WorldGenerator {
    #[must_use]
    pub fn dimension(&self) -> &Dimension {
        match self {
            Self::Noise(noise_gen) => &noise_gen.dimension,
            Self::Flat(flat_gen) => &flat_gen.dimension,
            Self::Custom(custom_gen) => custom_gen.dimension(),
        }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        match self {
            Self::Noise(noise_gen) => noise_gen.random_config.seed,
            Self::Flat(flat_gen) => flat_gen.seed,
            Self::Custom(custom_gen) => custom_gen.seed(),
        }
    }

    #[must_use]
    pub fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        match self {
            Self::Noise(noise_gen) => Some(&noise_gen.global_structure_cache),
            Self::Flat(_) => None,
            Self::Custom(custom_gen) => custom_gen.global_structure_cache(),
        }
    }

    #[must_use]
    pub fn find_spawn_position(&self) -> pumpkin_util::math::position::BlockPos {
        match self {
            Self::Noise(noise_gen) => noise_gen.find_spawn_position(),
            _ => pumpkin_util::math::position::BlockPos::ZERO,
        }
    }
}

pub struct VanillaGenerator {
    pub random_config: GlobalRandomConfig,
    pub base_router: ProtoNoiseRouters,
    pub dimension: Dimension,
    pub settings: &'static NoiseSettings,
    pub surface_rule: &'static MaterialRule,
    pub biome_supplier: crate::biome::ActiveBiomeSupplier,
    pub biome_mixer_seed: i64,

    pub terrain_cache: TerrainCache,

    pub default_block: &'static BlockState,

    pub global_structure_cache: crate::generation::structure::placement::GlobalStructureCache,
    pub structure_calculator: StructurePlacementCalculator,
    pub structure_allowed_biomes: FxHashMap<usize, Vec<u16>>,
    pub enabled_structure_sets: Option<rustc_hash::FxHashSet<usize>>,
}

impl VanillaGenerator {
    #[must_use]
    pub fn find_spawn_position(&self) -> pumpkin_util::math::position::BlockPos {
        if self.settings.spawn_target.is_empty() {
            return pumpkin_util::math::position::BlockPos::ZERO;
        }
        let mut sampler =
            crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler::generate(
                &self.base_router.multi_noise,
            );
        crate::biome::position_finder::SpawnFinder::find_spawn_position(
            self.settings.spawn_target,
            &mut sampler,
        )
    }

    #[must_use]
    pub fn new_with_settings(
        seed: Seed,
        dimension: Dimension,
        settings_name: Option<&str>,
    ) -> Self {
        Self::new_with_all_settings(seed, dimension, settings_name, None, None)
    }

    #[must_use]
    pub fn new_with_all_settings(
        seed: Seed,
        dimension: Dimension,
        settings_name: Option<&str>,
        biome_source: Option<&crate::world_info::BiomeSource>,
        structure_overrides: Option<&[String]>,
    ) -> Self {
        let settings = settings_name
            .and_then(NoiseSettings::from_name)
            .unwrap_or_else(|| NoiseSettings::from_dimension(&dimension));
        let surface_rule = settings_name
            .and_then(MaterialRule::from_name)
            .unwrap_or_else(|| MaterialRule::from_dimension(&dimension));
        let biome_supplier =
            crate::biome::ActiveBiomeSupplier::from_biome_source(biome_source, &dimension);
        let random_config = GlobalRandomConfig::new(seed.0, settings.legacy_random_source);

        let base = settings_name
            .and_then(BaseNoiseRouters::from_name)
            .unwrap_or_else(|| BaseNoiseRouters::from_dimension(&dimension));
        let terrain_cache = TerrainCache::from_random(&random_config);

        let default_block = settings.default_block;
        let base_router = ProtoNoiseRouters::generate(base, &random_config);
        let biome_mixer_seed = crate::biome::hash_seed(seed.0);

        let mut structure_allowed_biomes = FxHashMap::default();
        for (i, set) in StructureSet::ALL.iter().enumerate() {
            structure_allowed_biomes.insert(
                i,
                crate::generation::proto_chunk::ProtoChunk::get_allowed_biomes(set),
            );
        }

        let enabled_structure_sets: Option<rustc_hash::FxHashSet<usize>> =
            structure_overrides.map(|overrides| {
                overrides
                    .iter()
                    .filter_map(|s| {
                        let clean = s.strip_prefix("minecraft:").unwrap_or(s);
                        StructureSet::NAMES.iter().position(|&name| name == clean)
                    })
                    .collect()
            });

        let global_structure_cache =
            crate::generation::structure::placement::GlobalStructureCache::new();

        if dimension == Dimension::OVERWORLD {
            let pumpkin_data::structures::StructurePlacementType::ConcentricRings(rings) =
                &StructureSet::STRONGHOLDS.placement.placement_type
            else {
                unreachable!()
            };
            let strongholds =
                crate::generation::structure::placement::GlobalStructureCache::calculate_strongholds(
                    seed.0 as i64,
                    rings,
                    &base_router.multi_noise,
                );
            global_structure_cache.init_strongholds(strongholds);
        }

        Self {
            random_config,
            base_router,
            dimension,
            settings,
            surface_rule,
            biome_supplier,
            biome_mixer_seed,
            terrain_cache,
            default_block,
            global_structure_cache,
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            structure_allowed_biomes,
            enabled_structure_sets,
        }
    }
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed, dimension: Dimension) -> Self {
        Self::new_with_settings(seed, dimension, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_generator_settings() {
        let seed = Seed(42);

        // Default overworld
        let default_gen = VanillaGenerator::new_with_settings(seed, Dimension::OVERWORLD, None);
        assert_eq!(*default_gen.settings, NoiseSettings::OVERWORLD);

        // Amplified preset
        let amplified_gen = VanillaGenerator::new_with_settings(
            seed,
            Dimension::OVERWORLD,
            Some("minecraft:amplified"),
        );
        assert_eq!(*amplified_gen.settings, NoiseSettings::AMPLIFIED);

        // Large biomes preset
        let large_biomes_gen = VanillaGenerator::new_with_settings(
            seed,
            Dimension::OVERWORLD,
            Some("minecraft:large_biomes"),
        );
        assert_eq!(*large_biomes_gen.settings, NoiseSettings::LARGE_BIOMES);

        // Nether
        let nether_gen = VanillaGenerator::new_with_settings(
            seed,
            Dimension::THE_NETHER,
            Some("minecraft:nether"),
        );
        assert_eq!(*nether_gen.settings, NoiseSettings::NETHER);

        // End
        let end_gen =
            VanillaGenerator::new_with_settings(seed, Dimension::THE_END, Some("minecraft:end"));
        assert_eq!(*end_gen.settings, NoiseSettings::END);
    }

    #[test]
    fn vanilla_generator_structure_overrides() {
        let seed = Seed(42);

        // No overrides: None means all structure sets enabled
        let all_gen =
            VanillaGenerator::new_with_all_settings(seed, Dimension::OVERWORLD, None, None, None);
        assert!(all_gen.enabled_structure_sets.is_none());

        // Overrides with villages only
        let village_overrides = vec!["minecraft:villages".to_string()];
        let filtered_gen = VanillaGenerator::new_with_all_settings(
            seed,
            Dimension::OVERWORLD,
            None,
            None,
            Some(&village_overrides),
        );
        let enabled = filtered_gen
            .enabled_structure_sets
            .expect("should have enabled structure set filter");
        let village_idx = StructureSet::NAMES
            .iter()
            .position(|&n| n == "villages")
            .unwrap();
        let mineshaft_idx = StructureSet::NAMES
            .iter()
            .position(|&n| n == "mineshafts")
            .unwrap();
        assert!(enabled.contains(&village_idx));
        assert!(!enabled.contains(&mineshaft_idx));
    }

    #[test]
    fn vanilla_generator_fixed_biome_source() {
        use crate::biome::BiomeSupplier;

        let seed = Seed(42);
        let biome_source = crate::world_info::BiomeSource::Fixed {
            biome: "minecraft:desert".to_string(),
            biome_type: "minecraft:fixed".to_string(),
        };

        let desert_gen = VanillaGenerator::new_with_all_settings(
            seed,
            Dimension::OVERWORLD,
            None,
            Some(&biome_source),
            None,
        );

        let mut sampler =
            crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler::generate(
                &desert_gen.base_router.multi_noise,
            );
        let sampled_biome = desert_gen.biome_supplier.biome(0, 64, 0, &mut sampler);
        assert_eq!(sampled_biome.registry_id, "desert");
    }
}
