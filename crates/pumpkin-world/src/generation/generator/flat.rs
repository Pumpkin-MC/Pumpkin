use pumpkin_codecs::codec::FieldDecode;
use pumpkin_codecs::{DataResult, Decode, DynamicOps};
use pumpkin_data::{
    Block,
    dimension::Dimension,
    structures::{StructurePlacementCalculator, StructureSet},
};

use pumpkin_registry::{DataKey, ROOT};

use super::{ChunkGenerator, ChunkGeneratorDecode, FlatLayer};
use crate::{
    ProtoChunk,
    chunk_system::{StagedChunkEnum, generation_cache::Cache},
    generation::{
        Seed,
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::{placement::GlobalStructureCache, structures::HeightSampler},
    },
    world::WorldPortalExt,
};

pub struct FlatGeneratorConfig {
    pub biome: String,
    pub features: bool,
    pub lakes: bool,
    pub layers: Vec<FlatLayer>,
    pub structure_overrides: Vec<String>,
}

impl Decode for FlatLayer {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            String::decode_field::<O>("block", &map, ops).apply_2(
                |block, height| (Self { block, height }, ops.empty()),
                i32::decode_field::<O>("height", &map, ops),
            )
        })
    }
}

impl Decode for FlatGeneratorConfig {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            String::decode_field::<O>("biome", &map, ops).apply_5(
                |biome, features, lakes, layers, structure_overrides| {
                    (
                        Self {
                            biome,
                            features,
                            lakes,
                            layers,
                            structure_overrides,
                        },
                        ops.empty(),
                    )
                },
                bool::decode_defaulted_field::<O>("features", &map, ops, false, false),
                bool::decode_defaulted_field::<O>("lakes", &map, ops, false, false),
                Vec::<FlatLayer>::decode_field::<O>("layers", &map, ops),
                Vec::<String>::decode_defaulted_field::<O>(
                    "structure_overrides",
                    &map,
                    ops,
                    Vec::new(),
                    false,
                ),
            )
        })
    }
}

pub struct FlatGenerator {
    pub seed: u64,
    pub dimension: Dimension,
    pub layers: Vec<FlatLayer>,
    pub biome: String,
    pub features: bool,
    pub lakes: bool,
    pub structure_overrides: Vec<DataKey<StructureSet>>,
    pub global_structure_cache: GlobalStructureCache,
    pub structure_calculator: StructurePlacementCalculator,
    pub structure_allowed_biomes: Vec<Vec<u16>>,
}

impl FlatGenerator {
    #[must_use]
    pub const fn new(
        seed: Seed,
        dimension: Dimension,
        layers: Vec<FlatLayer>,
        biome: String,
    ) -> Self {
        Self {
            seed: seed.0,
            dimension,
            layers,
            biome,
            features: false,
            lakes: false,
            structure_overrides: Vec::new(),
            global_structure_cache: GlobalStructureCache::new(),
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            structure_allowed_biomes: Vec::new(),
        }
    }

    pub fn from_config(
        seed: Seed,
        dimension: Dimension,
        config: FlatGeneratorConfig,
    ) -> Result<Self, String> {
        let root = ROOT
            .get()
            .ok_or_else(|| "Root registry is not initialized".to_string())?;
        let mut structure_overrides = Vec::with_capacity(config.structure_overrides.len());
        let mut structure_allowed_biomes = Vec::with_capacity(config.structure_overrides.len());

        for identifier in &config.structure_overrides {
            let key = DataKey::<StructureSet>::owned(format!(
                "minecraft:worldgen/minecraft:structure_set/{identifier}"
            ));
            let set = key.get_blocking(root).map_err(|error| {
                format!("Failed to resolve structure set {identifier}: {error}")
            })?;

            structure_allowed_biomes.push(ProtoChunk::get_allowed_biomes(&set));
            structure_overrides.push(key);
        }

        Ok(Self {
            seed: seed.0,
            dimension,
            layers: config.layers,
            biome: config.biome,
            features: config.features,
            lakes: config.lakes,
            structure_overrides,
            global_structure_cache: GlobalStructureCache::new(),
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            structure_allowed_biomes,
        })
    }

    #[must_use]
    pub fn surface_y(&self) -> i32 {
        self.dimension.min_y + self.layers.iter().map(|layer| layer.height).sum::<i32>()
    }

    pub fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        let clean_biome = self.biome.strip_prefix("minecraft:").unwrap_or(&self.biome);
        let biome_id = pumpkin_data::chunk::Biome::from_name(clean_biome)
            .map_or(pumpkin_data::chunk::Biome::PLAINS.id, |b| b.id);
        chunk.flat_biome_map.fill(biome_id);
        chunk.stage = StagedChunkEnum::Biomes;
    }

    pub fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        let start_x = start_block_x(chunk.x);
        let start_z = start_block_z(chunk.z);
        for x in 0..16 {
            for z in 0..16 {
                let mut current_y = chunk.bottom_y() as i32;
                for layer in &self.layers {
                    let block = Block::from_name(&layer.block);
                    let state = block.map_or(Block::AIR.default_state, |b| b.default_state);
                    for _ in 0..layer.height {
                        if current_y < chunk.bottom_y() as i32 + chunk.height() as i32 {
                            chunk.set_block_state(start_x + x, current_y, start_z + z, state);
                            current_y += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        chunk.stage = StagedChunkEnum::Noise;
    }

    pub const fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Surface;
    }

    pub const fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Carvers;
    }
}

impl ChunkGeneratorDecode for FlatGenerator {
    type Config = FlatGeneratorConfig;

    fn from_config(seed: Seed, dimension: Dimension, config: Self::Config) -> DataResult<Self> {
        Self::from_config(seed, dimension, config)
            .map_or_else(DataResult::new_error, DataResult::new_success)
    }
}

pub(crate) struct FlatHeightSampler {
    height: i32,
}

impl FlatHeightSampler {
    pub(crate) const fn new(height: i32) -> Self {
        Self { height }
    }
}

impl HeightSampler for FlatHeightSampler {
    fn estimate_height(&mut self, _block_x: i32, _block_z: i32) -> i32 {
        self.height
    }
}

impl ChunkGenerator for FlatGenerator {
    fn dimension(&self) -> &Dimension {
        &self.dimension
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        Self::step_to_biomes(self, chunk);
    }

    fn step_to_structure_start(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        cache.chunks[chunk_index]
            .get_proto_chunk_mut()
            .set_flat_structure_starts(self);
    }

    fn step_to_structure_references(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        cache.chunks[chunk_index]
            .get_proto_chunk_mut()
            .set_flat_structure_references(self);
    }

    fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        Self::step_to_noise(self, chunk);
    }

    fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        Self::step_to_surface(self, chunk);
    }

    fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        Self::step_to_carvers(self, chunk);
    }

    fn step_to_features(
        &self,
        cache: &mut Cache,
        _chunk_index: usize,
        block_registry: &dyn WorldPortalExt,
    ) {
        ProtoChunk::generate_structures_only(cache, block_registry, self.seed as i64);
    }
}
