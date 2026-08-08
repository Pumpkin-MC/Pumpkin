use super::chunk_state::{Chunk, StagedChunkEnum};
use crate::ProtoChunk;
use crate::chunk::ChunkHeightmapType;
use crate::generation::biome_coords;
use crate::generation::generator;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::{BiomeNeighborhood, GenerationCache};
use crate::world::{BlockAccessor, WorldPortalExt};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::biome::Biome;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use tracing::debug;

pub struct Cache {
    pub x: i32,
    pub z: i32,
    pub size: i32,
    pub chunks: Vec<Chunk>,
}

impl HeightLimitView for Cache {
    fn height(&self) -> u16 {
        let mid = ((self.size * self.size) >> 1) as usize;
        match &self.chunks[mid] {
            Chunk::Proto(chunk) => chunk.height(),
            Chunk::Level(_) => panic!(),
        }
    }

    fn bottom_y(&self) -> i8 {
        let mid = ((self.size * self.size) >> 1) as usize;
        match &self.chunks[mid] {
            Chunk::Proto(chunk) => chunk.bottom_y(),
            Chunk::Level(_) => panic!(),
        }
    }

    fn sea_level(&self) -> i32 {
        let mid = ((self.size * self.size) >> 1) as usize;
        match &self.chunks[mid] {
            Chunk::Proto(chunk) => chunk.sea_level(),
            Chunk::Level(_) => panic!(),
        }
    }
}

impl BlockAccessor for Cache {
    fn get_block(&self, position: &BlockPos) -> &'static Block {
        GenerationCache::get_block_state(self, &position.0).to_block()
    }

    fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
        GenerationCache::get_block_state(self, &position.0).to_state()
    }

    fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
        GenerationCache::get_block_state(self, &position.0)
    }

    fn get_block_and_state(&self, position: &BlockPos) -> (&'static Block, &'static BlockState) {
        let id = GenerationCache::get_block_state(self, &position.0);
        BlockState::from_id_with_block(id)
    }

    fn get_fluid(&self, position: &BlockPos) -> Fluid {
        GenerationCache::get_fluid_and_fluid_state(self, &position.0).0
    }
}

impl GenerationCache for Cache {
    fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        if dx < 0 || dx >= self.size || dz < 0 || dz >= self.size {
            return None;
        }

        match &mut self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Proto(chunk) => Some(chunk),
            Chunk::Level(_) => None,
        }
    }

    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        if dx < 0 || dx >= self.size || dz < 0 || dz >= self.size {
            return None;
        }

        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Proto(chunk) => Some(chunk),
            Chunk::Level(_) => None,
        }
    }

    fn try_get_proto_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        if dx < 0 || dx >= self.size || dz < 0 || dz >= self.size {
            return None;
        }

        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Proto(chunk) => Some(chunk),
            Chunk::Level(_) => None,
        }
    }

    fn get_center_chunk(&self) -> &ProtoChunk {
        let mid = ((self.size * self.size) >> 1) as usize;
        self.chunks[mid].get_proto_chunk()
    }

    fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk {
        let mid = ((self.size * self.size) >> 1) as usize;
        self.chunks[mid].get_proto_chunk_mut()
    }

    fn get_fluid_and_fluid_state(&self, pos: &Vector3<i32>) -> (Fluid, FluidState) {
        let id = GenerationCache::get_block_state(self, pos);

        let Some(fluid) = Fluid::from_state_id(id) else {
            let block = Block::from_state_id(id);
            if let Some(properties) = block.properties(id) {
                for (name, value) in properties.to_props() {
                    if name == "waterlogged" {
                        if value == "true" {
                            let fluid = Fluid::FLOWING_WATER;
                            let state = fluid.states[0].clone();
                            return (fluid, state);
                        }

                        break;
                    }
                }
            }

            let fluid = Fluid::EMPTY;
            let state = fluid.states[0].clone();

            return (fluid, state);
        };

        //let state = fluid.get_state(id);
        let state = fluid.states[0].clone();

        (fluid.clone(), state)
    }

    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        let dx = (pos.x >> 4) - self.x;
        let dz = (pos.z >> 4) - self.z;
        // debug_assert!(dx < self.size && dz < self.size);
        // debug_assert!(dx >= 0 && dz >= 0);
        if !(dx < self.size && dz < self.size && dx >= 0 && dz >= 0) {
            // breakpoint here
            debug!(
                "illegal get_block_state {pos:?} cache pos ({}, {}) size {}",
                self.x, self.z, self.size
            );
            return BlockStateId::AIR;
        }
        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => data
                .section
                .get_block_absolute_y((pos.x & 15) as usize, pos.y, (pos.z & 15) as usize)
                .unwrap_or(BlockStateId::AIR),

            Chunk::Proto(data) => data.get_block_state(pos),
        }
    }
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState) {
        let dx = (pos.x >> 4) - self.x;
        let dz = (pos.z >> 4) - self.z;
        // debug_assert!(dx < self.size && dz < self.size);
        // debug_assert!(dx >= 0 && dz >= 0);
        if !(dx < self.size && dz < self.size && dx >= 0 && dz >= 0) {
            // breakpoint here
            debug!(
                "illegal set_block_state {pos:?} cache pos ({}, {}) size {}",
                self.x, self.z, self.size
            );
            return;
        }
        match &mut self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => {
                data.set_block_absolute_y(
                    (pos.x & 15) as usize,
                    pos.y,
                    (pos.z & 15) as usize,
                    block_state.id,
                );
            }
            Chunk::Proto(data) => {
                data.set_block_state(pos.x, pos.y, pos.z, block_state);
            }
        }
    }

    fn add_block_entity(&mut self, pos: &Vector3<i32>, nbt: NbtCompound) {
        let dx = (pos.x >> 4) - self.x;
        let dz = (pos.z >> 4) - self.z;
        if !(dx < self.size && dz < self.size && dx >= 0 && dz >= 0) {
            debug!(
                "illegal add_block_entity {pos:?} cache pos ({}, {}) size {}",
                self.x, self.z, self.size
            );
            return;
        }

        match &mut self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(_) => {
                debug!("add_block_entity on non-proto chunk at {pos:?}");
            }
            Chunk::Proto(data) => {
                data.add_block_entity(nbt);
            }
        }
    }

    fn get_top_y(&self, heightmap: &HeightMap, x: i32, z: i32) -> i32 {
        match heightmap {
            HeightMap::WorldSurfaceWg | HeightMap::WorldSurface => {
                self.top_block_height_exclusive(x, z)
            }
            HeightMap::OceanFloorWg | HeightMap::OceanFloor => {
                self.ocean_floor_height_exclusive(x, z)
            }
            HeightMap::MotionBlocking => self.top_motion_blocking_block_height_exclusive(x, z),
            HeightMap::MotionBlockingNoLeaves => {
                self.top_motion_blocking_block_no_leaves_height_exclusive(x, z)
            }
        }
    }

    fn top_motion_blocking_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let heightmap = data.heightmap.lock().unwrap();
                let min_y = data.section.min_y;

                heightmap.get(ChunkHeightmapType::MotionBlocking, x, z, min_y)
            }
            Chunk::Proto(data) => data.top_motion_blocking_block_height_exclusive(x, z),
        }
    }

    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let heightmap = data.heightmap.lock().unwrap();
                let min_y = data.section.min_y;
                heightmap.get(ChunkHeightmapType::MotionBlockingNoLeaves, x, z, min_y)
            }
            Chunk::Proto(data) => data.top_motion_blocking_block_no_leaves_height_exclusive(x, z),
        }
    }

    fn top_block_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let heightmap = data.heightmap.lock().unwrap();
                let min_y = data.section.min_y;
                heightmap.get(ChunkHeightmapType::WorldSurface, x, z, min_y) // can we return this?
            }
            Chunk::Proto(data) => data.top_block_height_exclusive(x, z),
        }
    }

    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        if dx < 0 || dy < 0 || dx >= self.size || dy >= self.size {
            return 0;
        }
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(_data) => {
                0 // todo missing
            }
            Chunk::Proto(data) => data.ocean_floor_height_exclusive(x, z),
        }
    }

    fn get_biome_for_terrain_gen(&self, x: i32, y: i32, z: i32) -> &'static Biome {
        // Apply vanilla's `BiomeManager.getBiome` offset once, then resolve the resulting biome
        // cell in the chunk that owns it. Routing the *pre*-offset position to a chunk and
        // letting that chunk offset and mask with `& 3` wraps edge lookups back into the same
        // chunk, which is what put chunk-aligned seams into terrain-gen biomes.
        let mid = ((self.size * self.size) >> 1) as usize;
        let Chunk::Proto(center) = &self.chunks[mid] else {
            unreachable!(
                "terrain-gen biome lookup on a cache whose centre is already a level chunk"
            )
        };
        let cell = center.terrain_gen_biome_cell(x, y, z);
        let id = self
            .biome_id_at_cell(cell.x, cell.y, cell.z)
            .unwrap_or_else(|| center.get_biome_id(cell.x, cell.y, cell.z));
        Biome::from_id(id).unwrap()
    }

    fn get_blending_data(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&crate::generation::blender::blending_data::BlendingData> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        if dx < 0 || dx >= self.size || dz < 0 || dz >= self.size {
            return None;
        }

        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Proto(chunk) => chunk.blending_data.as_ref(),
            Chunk::Level(data) => data.blending_data.as_ref(),
        }
    }

    fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        is_air(GenerationCache::get_block_state(self, local_pos))
    }
}

impl Cache {
    pub fn advance_all(
        &mut self,
        stage: StagedChunkEnum,
        generator: &generator::WorldGenerator,
        block_registry: &dyn WorldPortalExt,
        lighting_config: &LightingEngineConfig,
    ) {
        for index in 0..self.chunks.len() {
            self.advance_index(index, stage, generator, block_registry, lighting_config);
        }
    }

    pub fn advance_index(
        &mut self,
        index: usize,
        stage: StagedChunkEnum,
        generator: &generator::WorldGenerator,
        _block_registry: &dyn WorldPortalExt,
        _lighting_config: &LightingEngineConfig,
    ) {
        match &self.chunks[index] {
            Chunk::Level(_) => return,
            Chunk::Proto(chunk) if chunk.stage >= stage => return,
            Chunk::Proto(_) => {}
        }
        match stage {
            StagedChunkEnum::Empty => panic!("empty stage"),
            StagedChunkEnum::StructureStart => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[index]
                        .get_proto_chunk_mut()
                        .set_structure_starts(noise_gen);
                }
                generator::WorldGenerator::Flat(_) => {}
            },
            StagedChunkEnum::StructureReferences => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[index]
                        .get_proto_chunk_mut()
                        .set_structure_references(noise_gen);
                }
                generator::WorldGenerator::Flat(_) => {}
            },
            StagedChunkEnum::Biomes => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[index]
                        .get_proto_chunk_mut()
                        .step_to_biomes(noise_gen);
                }
                generator::WorldGenerator::Flat(flat_gen) => {
                    flat_gen.step_to_biomes(self.chunks[index].get_proto_chunk_mut());
                }
            },
            _ => {}
        }
    }

    #[must_use]
    pub fn new(x: i32, z: i32, size: i32) -> Self {
        Self {
            x,
            z,
            size,
            chunks: Vec::with_capacity((size * size) as usize),
        }
    }
    /// Collects the biome cells covering the centre chunk plus a one-cell border out of the
    /// surrounding chunks, so that the surface builder's terrain-gen biome lookups resolve
    /// against the chunk that actually owns the cell instead of wrapping within the centre
    /// chunk. See `BiomeNeighborhood`.
    fn build_biome_neighborhood(&self, mid: usize) -> Option<BiomeNeighborhood> {
        // Without the surrounding ring there is nothing to resolve edge lookups against and the
        // surface builder would silently fall back to the wrapped read, so make that loud.
        debug_assert!(
            self.size >= 3,
            "surface generation needs a one-chunk ring to resolve edge biome lookups"
        );
        let Chunk::Proto(center) = &self.chunks[mid] else {
            return None;
        };
        let (chunk_x, chunk_z) = (center.x, center.z);
        let (bottom_y, height) = (center.bottom_y(), center.height());

        Some(BiomeNeighborhood::build(
            chunk_x,
            chunk_z,
            bottom_y,
            height,
            |biome_x, biome_y, biome_z| self.biome_id_at_cell(biome_x, biome_y, biome_z),
        ))
    }

    /// Reads a single biome cell from the chunk that owns it, the way vanilla's
    /// `LevelReader.getNoiseBiome` routes through `QuartPos.toSection`. `None` when that chunk is
    /// not in the cache.
    fn biome_id_at_cell(&self, biome_x: i32, biome_y: i32, biome_z: i32) -> Option<u8> {
        let dx = biome_coords::to_chunk(biome_x) - self.x;
        let dz = biome_coords::to_chunk(biome_z) - self.z;
        if dx < 0 || dz < 0 || dx >= self.size || dz >= self.size {
            return None;
        }
        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Proto(data) => Some(data.get_biome_id(biome_x, biome_y, biome_z)),
            Chunk::Level(data) => data.section.get_rough_biome_absolute_y(
                (biome_coords::to_block(biome_x) & 15) as usize,
                biome_coords::to_block(biome_y),
                (biome_coords::to_block(biome_z) & 15) as usize,
            ),
        }
    }

    pub fn advance(
        &mut self,
        stage: StagedChunkEnum,
        generator: &generator::WorldGenerator,
        block_registry: &dyn WorldPortalExt,
        lighting_config: &LightingEngineConfig,
    ) {
        let mid = ((self.size * self.size) >> 1) as usize;
        match &self.chunks[mid] {
            Chunk::Level(_) => return,
            Chunk::Proto(chunk) if chunk.stage >= stage => return,
            Chunk::Proto(_) => {}
        }
        match stage {
            StagedChunkEnum::Empty => panic!("empty stage"),
            StagedChunkEnum::StructureStart => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .set_structure_starts(noise_gen);
                }
                generator::WorldGenerator::Flat(_) => {}
            },
            StagedChunkEnum::StructureReferences => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .set_structure_references(noise_gen);
                }
                generator::WorldGenerator::Flat(_) => {}
            },
            StagedChunkEnum::Biomes => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .step_to_biomes(noise_gen);
                }
                generator::WorldGenerator::Flat(flat_gen) => {
                    flat_gen.step_to_biomes(self.chunks[mid].get_proto_chunk_mut());
                }
            },
            StagedChunkEnum::Noise => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .step_to_noise(noise_gen);
                }
                generator::WorldGenerator::Flat(flat_gen) => {
                    flat_gen.step_to_noise(self.chunks[mid].get_proto_chunk_mut());
                }
            },
            StagedChunkEnum::Surface => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    let neighborhood = self.build_biome_neighborhood(mid);
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .step_to_surface(noise_gen, neighborhood.as_ref());
                }
                generator::WorldGenerator::Flat(flat_gen) => {
                    flat_gen.step_to_surface(self.chunks[mid].get_proto_chunk_mut());
                }
            },
            StagedChunkEnum::Carvers => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    self.chunks[mid]
                        .get_proto_chunk_mut()
                        .step_to_carvers(noise_gen);
                }
                generator::WorldGenerator::Flat(flat_gen) => {
                    flat_gen.step_to_carvers(self.chunks[mid].get_proto_chunk_mut());
                }
            },
            StagedChunkEnum::Features => match generator {
                generator::WorldGenerator::Noise(noise_gen) => {
                    ProtoChunk::generate_features_and_structure(
                        self,
                        block_registry,
                        &noise_gen.random_config,
                    );
                }
                generator::WorldGenerator::Flat(_) => {
                    self.chunks[mid].get_proto_chunk_mut().stage = StagedChunkEnum::Features;
                }
            },
            StagedChunkEnum::Lighting => {
                let mut engine = crate::lighting::LightEngine::new();
                engine.initialize_light(self, lighting_config, generator.dimension().has_skylight);
                // Only set stage to Lighting if it wasn't already at Lighting or higher
                // (initialize_light may short-circuit for already-lit chunks)
                let chunk = self.chunks[mid].get_proto_chunk_mut();
                if chunk.stage < StagedChunkEnum::Lighting {
                    chunk.stage = StagedChunkEnum::Lighting;
                }
                // Engine's internal state is cleared by initialize_light() and will be dropped here
                drop(engine);
            }
            StagedChunkEnum::Spawn => {
                ProtoChunk::spawn_mobs(self, block_registry);
            }
            StagedChunkEnum::Full => {
                let chunk = self.chunks[mid].get_proto_chunk_mut();
                debug_assert_eq!(chunk.stage, StagedChunkEnum::Spawn);
                chunk.stage = StagedChunkEnum::Full;
                self.chunks[mid].upgrade_to_level_chunk(generator.dimension(), lighting_config);
            }
            StagedChunkEnum::None => {}
        }
    }
}
