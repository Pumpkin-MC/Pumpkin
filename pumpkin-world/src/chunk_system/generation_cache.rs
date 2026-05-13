use super::chunk_state::{Chunk, StagedChunkEnum};
use crate::block::RawBlockState;
use crate::chunk::ChunkHeightmapType;
use crate::generation::generator;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::world::{BlockAccessor, WorldPortalExt};
use crate::{BlockStateId, ProtoChunk};
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::biome::Biome;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::future::Future;
use std::pin::Pin;
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
            _ => panic!(),
        }
    }

    fn bottom_y(&self) -> i8 {
        let mid = ((self.size * self.size) >> 1) as usize;
        match &self.chunks[mid] {
            Chunk::Proto(chunk) => chunk.bottom_y(),
            _ => panic!(),
        }
    }
}

impl BlockAccessor for Cache {
    fn get_block<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static Block> + Send + 'a>> {
        Box::pin(async move { GenerationCache::get_block_state(self, &position.0).to_block() })
    }

    fn get_block_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = &'static BlockState> + Send + 'a>> {
        Box::pin(async move { GenerationCache::get_block_state(self, &position.0).to_state() })
    }

    fn get_block_state_id<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = BlockStateId> + Send + 'a>> {
        Box::pin(async move { GenerationCache::get_block_state(self, &position.0).0 })
    }

    fn get_block_and_state<'a>(
        &'a self,
        position: &'a BlockPos,
    ) -> Pin<Box<dyn Future<Output = (&'static Block, &'static BlockState)> + Send + 'a>> {
        Box::pin(async move {
            let id = GenerationCache::get_block_state(self, &position.0);
            (id.to_block(), id.to_state())
        })
    }
}

impl GenerationCache for Cache {
    fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ProtoChunk> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        (dx >= 0 && dx < self.size && dz >= 0 && dz < self.size)
            .then(|| self.chunks[(dx * self.size + dz) as usize].get_proto_chunk_mut())
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
        let id = GenerationCache::get_block_state(self, pos).0;

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

    fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState {
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
            return RawBlockState::AIR;
        }
        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => RawBlockState(
                data.section
                    .get_block_absolute_y((pos.x & 15) as usize, pos.y, (pos.z & 15) as usize)
                    .unwrap_or(0),
            ),
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
            HeightMap::WorldSurfaceWg => self.top_block_height_exclusive(x, z),
            HeightMap::WorldSurface => self.top_block_height_exclusive(x, z),
            HeightMap::OceanFloorWg => self.ocean_floor_height_exclusive(x, z),
            HeightMap::OceanFloor => self.ocean_floor_height_exclusive(x, z),
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
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        let (dx, dy) = if dx < 0 || dy < 0 || dx >= self.size || dy >= self.size {
            // Position is outside the cache — fall back to the centre chunk's biome
            let mid = self.size / 2;
            (mid, mid)
        } else {
            (dx, dy)
        };
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                // Could this happen?
                Biome::from_id(
                    data.section
                        .get_rough_biome_absolute_y((x & 15) as usize, y, (z & 15) as usize)
                        .unwrap_or(0),
                )
                .unwrap()
            }
            Chunk::Proto(data) => data.get_terrain_gen_biome(x, y, z),
        }
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
        is_air(GenerationCache::get_block_state(self, local_pos).0)
    }
}

impl Cache {
    #[must_use]
    pub fn new(x: i32, z: i32, size: i32) -> Self {
        Self {
            x,
            z,
            size,
            chunks: Vec::with_capacity((size * size) as usize),
        }
    }
    pub fn advance(
        &mut self,
        stage: StagedChunkEnum,
        generator: &generator::VanillaGenerator,
        block_registry: &dyn WorldPortalExt,
        lighting_config: &LightingEngineConfig,
    ) {
        let mid = ((self.size * self.size) >> 1) as usize;
        match stage {
            StagedChunkEnum::Empty => panic!("empty stage"),
            StagedChunkEnum::StructureStart => {
                self.chunks[mid]
                    .get_proto_chunk_mut()
                    .set_structure_starts(generator);
            }
            StagedChunkEnum::StructureReferences => {
                self.chunks[mid]
                    .get_proto_chunk_mut()
                    .set_structure_references(generator);
            }
            StagedChunkEnum::Biomes => self.chunks[mid]
                .get_proto_chunk_mut()
                .step_to_biomes(generator),
            StagedChunkEnum::Noise => self.chunks[mid]
                .get_proto_chunk_mut()
                .step_to_noise(generator),
            StagedChunkEnum::Surface => self.chunks[mid]
                .get_proto_chunk_mut()
                .step_to_surface(generator),
            StagedChunkEnum::Carvers => self.chunks[mid]
                .get_proto_chunk_mut()
                .step_to_carvers(generator),
            StagedChunkEnum::Features => {
                ProtoChunk::generate_features_and_structure(
                    self,
                    block_registry,
                    &generator.random_config,
                );
            }
            StagedChunkEnum::Lighting => {
                let mut engine = crate::lighting::LightEngine::new();
                engine.initialize_light(self, lighting_config);
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
                self.chunks[mid].upgrade_to_level_chunk(&generator.dimension, lighting_config);
            }
            StagedChunkEnum::None => {}
        }
    }
}
