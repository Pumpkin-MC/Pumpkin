use pumpkin_data::{
    Block,
    chunk::Biome,
    structures::{Structure, StructureKeys, StructurePlacementType, StructureSet, WeightedEntry},
};
use pumpkin_util::{
    math::{block_box::BlockBox, position::BlockPos, vector3::Vector3},
    random::{
        RandomGenerator, RandomImpl as _, get_carver_seed, get_decorator_seed,
        xoroshiro128::{Xoroshiro, XoroshiroSplitter},
    },
};

use crate::{
    GlobalRandomConfig, ProtoChunk,
    biome::BiomeSupplier as _,
    chunk_system::StagedChunkEnum,
    generation::{
        biome_coords,
        blender::{Blender, BlenderImpl as _},
        feature::placed_features::PLACED_FEATURES,
        generator::VanillaGenerator,
        noise::{
            CHUNK_DIM, ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK,
            aquifer_sampler::FluidLevel,
            router::{
                multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
                surface_height_sampler::{
                    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
                },
            },
        },
        positions::chunk_pos,
        proto_chunk::{GenerationCache, StandardChunkFluidLevelSampler},
        section_coords,
        structure::{
            lazily_generate_structure,
            placement::{GlobalStructureCache, should_generate_structure},
            structures::{StructureGeneratorContext, StructureInstance, create_chunk_random},
            try_generate_structure,
        },
        surface::{MaterialRuleContext, estimate_surface_height, rule::try_apply_material_rule},
    },
    world::WorldPortalExt,
};

impl VanillaGenerator {
    pub fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Empty);
        let start_x = chunk.start_block_x();
        let start_z = chunk.start_block_z();
        let horizontal_biome_end = biome_coords::from_block(16);
        let multi_noise_config =
            crate::generation::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end as usize,
            );
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&self.base_router.multi_noise, &multi_noise_config);
        self.populate_biomes(chunk, &mut multi_noise_sampler);
        chunk.stage = StagedChunkEnum::Biomes;
    }

    #[expect(clippy::too_many_lines)]
    pub fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::StructureReferences);
        let settings = self.settings();
        let generation_shape = &settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
        let start_x = chunk.start_block_x();
        let start_z = chunk.start_block_z();

        let sampler = StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                settings.sea_level,
                Block::from_state_id(settings.default_fluid.id),
            ),
            FluidLevel::new(-54, &Block::LAVA),
        );

        let mut beardifier_structures = Vec::new();
        let mut beardifier_junctions = Vec::new();
        let mut any_piece_bounding_box: Option<BlockBox> = None;

        let chunk_start_x = chunk.start_block_x();
        let chunk_start_z = chunk.start_block_z();

        for (key, instance) in &chunk.structure_starts {
            let structure = pumpkin_data::structures::Structure::get(key);
            let terrain_adaptation = match structure.terrain_adaptation {
                pumpkin_data::structures::TerrainAdaptation::None => {
                    crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::None
                }
                pumpkin_data::structures::TerrainAdaptation::BeardThin => {
                    crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::BeardThin
                }
                pumpkin_data::structures::TerrainAdaptation::BeardBox => {
                    crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::BeardBox
                }
                pumpkin_data::structures::TerrainAdaptation::Bury => {
                    crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::Bury
                }
                pumpkin_data::structures::TerrainAdaptation::Encapsulate => {
                    crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::Encapsulate
                }
            };

            // Vanilla strictly skips filtering Beardifier parts if adaptation is None early-on
            if terrain_adaptation == crate::generation::noise::router::density_function::beardifier::TerrainAdaptation::None {
                continue;
            }

            let collector = match instance {
                StructureInstance::Start(pos) => &pos.collector,
                StructureInstance::Reference(collector) => collector,
            };

            let collector = collector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for piece in &collector.pieces {
                let bounding_box = piece.get_structure_piece().bounding_box;

                // Match `piece.isCloseToChunk(chunkPos, 12)`
                // Validates if an expansion 12 blocks out covers the chunk borders
                if !bounding_box.intersects_raw_xz(
                    chunk_start_x - 12,
                    chunk_start_z - 12,
                    chunk_start_x + 15 + 12,
                    chunk_start_z + 15 + 12,
                ) {
                    continue;
                }

                let mut ground_level_delta = 0;

                if let Some(jigsaw_piece) = piece.as_any().downcast_ref::<crate::generation::structure::structures::jigsaw::PoolElementStructurePiece>() {
                    // Java only adds to rigids if projection is RIGID
                    if jigsaw_piece.projection == crate::generation::structure::structures::jigsaw::JigsawProjection::Rigid {
                        ground_level_delta = jigsaw_piece.ground_level_delta;
                        any_piece_bounding_box = any_piece_bounding_box.map_or(Some(bounding_box), |mut b| {
                                 b.encompass(&bounding_box);
                                 Some(b)
                             });

                        beardifier_structures.push(
                            crate::generation::noise::router::density_function::beardifier::BeardifierStructure {
                                bounding_box,
                                terrain_adaptation,
                                ground_level_delta,
                            }
                        );
                    }

                    for j in &jigsaw_piece.junctions {
                        let j_x = j.source_x;
                        let j_z = j.source_z;
                        // Junction bounds filter (match vanilla proximity checks)
                        if j_x > chunk_start_x - 12
                            && j_z > chunk_start_z - 12
                            && j_x < chunk_start_x + 15 + 12
                            && j_z < chunk_start_z + 15 + 12
                        {
                            beardifier_junctions.push(
                                crate::generation::noise::router::density_function::beardifier::BeardifierJunction {
                                    x: j_x,
                                    ground_y: j.source_ground_y,
                                    z: j_z,
                                }
                            );
                            let _junction_box = BlockBox::from_pos(BlockPos::new(j_x, j.source_ground_y, j_z));
                     any_piece_bounding_box = any_piece_bounding_box.map_or(Some(bounding_box), |mut b| {
                            b.encompass(&bounding_box);
                             Some(b)
                        });
                        }
                    }
                } else {
                        any_piece_bounding_box = any_piece_bounding_box.map_or(Some(bounding_box), |mut b| {
                            b.encompass(&bounding_box);
                             Some(b)
                         });

                    beardifier_structures.push(
                        crate::generation::noise::router::density_function::beardifier::BeardifierStructure {
                            bounding_box,
                            terrain_adaptation,
                            ground_level_delta,
                        }
                    );
                }
            }
        }

        let affected_box = any_piece_bounding_box.map(|b| b.expand(24, 24, 24));

        // Passed the newly mapped beardifier structures & junctions arrays independently!
        let mut noise_sampler = ChunkNoiseGenerator::new(
            &self.base_router.noise,
            &self.random_config,
            horizontal_cell_count as usize,
            start_x,
            start_z,
            generation_shape,
            sampler,
            settings.aquifers_enabled,
            settings.ore_veins_enabled,
            beardifier_structures,
            beardifier_junctions,
            affected_box,
        );

        let horizontal_biome_end = biome_coords::from_block(
            horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
        );
        let surface_config = SurfaceHeightSamplerBuilderOptions::new(
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
            generation_shape.min_y as i32,
            generation_shape.max_y() as i32,
            generation_shape.vertical_cell_block_count() as usize,
        );
        let mut surface_height_estimate_sampler = SurfaceHeightEstimateSampler::generate(
            &self.base_router.surface_estimator,
            &surface_config,
        );
        self.populate_noise(
            chunk,
            &mut noise_sampler,
            &self.random_config.ore_random_deriver,
            &mut surface_height_estimate_sampler,
        );

        chunk.stage = StagedChunkEnum::Noise;
    }

    pub fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Noise);
        let start_x = chunk.start_block_x();
        let start_z = chunk.start_block_z();
        let generation_shape = &self.settings().shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();

        let horizontal_biome_end = biome_coords::from_block(
            horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
        );
        let surface_config = SurfaceHeightSamplerBuilderOptions::new(
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
            generation_shape.min_y as i32,
            generation_shape.max_y() as i32,
            generation_shape.vertical_cell_block_count() as usize,
        );
        let mut surface_height_estimate_sampler = SurfaceHeightEstimateSampler::generate(
            &self.base_router.surface_estimator,
            &surface_config,
        );

        self.build_surface(chunk, &mut surface_height_estimate_sampler);
        chunk.stage = StagedChunkEnum::Surface;
    }

    pub fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Surface);
        crate::generation::carver::carve(chunk, self);

        chunk.stage = StagedChunkEnum::Carvers;
    }

    pub fn populate_biomes(
        &self,
        chunk: &mut ProtoChunk,
        multi_noise_sampler: &mut MultiNoiseSampler,
    ) {
        let blender = Blender::empty();
        let biome_supplier = blender.get_biome_supplier(self.biome_source.as_ref());
        let min_y = chunk.bottom_y();
        let bottom_section = section_coords::block_to_section(min_y as i32);
        let top_section =
            section_coords::block_to_section(min_y as i32 + chunk.height() as i32 - 1);

        let start_block_x = chunk.start_block_x();
        let start_block_z = chunk.start_block_z();

        let start_biome_x = biome_coords::from_block(start_block_x);
        let start_biome_z = biome_coords::from_block(start_block_z);

        for i in bottom_section..=top_section {
            let start_block_y = section_coords::section_to_block(i);
            let start_biome_y = biome_coords::from_block(start_block_y);

            let biomes_per_section = biome_coords::from_block(CHUNK_DIM as i32);
            for x in 0..biomes_per_section {
                for y in 0..biomes_per_section {
                    for z in 0..biomes_per_section {
                        let biome = biome_supplier.biome(
                            start_biome_x + x,
                            start_biome_y + y,
                            start_biome_z + z,
                            multi_noise_sampler,
                        );
                        let index = chunk.local_biome_pos_to_biome_index(
                            x,
                            start_biome_y + y - biome_coords::from_block(min_y as i32),
                            z,
                        );

                        chunk.flat_biome_map[index] = biome.id;
                    }
                }
            }
        }
    }

    #[expect(clippy::similar_names)]
    pub fn populate_noise(
        &self,
        chunk: &mut ProtoChunk,
        noise_sampler: &mut ChunkNoiseGenerator,
        ore_random_deriver: &XoroshiroSplitter,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let h_count = noise_sampler.horizontal_cell_block_count() as i32;
        let v_count = noise_sampler.vertical_cell_block_count() as i32;
        let horizontal_cells = CHUNK_DIM as i32 / h_count;

        let minimum_cell_y = noise_sampler.min_y() / v_count as i8;
        let cell_height = noise_sampler.height() / v_count as u16;

        let delta_y_step = 1.0 / v_count as f64;
        let delta_x_z_step = 1.0 / h_count as f64;

        noise_sampler.sample_start_density();
        for cell_x in 0..horizontal_cells {
            noise_sampler.sample_end_density(cell_x);
            let sample_start_x = (chunk.start_cell_x(h_count) + cell_x) * h_count;
            let block_x_base = chunk.start_block_x() + cell_x * h_count;

            for cell_z in 0..horizontal_cells {
                let sample_start_z = (chunk.start_cell_z(h_count) + cell_z) * h_count;
                let block_z_base = chunk.start_block_z() + cell_z * h_count;

                for cell_y in (0..cell_height).rev() {
                    noise_sampler.on_sampled_cell_corners(cell_x, cell_y as i32, cell_z);
                    let sample_start_y = (minimum_cell_y as i32 + cell_y as i32) * v_count;

                    for local_y in (0..v_count).rev() {
                        let block_y = sample_start_y + local_y;
                        noise_sampler.interpolate_y(local_y as f64 * delta_y_step);

                        for local_x in 0..h_count {
                            noise_sampler.interpolate_x(local_x as f64 * delta_x_z_step);
                            let block_x = block_x_base + local_x;

                            for local_z in 0..h_count {
                                noise_sampler.interpolate_z(local_z as f64 * delta_x_z_step);
                                let block_z = block_z_base + local_z;

                                let block_state = noise_sampler
                                    .sample_block_state(
                                        ore_random_deriver,
                                        sample_start_x,
                                        sample_start_y,
                                        sample_start_z,
                                        local_x,
                                        block_y - sample_start_y,
                                        local_z,
                                        surface_height_estimate_sampler,
                                    )
                                    .unwrap_or(self.default_block);
                                chunk.set_block_state(block_x, block_y, block_z, block_state);
                            }
                        }
                    }
                }
            }
            noise_sampler.swap_buffers();
        }
    }
    #[expect(clippy::too_many_lines)]
    pub fn build_surface(
        &self,
        chunk: &mut ProtoChunk,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let start_x = chunk.start_block_x();
        let start_z = chunk.start_block_z();
        let min_y = chunk.bottom_y();

        let settings = self.settings();
        let random_config = &self.random_config;
        let terrain_cache = &self.terrain_cache;

        let random = &random_config.base_random_deriver;
        let mut context = MaterialRuleContext::new(
            chunk.generation_bottom_y(),
            chunk.generation_height(),
            random,
            &terrain_cache.terrain_builder,
            &terrain_cache.surface_noise,
            &terrain_cache.secondary_noise,
            settings.sea_level,
        );
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = start_x + local_x;
                let z = start_z + local_z;

                let mut top_block = chunk.top_block_height_exclusive(local_x, local_z);

                let biome_y = if settings.legacy_random_source {
                    0
                } else {
                    top_block
                };

                let this_biome = chunk.get_terrain_gen_biome_id(x, biome_y, z);
                if this_biome == Biome::ERODED_BADLANDS {
                    terrain_cache
                        .terrain_builder
                        .place_badlands_pillar(chunk, x, z, top_block);

                    top_block = chunk.top_block_height_exclusive(local_x, local_z);
                }

                context.init_horizontal(x, z);

                let mut stone_depth_above = 0;
                let mut min = i32::MAX;
                let mut fluid_height = i32::MIN;
                for y in (min_y as i32..top_block).rev() {
                    let pos = Vector3::new(x, y, z);
                    let state = chunk.get_block_state(&pos).to_state();
                    if state.is_air() {
                        stone_depth_above = 0;
                        fluid_height = i32::MIN;
                        continue;
                    }
                    if state.is_liquid() {
                        if fluid_height == i32::MIN {
                            fluid_height = y + 1;
                        }
                        continue;
                    }
                    if min >= y {
                        let shift = min_y << 4;
                        min = shift as i32;

                        for search_y in ((min_y as i32 - 1)..y).rev() {
                            if search_y < min_y as i32 {
                                min = search_y + 1;
                                break;
                            }

                            let block_id = chunk
                                .get_block_state(&Vector3::new(local_x, search_y, local_z))
                                .to_block_id();

                            if !(block_id != Block::AIR
                                && block_id != WATER_BLOCK
                                && block_id != LAVA_BLOCK)
                            {
                                min = search_y + 1;
                                break;
                            }
                        }
                    }

                    stone_depth_above += 1;
                    let stone_depth_below = y - min + 1;
                    context.init_vertical(stone_depth_above, stone_depth_below, y, fluid_height);

                    if state.id == self.default_block.id {
                        context.biome = chunk.get_terrain_gen_biome(
                            context.block_pos_x,
                            context.block_pos_y,
                            context.block_pos_z,
                        );
                        let new_state = try_apply_material_rule(
                            &settings.surface_rule,
                            chunk,
                            &mut context,
                            surface_height_estimate_sampler,
                        );

                        if let Some(state) = new_state {
                            chunk.set_block_state(x, y, z, state);
                        }
                    }
                }
                if this_biome == Biome::FROZEN_OCEAN || this_biome == Biome::DEEP_FROZEN_OCEAN {
                    let surface_estimate =
                        estimate_surface_height(&mut context, surface_height_estimate_sampler);

                    terrain_cache.terrain_builder.place_iceberg(
                        chunk,
                        Biome::from_id(this_biome).unwrap_or(&Biome::PLAINS),
                        x,
                        z,
                        surface_estimate,
                        top_block,
                        settings.sea_level,
                        &random_config.base_random_deriver,
                    );
                }
            }
        }
    }

    pub fn generate_features_and_structure<T: GenerationCache>(
        cache: &mut T,
        block_registry: &dyn WorldPortalExt,
        random_config: &GlobalRandomConfig,
    ) {
        let (center_x, center_z, min_y, generation_min_y, generation_height, biomes_in_chunk) = {
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
                chunk.generation_bottom_y(),
                chunk.generation_height(),
                unique_biomes,
            )
        };

        let start_block_x = chunk_pos::start_block_x(center_x);
        let start_block_z = chunk_pos::start_block_z(center_z);
        let origin_pos = BlockPos::new(start_block_x, min_y, start_block_z);

        let population_seed =
            Xoroshiro::get_population_seed(random_config.seed, start_block_x, start_block_z);

        for step in 0..11 {
            ProtoChunk::generate_structure_step(
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
                        generation_min_y,
                        generation_height,
                        feature_enum,
                        &mut random,
                        origin_pos,
                    );
                }
            }
        }

        cache.get_center_chunk_mut().stage = StagedChunkEnum::Features;
    }

    pub fn set_structure_starts(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Biomes);
        let random_config = &self.random_config;
        let settings = self.settings();
        let global_cache = &self.global_structure_cache;
        let calculator = &self.structure_calculator;

        let seed = random_config.seed;

        let mut height_sampler =
            crate::generation::structure::height_sampler::NoiseHeightSampler::new(
                self,
                chunk.start_block_x(),
                chunk.start_block_z(),
            );

        for (i, set) in StructureSet::ALL.iter().enumerate() {
            let allowed_biomes = &self.structure_allowed_biomes[&i];

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

            if set.structures.len() == 1 {
                if let Some(entry) = set.structures.first() {
                    self.try_set_structure_start(
                        chunk,
                        global_cache,
                        settings.sea_level,
                        entry,
                        &mut height_sampler,
                    );
                }
                continue;
            }

            let mut candidates = set.structures.to_vec();
            let carver_seed = get_carver_seed(seed, chunk.x, chunk.z);
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
                    chunk,
                    global_cache,
                    settings.sea_level,
                    selected_entry,
                    &mut height_sampler,
                ) {
                    break;
                }

                let failed_entry = candidates.remove(selected_idx);
                total_weight -= failed_entry.weight;
            }
        }
        chunk.stage = StagedChunkEnum::StructureStart;
    }

    fn try_set_structure_start(
        &self,
        chunk: &mut ProtoChunk,
        global_cache: &GlobalStructureCache,
        sea_level: i32,
        entry: &WeightedEntry,
        height_sampler: &mut dyn crate::generation::structure::structures::HeightSampler,
    ) -> bool {
        if entry.structure == StructureKeys::Monument {
            let config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
            let mut sampler = MultiNoiseSampler::generate(&self.base_router.multi_noise, &config);
            let center_x = chunk_pos::get_center_x(chunk.x);
            let center_z = chunk_pos::get_center_z(chunk.z);
            let start_y = height_sampler.estimate_ocean_floor_height(center_x, center_z);
            if !crate::generation::structure::structures::ocean_monument::has_valid_biomes(
                self.biome_source.as_ref(),
                &mut sampler,
                chunk.x,
                chunk.z,
                sea_level,
                start_y,
            ) {
                return false;
            }
        }

        let chunk_x = chunk.x;
        let chunk_z = chunk.z;
        let position =
            global_cache.get_or_compute_structure_start(entry.structure, chunk_x, chunk_z, || {
                let structure = Structure::get(&entry.structure);
                try_generate_structure(
                    &entry.structure,
                    structure,
                    self.random_config.seed as i64,
                    chunk,
                    sea_level,
                    Some(height_sampler),
                )
            });

        if let Some(pos) = position {
            chunk
                .structure_starts
                .insert(entry.structure, StructureInstance::Start(pos));
            return true;
        }
        false
    }

    #[expect(clippy::too_many_lines)]
    pub fn set_structure_references(&self, chunk: &mut ProtoChunk) {
        debug_assert_eq!(chunk.stage, StagedChunkEnum::StructureStart);
        let random_config = &self.random_config;
        let settings = self.settings();
        let noise_router = &self.base_router;
        let global_cache = &self.global_structure_cache;
        let calculator = &self.structure_calculator;

        let start_x = chunk.start_block_x();
        let start_z = chunk.start_block_z();
        let end_x = start_x + 15;
        let end_z = start_z + 15;

        let seed = random_config.seed as i64;

        let blender = Blender::empty();
        let biome_supplier = blender.get_biome_supplier(self.biome_source.as_ref());
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(0, 0, 0);
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);

        let mut height_sampler =
            crate::generation::structure::height_sampler::NoiseHeightSampler::new(
                self, start_x, start_z,
            );

        let mut references = Vec::new();
        // Constant across every chunk in the dimension, so hoist it out of the loop
        // and out of the (cached) structure-start computation below.
        let chunk_min_y = chunk.bottom_y() as i32;

        for (set_index, set) in StructureSet::ALL.iter().enumerate() {
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
                                )
                            );
                        }
                    }
                }
                StructurePlacementType::ConcentricRings(rings) => {
                    let allowed_biomes = ProtoChunk::get_allowed_biomes(set);
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
                    &self.structure_allowed_biomes[&set_index],
                ) {
                    continue;
                }

                if (candidate_chunk_x - chunk.x).abs() <= 8
                    && (candidate_chunk_z - chunk.z).abs() <= 8
                {
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
            chunk
                .structure_starts
                .entry(key)
                .or_insert_with(|| StructureInstance::Reference(pos));
        }

        chunk.stage = StagedChunkEnum::StructureReferences;
    }
}
