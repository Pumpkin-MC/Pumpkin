use std::sync::Arc;

use pumpkin_data::chunk::Biome;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::structures::{Structure, StructurePlacementType, StructureSet, WeightedEntry};
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{
    RandomGenerator, RandomImpl, get_carver_seed, get_decorator_seed, xoroshiro128::Xoroshiro,
};

use crate::biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::chunk_system::StagedChunkEnum;
use crate::generation::structure::lazily_generate_structure;
use crate::generation::structure::placement::should_generate_structure;
use crate::generation::structure::structures::{
    StructureGeneratorContext, StructureInstance, create_chunk_random,
};
use crate::generation::structure::try_generate_structure;
use crate::generation::{
    GlobalRandomConfig,
    blender::{Blender, BlenderImpl},
    feature::placed_features::PLACED_FEATURES,
    noise::router::multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
    positions::chunk_pos,
};
use crate::world::WorldPortalExt;

use super::{ActiveSupplier, GenerationCache, ProtoChunk};

impl ProtoChunk {
    pub fn generate_features_and_structure<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn WorldPortalExt,
        random_config: &GlobalRandomConfig,
    ) {
        let (center_x, center_z, min_y, height, biomes_in_chunk) = {
            let chunk = cache.get_center_chunk();
            let mut unique_biomes = Vec::with_capacity(4);
            for &biome_id in &chunk.flat_biome_map {
                if !unique_biomes.contains(&biome_id) {
                    unique_biomes.push(biome_id);
                }
            }
            (
                chunk.x,
                chunk.z,
                chunk.bottom_y() as i32,
                chunk.height() as i32,
                unique_biomes,
            )
        };

        let start_block_x = chunk_pos::start_block_x(center_x);
        let start_block_z = chunk_pos::start_block_z(center_z);
        let origin_pos = BlockPos::new(start_block_x, min_y, start_block_z);

        let population_seed =
            Xoroshiro::get_population_seed(random_config.seed, start_block_x, start_block_z);

        for step in 0..11 {
            Self::generate_structure_step(
                cache,
                block_registry,
                step,
                population_seed,
                random_config.seed as i64,
            );

            let mut features_to_run = Vec::new();
            for biome_id in &biomes_in_chunk {
                if let Some(biome) = Biome::from_id(*biome_id)
                    && let Some(features_at_step) = biome.features.get(step)
                {
                    for &feature_id in *features_at_step {
                        features_to_run.push(feature_id);
                    }
                }
            }

            features_to_run.sort_unstable();
            features_to_run.dedup();

            for (p, feature_enum) in features_to_run.into_iter().enumerate() {
                if let Some(feature) = PLACED_FEATURES.get(&feature_enum) {
                    let decorator_seed = get_decorator_seed(population_seed, p as u64, step as u64);
                    let mut random =
                        RandomGenerator::Xoroshiro(Xoroshiro::from_seed(decorator_seed));

                    feature.generate(
                        cache,
                        block_registry,
                        min_y as i8,
                        height as u16,
                        feature_enum,
                        &mut random,
                        origin_pos,
                    );
                }
            }
        }

        cache.get_center_chunk_mut().stage = StagedChunkEnum::Features;
    }

    fn generate_structure_step<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn WorldPortalExt,
        step: usize,
        population_seed: u64,
        world_seed: i64,
    ) {
        let mut tasks = Vec::new();
        {
            let center_chunk = cache.get_center_chunk();
            let center_x = center_chunk.x;
            let center_z = center_chunk.z;

            for (id, instance) in &center_chunk.structure_starts {
                let s = Structure::get(id);
                if s.step.ordinal() != step {
                    continue;
                }

                match instance {
                    StructureInstance::Start(pos) => tasks.push(pos.collector.clone()),
                    StructureInstance::Reference(collector) => {
                        let collector_arc = collector.clone();
                        if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                            tasks.push(collector_arc);
                        }
                    }
                }
            }

            let radius = 8;
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx == 0 && dz == 0 {
                        continue;
                    }

                    let neighbor_x = center_x + dx;
                    let neighbor_z = center_z + dz;

                    if let Some(neighbor) = cache.try_get_proto_chunk(neighbor_x, neighbor_z) {
                        for (id, instance) in &neighbor.structure_starts {
                            let s = Structure::get(id);
                            if s.step.ordinal() != step {
                                continue;
                            }

                            match instance {
                                StructureInstance::Start(pos) => {
                                    let start_x = chunk_pos::start_block_x(center_x);
                                    let start_z = chunk_pos::start_block_z(center_z);
                                    let end_x = start_x + 15;
                                    let end_z = start_z + 15;

                                    if pos
                                        .get_bounding_box()
                                        .intersects_raw_xz(start_x, start_z, end_x, end_z)
                                    {
                                        let collector_arc = pos.collector.clone();
                                        if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                                            tasks.push(collector_arc);
                                        }
                                    }
                                }
                                StructureInstance::Reference(collector) => {
                                    let collector_arc = collector.clone();
                                    if !tasks.iter().any(|t| Arc::ptr_eq(t, &collector_arc)) {
                                        tasks.push(collector_arc);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let decorator_seed = get_decorator_seed(population_seed, 0, step as u64);
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(decorator_seed));

        let chunk = cache.get_center_chunk_mut();
        for collector_arc in tasks {
            let mut collector = collector_arc.lock().unwrap();
            collector.generate_in_chunk(chunk, block_registry, &mut random, world_seed);
        }
    }

    #[must_use]
    pub fn get_allowed_biomes(set: &StructureSet) -> Vec<u16> {
        let mut allowed_biomes = Vec::new();
        for entry in set.structures {
            let structure = Structure::get(&entry.structure);
            if let Some(biomes) = get_tag_ids(
                RegistryKey::WorldgenBiome,
                structure
                    .biomes
                    .strip_prefix('#')
                    .unwrap_or(structure.biomes),
            ) {
                allowed_biomes.extend_from_slice(biomes);
            }
        }
        allowed_biomes
    }

    pub fn set_structure_starts(
        &mut self,
        generator: &crate::generation::generator::VanillaGenerator,
    ) {
        debug_assert_eq!(self.stage, StagedChunkEnum::Biomes);
        let random_config = &generator.random_config;
        let settings = generator.settings;
        let global_cache = &generator.global_structure_cache;
        let calculator = &generator.structure_calculator;

        let seed = random_config.seed;

        let mut height_sampler = crate::generation::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
            &generator.base_router.surface_estimator,
            &crate::generation::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                crate::generation::biome_coords::from_block(crate::generation::positions::chunk_pos::start_block_x(self.x)),
                crate::generation::biome_coords::from_block(crate::generation::positions::chunk_pos::start_block_z(self.z)),
                4,
                settings.shape.min_y as i32,
                settings.shape.height as i32,
                (settings.shape.height / settings.shape.vertical_cell_block_count() as u16) as usize,
            ),
        );

        for (i, set) in StructureSet::ALL.iter().enumerate() {
            let allowed_biomes = &generator.structure_allowed_biomes[&i];

            if !should_generate_structure(
                &set.placement,
                calculator,
                self.x,
                self.z,
                global_cache,
                self,
                allowed_biomes,
            ) {
                continue;
            }

            if set.structures.len() == 1 {
                if let Some(entry) = set.structures.first() {
                    self.try_set_structure_start(
                        settings.sea_level,
                        entry,
                        random_config,
                        &mut height_sampler,
                    );
                }
                continue;
            }

            let mut candidates = set.structures.to_vec();
            let carver_seed = get_carver_seed(seed, self.x, self.z);
            let mut random: RandomGenerator =
                RandomGenerator::Xoroshiro(Xoroshiro::from_seed(carver_seed));

            let mut total_weight: u32 = candidates.iter().map(|e| e.weight).sum();

            while !candidates.is_empty() {
                let mut roll = random.next_bounded_i32(total_weight as i32);
                let mut selected_idx = 0;

                for (i, entry) in candidates.iter().enumerate() {
                    roll -= entry.weight as i32;
                    if roll < 0 {
                        selected_idx = i;
                        break;
                    }
                }

                let selected_entry = &candidates[selected_idx];

                if self.try_set_structure_start(
                    settings.sea_level,
                    selected_entry,
                    random_config,
                    &mut height_sampler,
                ) {
                    break;
                }

                let failed_entry = candidates.remove(selected_idx);
                total_weight -= failed_entry.weight;
            }
        }
        self.stage = StagedChunkEnum::StructureStart;
    }

    fn try_set_structure_start(
        &mut self,
        sea_level: i32,
        entry: &WeightedEntry,
        random_config: &GlobalRandomConfig,
        height_sampler: &mut crate::generation::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler<'_>,
    ) -> bool {
        let structure = Structure::get(&entry.structure);
        let position = try_generate_structure(
            &entry.structure,
            structure,
            random_config.seed as i64,
            self,
            sea_level,
            Some(height_sampler),
        );

        if let Some(pos) = position {
            self.structure_starts
                .insert(entry.structure, StructureInstance::Start(pos));
            return true;
        }
        false
    }

    #[expect(clippy::too_many_lines)]
    pub fn set_structure_references(
        &mut self,
        generator: &crate::generation::generator::VanillaGenerator,
    ) {
        debug_assert_eq!(self.stage, StagedChunkEnum::StructureStart);
        let random_config = &generator.random_config;
        let settings = generator.settings;
        let dimension = &generator.dimension;
        let noise_router = &generator.base_router;
        let global_cache = &generator.global_structure_cache;

        let start_x = chunk_pos::start_block_x(self.x);
        let start_z = chunk_pos::start_block_z(self.z);
        let end_x = start_x + 15;
        let end_z = start_z + 15;

        let seed = random_config.seed as i64;

        let active_supplier = if *dimension == Dimension::THE_END {
            ActiveSupplier::End(TheEndBiomeSupplier)
        } else if *dimension == Dimension::THE_NETHER {
            ActiveSupplier::Nether(MultiNoiseBiomeSupplier::NETHER)
        } else {
            ActiveSupplier::Overworld(MultiNoiseBiomeSupplier::OVERWORLD)
        };

        let base_supplier: &dyn BiomeSupplier = match &active_supplier {
            ActiveSupplier::End(s) => s,
            ActiveSupplier::Nether(s) | ActiveSupplier::Overworld(s) => s,
        };
        let blender = Blender::empty();
        let biome_supplier = blender.get_biome_supplier(base_supplier);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);

        let mut height_sampler = crate::generation::noise::router::surface_height_sampler::SurfaceHeightEstimateSampler::generate(
            &noise_router.surface_estimator,
            &crate::generation::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                crate::generation::biome_coords::from_block(start_x),
                crate::generation::biome_coords::from_block(start_z),
                4,
                settings.shape.min_y as i32,
                settings.shape.height as i32,
                (settings.shape.height / settings.shape.vertical_cell_block_count() as u16) as usize,
            ),
        );

        let mut references = Vec::new();
        // Constant across every chunk in the dimension, so hoist it out of the loop
        // and out of the (cached) structure-start computation below.
        let chunk_min_y = self.bottom_y() as i32;
        let calculator = &generator.structure_calculator;

        for (set_index, set) in StructureSet::ALL.iter().enumerate() {
            let set_allowed_biomes = &generator.structure_allowed_biomes[&set_index];
            let mut candidate_chunks = Vec::new();

            match &set.placement.placement_type {
                StructurePlacementType::RandomSpread(spread) => {
                    // Vanilla ChunkGenerator.createReferences (ChunkGenerator.java
                    // l.450-458) scans the actual starts of every chunk within 8
                    // chunks of this one. Cover every placement region that can
                    // contain a candidate chunk inside that window: for spacing-1
                    // sets (minecraft:mineshafts, spacing 1) this is the full
                    // 17x17 chunk neighborhood, where a region +-1 scan would
                    // only reach chunks +-1 away and truncate sprawling
                    // structures at that boundary.
                    let region_min_x = pumpkin_util::math::floor_div(self.x - 8, spread.spacing);
                    let region_max_x = pumpkin_util::math::floor_div(self.x + 8, spread.spacing);
                    let region_min_z = pumpkin_util::math::floor_div(self.z - 8, spread.spacing);
                    let region_max_z = pumpkin_util::math::floor_div(self.z + 8, spread.spacing);

                    for rx in region_min_x..=region_max_x {
                        for rz in region_min_z..=region_max_z {
                            candidate_chunks.push(
                                crate::generation::structure::placement::get_structure_chunk_in_region(
                                    spread,
                                    seed,
                                    rx,
                                    rz,
                                    set.placement.salt,
                                )
                            );
                        }
                    }
                }
                StructurePlacementType::ConcentricRings(rings) => {
                    let allowed_biomes = Self::get_allowed_biomes(set);
                    let strongholds = global_cache.get_or_calculate_strongholds(
                        seed,
                        rings,
                        self,
                        &allowed_biomes,
                    );
                    for &(cx, cz) in strongholds {
                        if (cx - self.x).abs() <= 8 && (cz - self.z).abs() <= 8 {
                            candidate_chunks.push((cx, cz));
                        }
                    }
                }
            }

            for (candidate_chunk_x, candidate_chunk_z) in candidate_chunks {
                if (candidate_chunk_x - self.x).abs() <= 8
                    && (candidate_chunk_z - self.z).abs() <= 8
                {
                    // Vanilla only ever creates a start where
                    // StructurePlacement.isStructureChunk passes (placement chunk
                    // AND frequency reduction, StructurePlacement.java l.77-83;
                    // gated in ChunkGenerator.createStructures l.398), and
                    // createReferences propagates only those actual starts. The
                    // same gate must apply to recomputed candidates here:
                    // minecraft:mineshafts has spacing 1 / frequency 0.004, so
                    // without it every biome-valid chunk becomes a phantom
                    // mineshaft start.
                    if !should_generate_structure(
                        &set.placement,
                        calculator,
                        candidate_chunk_x,
                        candidate_chunk_z,
                        global_cache,
                        self,
                        set_allowed_biomes,
                    ) {
                        continue;
                    }
                    for entry in set.structures {
                        let structure = Structure::get(&entry.structure);

                        // A structure's placement depends only on its start chunk and the
                        // world seed, so cache it: otherwise every surrounding chunk whose
                        // references overlap it would re-run the (expensive) jigsaw
                        // expansion. `context` is only built on a cache miss.
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
                                    sea_level: settings.sea_level,
                                    min_y: chunk_min_y,
                                    max_y: chunk_min_y + self.height() as i32 - 1,
                                    height_sampler: Some(&mut height_sampler),
                                    structure_key: Some(entry.structure),
                                };
                                lazily_generate_structure(
                                    &entry.structure,
                                    structure,
                                    context,
                                    &biome_supplier,
                                    &mut multi_noise_sampler,
                                )
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
        }

        for (key, pos) in references {
            self.structure_starts
                .entry(key)
                .or_insert_with(|| StructureInstance::Reference(pos));
        }

        self.stage = StagedChunkEnum::StructureReferences;
    }
}
