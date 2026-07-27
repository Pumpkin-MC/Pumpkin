use pumpkin_data::Block;
use pumpkin_data::chunk::Biome;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::{block_box::BlockBox, position::BlockPos, vector3::Vector3};
use pumpkin_util::random::xoroshiro128::XoroshiroSplitter;

use crate::biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::chunk_system::StagedChunkEnum;
use crate::generation::noise::aquifer_sampler::FluidLevel;
use crate::generation::noise::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions;
use crate::generation::noise::{CHUNK_DIM, ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK};
use crate::generation::section_coords::section_to_block;
use crate::generation::structure::structures::StructureInstance;
use crate::generation::surface::rule::try_apply_material_rule;
use crate::generation::{
    biome, biome_coords,
    blender::{Blender, BlenderImpl},
    noise::router::{
        multi_noise_sampler::MultiNoiseSampler,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
    positions::chunk_pos,
    positions::chunk_pos::{start_block_x, start_block_z},
    section_coords,
    surface::{MaterialRuleContext, estimate_surface_height},
};
use crate::world::WorldPortalExt;

use super::{
    AIR_BLOCK, ActiveSupplier, GenerationCache, ProtoChunk, StandardChunkFluidLevelSampler,
};

impl ProtoChunk {
    pub fn step_to_biomes(&mut self, generator: &crate::generation::generator::VanillaGenerator) {
        debug_assert_eq!(self.stage, StagedChunkEnum::Empty);
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);
        let horizontal_biome_end = biome_coords::from_block(16);
        let multi_noise_config =
            crate::generation::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end as usize,
            );
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&generator.base_router.multi_noise, &multi_noise_config);
        self.populate_biomes(generator, &mut multi_noise_sampler);
        self.stage = StagedChunkEnum::Biomes;
    }

    #[expect(clippy::too_many_lines)]
    pub fn step_to_noise(&mut self, generator: &crate::generation::generator::VanillaGenerator) {
        debug_assert_eq!(self.stage, StagedChunkEnum::StructureReferences);
        let settings = generator.settings;
        let generation_shape = &settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);

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

        let chunk_start_x = self.start_block_x();
        let chunk_start_z = self.start_block_z();

        for (key, instance) in &self.structure_starts {
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

            let collector = collector.lock().unwrap();
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
                            // Vanilla Beardifier.java:74-75 grows the affected box with the
                            // junction's own single-block box, not the piece's bounding box.
                            let junction_box = BlockBox::from_pos(BlockPos::new(j_x, j.source_ground_y, j_z));
                            any_piece_bounding_box = any_piece_bounding_box.map_or(Some(junction_box), |mut b| {
                                b.encompass(&junction_box);
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
            &generator.base_router.noise,
            &generator.random_config,
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
            &generator.base_router.surface_estimator,
            &surface_config,
        );
        self.populate_noise(
            generator,
            &mut noise_sampler,
            &generator.random_config.ore_random_deriver,
            &mut surface_height_estimate_sampler,
        );

        self.stage = StagedChunkEnum::Noise;
    }

    pub fn step_to_surface(&mut self, generator: &crate::generation::generator::VanillaGenerator) {
        debug_assert_eq!(self.stage, StagedChunkEnum::Noise);
        let start_x = start_block_x(self.x);
        let start_z = start_block_z(self.z);
        let generation_shape = &generator.settings.shape;
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
            &generator.base_router.surface_estimator,
            &surface_config,
        );

        self.build_surface(generator, &mut surface_height_estimate_sampler);
        self.stage = StagedChunkEnum::Surface;
    }

    pub fn step_to_carvers(&mut self, generator: &crate::generation::generator::VanillaGenerator) {
        debug_assert_eq!(self.stage, StagedChunkEnum::Surface);
        crate::generation::carver::carve(self, generator);

        self.stage = StagedChunkEnum::Carvers;
    }

    pub fn populate_biomes(
        &mut self,
        generator: &crate::generation::generator::VanillaGenerator,
        multi_noise_sampler: &mut MultiNoiseSampler,
    ) {
        let dimension = &generator.dimension;
        let active_supplier = if dimension == &Dimension::THE_END {
            ActiveSupplier::End(TheEndBiomeSupplier)
        } else if dimension == &Dimension::THE_NETHER {
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
        let min_y = self.bottom_y();
        let bottom_section = section_coords::block_to_section(min_y as i32);
        let top_section = section_coords::block_to_section(min_y as i32 + self.height() as i32 - 1);

        let start_block_x = start_block_x(self.x);
        let start_block_z = start_block_z(self.z);

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
                        let index = self.local_biome_pos_to_biome_index(
                            x,
                            start_biome_y + y - biome_coords::from_block(min_y as i32),
                            z,
                        );

                        self.flat_biome_map[index] = biome.id;
                    }
                }
            }
        }
    }

    #[expect(clippy::similar_names)]
    pub fn populate_noise(
        &mut self,
        generator: &crate::generation::generator::VanillaGenerator,
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
            let sample_start_x = (self.start_cell_x(h_count) + cell_x) * h_count;
            let block_x_base = self.start_block_x() + cell_x * h_count;

            for cell_z in 0..horizontal_cells {
                let sample_start_z = (self.start_cell_z(h_count) + cell_z) * h_count;
                let block_z_base = self.start_block_z() + cell_z * h_count;

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
                                    .unwrap_or(generator.default_block);
                                self.set_block_state(block_x, block_y, block_z, block_state);
                            }
                        }
                    }
                }
            }
            noise_sampler.swap_buffers();
        }
    }

    pub fn spawn_mobs<T: GenerationCache>(cache: &mut T, block_registry: &dyn WorldPortalExt) {
        let chunk = cache.get_center_chunk();
        if chunk.stage >= StagedChunkEnum::Spawn {
            return;
        }
        debug_assert_eq!(chunk.stage, StagedChunkEnum::Lighting);

        let biome = chunk.get_terrain_gen_biome(
            section_to_block(chunk.x),
            chunk.bottom_y() as i32 + chunk.height() as i32 - 1,
            section_to_block(chunk.z),
        );
        let x = chunk.x;
        let z = chunk.z;

        block_registry.spawn_mobs_for_chunk_generation(cache, biome, x, z);

        cache.get_center_chunk_mut().stage = StagedChunkEnum::Spawn;
    }

    #[must_use]
    pub fn get_terrain_gen_biome_id(&self, x: i32, y: i32, z: i32) -> u8 {
        let seed_biome_pos = biome::get_biome_blend(
            self.bottom_y(),
            self.height(),
            self.biome_mixer_seed,
            x,
            y,
            z,
        );

        // This is deliberately only the persisted-palette fallback. The fuzzy
        // lookup can select a neighbor quart; without a region palette, clamp
        // that lookup to prevent `get_biome_id`'s local `& 3` from wrapping to
        // the opposite edge. Surface and carver material rules instead resolve
        // their fuzzy quart through `VanillaGenerator::terrain_gen_biome_at_block`.
        // Vanilla references: ChunkAccess.java:427-435 and
        // SurfaceSystem.java:110,119,156-157,179-182.
        let min_quart_x = biome_coords::from_block(start_block_x(self.x));
        let min_quart_z = biome_coords::from_block(start_block_z(self.z));
        let max_quart_offset = biome_coords::from_block(15);
        let quart_x = seed_biome_pos
            .x
            .clamp(min_quart_x, min_quart_x + max_quart_offset);
        let quart_z = seed_biome_pos
            .z
            .clamp(min_quart_z, min_quart_z + max_quart_offset);

        self.get_biome_id(quart_x, seed_biome_pos.y, quart_z)
    }

    #[must_use]
    pub fn get_terrain_gen_biome(&self, x: i32, y: i32, z: i32) -> &'static Biome {
        Biome::from_id(self.get_terrain_gen_biome_id(x, y, z)).unwrap()
    }

    #[expect(clippy::too_many_lines)]
    pub fn build_surface(
        &mut self,
        generator: &crate::generation::generator::VanillaGenerator,
        surface_height_estimate_sampler: &mut SurfaceHeightEstimateSampler,
    ) {
        let start_x = chunk_pos::start_block_x(self.x);
        let start_z = chunk_pos::start_block_z(self.z);
        let min_y = self.bottom_y();

        let settings = generator.settings;
        let random_config = &generator.random_config;
        let terrain_cache = &generator.terrain_cache;

        let random = &random_config.base_random_deriver;
        let mut context = MaterialRuleContext::new(
            min_y,
            self.height(),
            random,
            &terrain_cache.terrain_builder,
            &terrain_cache.surface_noise,
            &terrain_cache.secondary_noise,
            settings.sea_level,
        );
        // Vanilla SurfaceSystem passes `BiomeManager::getBiome` into its rule
        // context and uses the same fuzzy lookup for badlands and frozen oceans
        // (SurfaceSystem.java:110,119,156-157). Keep the sampler halo to the
        // maximum one-quart edge selection; farther calls use its uncached path.
        let mut biome_sampler = generator.terrain_gen_biome_sampler(self.x, self.z);
        for local_x in 0..16 {
            for local_z in 0..16 {
                let x = start_x + local_x;
                let z = start_z + local_z;

                let mut top_block = self.top_block_height_exclusive(local_x, local_z);

                let biome_y = if settings.legacy_random_source {
                    0
                } else {
                    top_block
                };

                let surface_biome =
                    generator.terrain_gen_biome_at_block(x, biome_y, z, &mut biome_sampler);
                if surface_biome.id == Biome::ERODED_BADLANDS {
                    terrain_cache
                        .terrain_builder
                        .place_badlands_pillar(self, x, z, top_block);

                    top_block = self.top_block_height_exclusive(local_x, local_z);
                }

                context.init_horizontal(x, z);

                let mut stone_depth_above = 0;
                let mut min = i32::MAX;
                let mut fluid_height = i32::MIN;
                for y in (min_y as i32..top_block).rev() {
                    let pos = Vector3::new(x, y, z);
                    let state = self.get_block_state(&pos).to_state();
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
                        // Vanilla SurfaceSystem.java:143 resets to DimensionType.WAY_BELOW_MIN_Y
                        // (DimensionType.java:48: MIN_Y << 4 = -32512, computed in int).
                        // `min_y << 4` on i8 would wrap (-64 << 4 overflows i8).
                        min = crate::generation::positions::MIN_HEIGHT_CELL;

                        for search_y in ((min_y as i32 - 1)..y).rev() {
                            if search_y < min_y as i32 {
                                min = search_y + 1;
                                break;
                            }

                            let block_id = self
                                .get_block_state(&Vector3::new(local_x, search_y, local_z))
                                .to_block_id();

                            if !(block_id != AIR_BLOCK
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
                        context.biome = generator.terrain_gen_biome_at_block(
                            context.block_pos_x,
                            context.block_pos_y,
                            context.block_pos_z,
                            &mut biome_sampler,
                        );
                        let new_state = try_apply_material_rule(
                            &settings.surface_rule,
                            self,
                            &mut context,
                            surface_height_estimate_sampler,
                        );

                        if let Some(state) = new_state {
                            self.set_block_state(x, y, z, state);
                        }
                    }
                }
                if surface_biome.id == Biome::FROZEN_OCEAN
                    || surface_biome.id == Biome::DEEP_FROZEN_OCEAN
                {
                    let surface_estimate =
                        estimate_surface_height(&mut context, surface_height_estimate_sampler);

                    terrain_cache.terrain_builder.place_iceberg(
                        self,
                        surface_biome,
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
}
