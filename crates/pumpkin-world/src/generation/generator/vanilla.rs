use pumpkin_codecs::DataResult;
use pumpkin_data::{
    BlockState, chunk_gen_settings::GenerationSettings, dimension::Dimension,
    structures::StructurePlacementCalculator,
};
use pumpkin_registry::{DataKey, DataKeyRef, ROOT, Registry};
use pumpkin_util::world_seed::Seed;
use std::{ops::Deref, sync::Arc};

use super::{ChunkGenerator, ChunkGeneratorDecode, NoiseGeneratorConfig};
use crate::{
    ProtoChunk,
    biome::BiomeSupplier,
    chunk_system::generation_cache::Cache,
    generation::{
        GlobalRandomConfig, noise::router::proto_noise_router::ProtoNoiseRouters,
        proto_chunk::TerrainCache, structure::placement::GlobalStructureCache,
    },
    world::WorldPortalExt,
};

pub(crate) enum GenerationSettingsRef {
    Registry(DataKeyRef<'static, GenerationSettings>),
    Static(&'static GenerationSettings),
}

impl Deref for GenerationSettingsRef {
    type Target = GenerationSettings;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Registry(settings) => settings,
            Self::Static(settings) => settings,
        }
    }
}

pub struct VanillaGenerator {
    pub random_config: GlobalRandomConfig,
    pub base_router: ProtoNoiseRouters,
    pub dimension: Dimension,
    pub settings: DataKey<GenerationSettings>,
    pub biome_source: Box<dyn BiomeSupplier>,
    pub biome_mixer_seed: i64,
    pub terrain_cache: TerrainCache,
    pub default_block: &'static BlockState,
    pub global_structure_cache: GlobalStructureCache,
    pub structure_calculator: StructurePlacementCalculator,
    legacy_settings: Option<&'static GenerationSettings>,
}

impl VanillaGenerator {
    fn from_parts(
        seed: Seed,
        dimension: Dimension,
        settings_key: DataKey<GenerationSettings>,
        settings: &GenerationSettings,
        biome_source: Box<dyn BiomeSupplier>,
        legacy_settings: Option<&'static GenerationSettings>,
    ) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0, settings.legacy_random_source);
        let terrain_cache = TerrainCache::from_random(&random_config);
        let default_block = settings.default_block;
        let base_router = ProtoNoiseRouters::generate(settings.base_router, &random_config);
        let biome_mixer_seed = crate::biome::hash_seed(seed.0);

        Self {
            random_config,
            base_router,
            dimension,
            settings: settings_key,
            biome_source,
            biome_mixer_seed,
            terrain_cache,
            default_block,
            global_structure_cache: GlobalStructureCache::new(),
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            legacy_settings,
        }
    }

    pub fn from_config(
        seed: Seed,
        dimension: Dimension,
        config: NoiseGeneratorConfig,
    ) -> Result<Self, String> {
        let root = ROOT
            .get()
            .ok_or_else(|| "Root registry is not initialized".to_string())?;

        let settings = config
            .settings
            .get_blocking(root)
            .map_err(|error| format!("Failed to resolve noise settings: {error}"))?;

        let source_type = config
            .biome_source
            .source_type
            .get_blocking(root)
            .map_err(|error| format!("Failed to resolve biome source type: {error}"))?;

        let biome_source = source_type
            .decode(config.biome_source.input, &pumpkin_nbt::nbt_ops::NbtOps)
            .into_result()
            .ok_or_else(|| "Failed to decode biome source".to_string())?;

        Ok(Self::from_parts(
            seed,
            dimension,
            config.settings,
            &settings,
            biome_source,
            None,
        ))
    }

    #[must_use]
    #[allow(clippy::expect_used)]
    pub(crate) fn structure_sets() -> DataKeyRef<'static, Arc<dyn Registry>> {
        static STRUCTURE_SETS: DataKey<Arc<dyn Registry>> =
            DataKey::new("minecraft:worldgen/minecraft:structure_set");

        let root = ROOT
            .get()
            .expect("VanillaGenerator decoded only after the root registry is initialized");
        STRUCTURE_SETS
            .get_blocking(root)
            .expect("Structure set registry must exist for vanilla world generation")
    }

    #[must_use]
    #[allow(clippy::expect_used)]
    pub(crate) fn settings(&self) -> GenerationSettingsRef {
        if let Some(settings) = self.legacy_settings {
            return GenerationSettingsRef::Static(settings);
        }

        let root = ROOT
            .get()
            .expect("VanillaGenerator decoded only after the root registry is initialized");
        let settings = self
            .settings
            .get_blocking(root)
            .expect("VanillaGenerator noise settings were resolved during construction");
        GenerationSettingsRef::Registry(settings)
    }

    #[must_use]
    pub fn new(seed: Seed, dimension: Dimension) -> Self {
        let (name, settings, biome_source): (
            &'static str,
            &'static GenerationSettings,
            Box<dyn BiomeSupplier>,
        ) = if dimension == Dimension::THE_NETHER {
            (
                "nether",
                &GenerationSettings::NETHER,
                Box::new(crate::biome::MultiNoiseBiomeSupplier::NETHER),
            )
        } else if dimension == Dimension::THE_END {
            (
                "end",
                &GenerationSettings::END,
                Box::new(crate::biome::end::TheEndBiomeSupplier),
            )
        } else {
            (
                "overworld",
                &GenerationSettings::OVERWORLD,
                Box::new(crate::biome::MultiNoiseBiomeSupplier::OVERWORLD),
            )
        };

        Self::from_parts(
            seed,
            dimension,
            DataKey::owned(format!(
                "minecraft:worldgen/minecraft:noise_settings/minecraft:{name}"
            )),
            settings,
            biome_source,
            Some(settings),
        )
    }
}

impl ChunkGeneratorDecode for VanillaGenerator {
    type Config = NoiseGeneratorConfig;

    fn from_config(seed: Seed, dimension: Dimension, config: Self::Config) -> DataResult<Self> {
        Self::from_config(seed, dimension, config)
            .map_or_else(DataResult::new_error, DataResult::new_success)
    }
}

impl ChunkGenerator for VanillaGenerator {
    fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    fn seed(&self) -> u64 {
        self.random_config.seed
    }

    fn generation_bounds(&self) -> (u16, i8) {
        let dimension = self.dimension();
        let shape = self.settings().shape.trim_height(
            dimension.min_y as i8,
            (dimension.min_y + dimension.height) as u16,
        );
        (shape.height, shape.min_y)
    }

    fn default_block(&self) -> &'static BlockState {
        self.default_block
    }

    fn biome_mixer_seed(&self) -> i64 {
        self.biome_mixer_seed
    }

    fn sea_level(&self) -> i32 {
        self.settings().sea_level
    }

    fn global_structure_cache(&self) -> Option<&GlobalStructureCache> {
        Some(&self.global_structure_cache)
    }

    fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        self.step_to_biomes(chunk);
    }

    fn step_to_structure_start(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        let chunk = cache.chunks[chunk_index].get_proto_chunk_mut();
        self.set_structure_starts(chunk);
    }

    fn step_to_structure_references(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        let chunk = cache.chunks[chunk_index].get_proto_chunk_mut();
        self.set_structure_references(chunk);
    }

    fn rebuild_structure_starts(&self, chunk: &mut ProtoChunk) {
        self.set_structure_starts(chunk);
    }

    fn rebuild_structure_references(&self, chunk: &mut ProtoChunk) {
        self.set_structure_references(chunk);
    }

    fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        self.step_to_noise(chunk);
    }

    fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        self.step_to_surface(chunk);
    }

    fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        self.step_to_carvers(chunk);
    }

    fn step_to_features(
        &self,
        cache: &mut Cache,
        _chunk_index: usize,
        block_registry: &dyn WorldPortalExt,
    ) {
        Self::generate_features_and_structure(cache, block_registry, &self.random_config);
    }
}
