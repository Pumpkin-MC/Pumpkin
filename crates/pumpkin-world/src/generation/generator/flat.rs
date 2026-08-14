use pumpkin_codecs::codec::FieldDecode;
use pumpkin_codecs::{DataResult, Decode, DynamicOps};
use pumpkin_data::structures::{Structure, StructurePlacementType};
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::{
    Block,
    dimension::Dimension,
    structures::{StructurePlacementCalculator, StructureSet},
};

use pumpkin_registry::{DataKey, ROOT};
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl as _, get_carver_seed};

use super::{ChunkGenerator, ChunkGeneratorDecode};
use crate::generation::positions::chunk_pos;
use crate::generation::structure::placement::should_generate_structure;
use crate::generation::structure::structures::{
    StructureGeneratorContext, StructureInstance, create_chunk_random,
};
use crate::generation::structure::{generate_structure_position, try_generate_structure};
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

#[derive(Clone, Debug)]
pub struct FlatLayer {
    pub block: String,
    pub height: i32,
}

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

    pub fn set_flat_structure_starts(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Biomes);

        let global_cache = &self.global_structure_cache;
        let calculator = &self.structure_calculator;
        let mut height_sampler =
            crate::generation::generator::flat::FlatHeightSampler::new(self.surface_y());

        let Some(root) = ROOT.get() else {
            chunk.stage = StagedChunkEnum::StructureStart;
            return;
        };

        for (key, allowed_biomes) in self
            .structure_overrides
            .iter()
            .zip(&self.structure_allowed_biomes)
        {
            let Ok(set) = key.get_blocking(root) else {
                continue;
            };

            if !should_generate_structure(
                &set.placement,
                calculator,
                chunk.x,
                chunk.z,
                global_cache,
                chunk,
                allowed_biomes,
            ) {
                continue;
            }

            let mut candidates = set.structures.to_vec();
            let mut total_weight: u32 = candidates.iter().map(|entry| entry.weight).sum();
            let carver_seed = get_carver_seed(self.seed, chunk.x, chunk.z);
            let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed));

            while !candidates.is_empty() {
                let mut roll = random.next_bounded_i32(total_weight as i32);
                let mut selected_idx = 0;

                for (candidate_idx, entry) in candidates.iter().enumerate() {
                    roll -= entry.weight as i32;
                    if roll < 0 {
                        selected_idx = candidate_idx;
                        break;
                    }
                }

                let entry = &candidates[selected_idx];
                let structure = Structure::get(&entry.structure);
                let position = global_cache.get_or_compute_structure_start(
                    entry.structure,
                    chunk.x,
                    chunk.z,
                    || {
                        try_generate_structure(
                            &entry.structure,
                            structure,
                            self.seed as i64,
                            chunk,
                            self.surface_y(),
                            Some(&mut height_sampler),
                        )
                    },
                );

                if let Some(position) = position {
                    chunk
                        .structure_starts
                        .insert(entry.structure, StructureInstance::Start(position));
                    break;
                }

                let failed = candidates.remove(selected_idx);
                total_weight -= failed.weight;
            }
        }

        chunk.stage = StagedChunkEnum::StructureStart;
    }

    #[expect(clippy::too_many_lines)]
    pub fn set_flat_structure_references(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::StructureStart);

        let global_cache = &self.global_structure_cache;
        let calculator = &self.structure_calculator;
        let start_x = chunk_pos::start_block_x(chunk.x);
        let start_z = chunk_pos::start_block_z(chunk.z);
        let end_x = start_x + 15;
        let end_z = start_z + 15;
        let seed = self.seed as i64;
        let chunk_min_y = chunk.bottom_y() as i32;
        let flat_biome = pumpkin_data::chunk::Biome::from_name(
            self.biome.strip_prefix("minecraft:").unwrap_or(&self.biome),
        )
        .unwrap_or(&pumpkin_data::chunk::Biome::PLAINS);

        let mut references = Vec::new();
        let mut height_sampler =
            crate::generation::generator::flat::FlatHeightSampler::new(self.surface_y());

        let Some(root) = ROOT.get() else {
            chunk.stage = StagedChunkEnum::StructureReferences;
            return;
        };

        for (key, allowed_biomes) in self
            .structure_overrides
            .iter()
            .zip(&self.structure_allowed_biomes)
        {
            let Ok(set) = key.get_blocking(root) else {
                continue;
            };
            let mut candidate_chunks = Vec::new();

            match &set.placement.placement_type {
                StructurePlacementType::RandomSpread(spread) => {
                    let region_x = pumpkin_util::math::floor_div(chunk.x, spread.spacing);
                    let region_z = pumpkin_util::math::floor_div(chunk.z, spread.spacing);

                    for rx in (region_x - 1)..=(region_x + 1) {
                        for rz in (region_z - 1)..=(region_z + 1) {
                            candidate_chunks.push(
                                crate::generation::structure::placement::get_structure_chunk_in_region(
                                    spread,
                                    seed,
                                    rx,
                                    rz,
                                    set.placement.salt,
                                ),
                            );
                        }
                    }
                }
                StructurePlacementType::ConcentricRings(rings) => {
                    let allowed_biomes = ProtoChunk::get_allowed_biomes(&set);
                    let strongholds = global_cache.get_or_calculate_strongholds(
                        seed,
                        rings,
                        chunk,
                        &allowed_biomes,
                    );
                    for &(cx, cz) in strongholds {
                        if (cx - chunk.x).abs() <= 8 && (cz - chunk.z).abs() <= 8 {
                            candidate_chunks.push((cx, cz));
                        }
                    }
                }
            }

            for (candidate_chunk_x, candidate_chunk_z) in candidate_chunks {
                if !should_generate_structure(
                    &set.placement,
                    calculator,
                    candidate_chunk_x,
                    candidate_chunk_z,
                    global_cache,
                    chunk,
                    allowed_biomes,
                ) {
                    continue;
                }

                for entry in set.structures {
                    let structure = Structure::get(&entry.structure);
                    let start_data = global_cache.get_or_compute_structure_start(
                        entry.structure,
                        candidate_chunk_x,
                        candidate_chunk_z,
                        || {
                            let context = StructureGeneratorContext {
                                seed,
                                chunk_x: candidate_chunk_x,
                                chunk_z: candidate_chunk_z,
                                random: create_chunk_random(
                                    seed,
                                    candidate_chunk_x,
                                    candidate_chunk_z,
                                ),
                                sea_level: self.surface_y(),
                                min_y: chunk_min_y,
                                height_sampler: Some(&mut height_sampler),
                                structure_key: Some(entry.structure),
                            };

                            let position =
                                generate_structure_position(&entry.structure, structure, context)?;
                            let allowed = get_tag_ids(
                                RegistryKey::WorldgenBiome,
                                structure
                                    .biomes
                                    .strip_prefix('#')
                                    .unwrap_or(structure.biomes),
                            )?;
                            allowed
                                .contains(&(flat_biome.id as u16))
                                .then_some(position)
                        },
                    );

                    if let Some(start_data) = start_data
                        && start_data
                            .get_bounding_box()
                            .intersects_raw_xz(start_x, start_z, end_x, end_z)
                    {
                        references.push((entry.structure, start_data.collector.clone()));
                        break;
                    }
                }
            }
        }

        for (key, collector) in references {
            chunk
                .structure_starts
                .entry(key)
                .or_insert_with(|| StructureInstance::Reference(collector));
        }

        chunk.stage = StagedChunkEnum::StructureReferences;
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
        self.step_to_biomes(chunk);
    }

    fn step_to_structure_start(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        let chunk = cache.chunks[chunk_index].get_proto_chunk_mut();
        self.set_flat_structure_starts(chunk);
    }

    fn step_to_structure_references(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        _block_registry: &dyn WorldPortalExt,
    ) {
        let chunk = cache.chunks[chunk_index].get_proto_chunk_mut();
        self.set_flat_structure_references(chunk);
    }

    fn rebuild_structure_starts(&self, chunk: &mut ProtoChunk) {
        self.set_flat_structure_starts(chunk);
    }

    fn rebuild_structure_references(&self, chunk: &mut ProtoChunk) {
        self.set_flat_structure_references(chunk);
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
        ProtoChunk::generate_structures_only(cache, block_registry, self.seed as i64);
    }
}
