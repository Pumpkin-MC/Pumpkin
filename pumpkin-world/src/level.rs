use crate::generation::generator::VanillaGenerator;
use crate::generation::settings::gen_settings_from_dimension;
use crate::level::new_chunk_system::{ChunkLoading, GenerationSchedule, LevelChannel};
use crate::{
    BlockStateId,
    block::{RawBlockState, entities::BlockEntity},
    chunk::{
        ChunkData, ChunkEntityData, ChunkReadingError,
        format::{anvil::AnvilChunkFile, linear::LinearFile},
        io::{Dirtiable, FileIO, LoadedData, file_manager::ChunkFileManager},
    },
    dimension::Dimension,
    generation::{Seed, generator::WorldGenerator, get_world_gen, proto_chunk::StagedChunk},
    tick::{OrderedTick, ScheduledTick, TickPriority},
    world::BlockRegistryExt,
};
use crossbeam::channel::Sender;
use dashmap::{DashMap, Entry};
use log::trace;
use num_traits::Zero;
use pumpkin_config::{advanced_config, chunk::ChunkFormat};
use pumpkin_data::biome::Biome;
use pumpkin_data::{Block, block_properties::has_random_ticks, fluid::Fluid};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};
use tokio::time::sleep;
use tokio::{
    select,
    sync::{
        Notify, RwLock,
        mpsc::{self, UnboundedReceiver},
        oneshot,
    },
    task::JoinHandle,
};
use tokio_util::task::TaskTracker;

pub type SyncChunk = Arc<RwLock<ChunkData>>;
pub type SyncEntityChunk = Arc<RwLock<ChunkEntityData>>;

pub mod new_chunk_system {
    use crate::block::RawBlockState;
    use crate::chunk::ChunkHeightmapType;
    use crate::chunk::io::FileIO;
    use crate::chunk::io::LoadedData::Loaded;
    use crate::dimension::Dimension;
    use crate::generation::biome_coords;
    use crate::generation::height_limit::HeightLimitView;
    use crate::generation::positions::chunk_pos;
    use crate::generation::proto_chunk::{
        GenerationCache, StagedChunk, StandardChunkFluidLevelSampler, TerrainCache,
    };
    use crate::generation::settings::{GenerationSettings, gen_settings_from_dimension};
    use crate::level::new_chunk_system::StagedChunkEnum::{
        Biomes, Empty, Features, Noise, Surface,
    };
    use crate::level::{Level, LevelFolder, SyncChunk};
    use crate::world::{BlockAccessor, BlockRegistryExt};
    use crate::{GlobalRandomConfig, ProtoChunk, ProtoNoiseRouters};
    use async_trait::async_trait;
    use crossbeam::channel::{Receiver, Sender};
    use dashmap::DashMap;
    use itertools::Itertools;
    use log::debug;
    use num_traits::abs;
    use pumpkin_data::biome::Biome;
    use pumpkin_data::data_component_impl::get;
    use pumpkin_data::particle::Particle::SonicBoom;
    use pumpkin_data::{Block, BlockState};
    use pumpkin_util::HeightMap;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use std::char::MAX;
    use std::cmp::{PartialEq, max, min};
    use std::collections::hash_map::Entry;
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::mem::swap;
    use std::ptr;
    use std::sync::{Arc, Condvar, Mutex};
    use std::time::{Duration, Instant};
    use tokio::runtime::{Builder, Runtime};
    use tokio::sync::RwLock;
    use tokio::task::spawn_blocking;
    use tokio_util::task::TaskTracker;

    type ChunkPos = Vector2<i32>;
    type ChunkLevel = HashMap<ChunkPos, i8>;

    pub struct ChunkLoading {
        pub pos_level: ChunkLevel,
        pub ticket: HashMap<ChunkPos, Vec<i8>>, // TODO lifetime & id
    }

    impl ChunkLoading {
        pub fn new() -> Self {
            Self {
                pos_level: ChunkLevel::new(),
                ticket: HashMap::new(),
            }
        }
        pub const MAX_LEVEL: i8 = 46; // 46 就已经卸载了，不能是46
        fn run_decrease_update(pos_level: &mut ChunkLevel, pos: ChunkPos, level: i8) {
            debug_assert!(level < Self::MAX_LEVEL);
            debug_assert!(level < *pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL));
            let mut queue: VecDeque<(ChunkPos, i8)> = Default::default();
            queue.push_back((pos, level));
            while !queue.is_empty() {
                let (pos, level) = queue.pop_front().unwrap(); // can use unsafe
                debug_assert!(level < Self::MAX_LEVEL);
                match pos_level.entry(pos) {
                    Entry::Occupied(mut entry) => {
                        let old = entry.get_mut();
                        if *old <= level {
                            continue;
                        }
                        *old = level;
                    }
                    Entry::Vacant(empty) => {
                        empty.insert(level);
                    }
                }
                let spread_level = level + 1;
                if spread_level >= Self::MAX_LEVEL {
                    continue;
                }
                for dx in -1..2 {
                    for dy in -1..2 {
                        let new_pos = pos.add_raw(dx, dy);
                        if new_pos != pos
                            && spread_level < *pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL)
                        {
                            queue.push_back((new_pos, spread_level));
                        }
                    }
                }
            }
        }

        fn get_level_from_neighbour(&self, pos: ChunkPos) -> i8 {
            let mut ret = Self::MAX_LEVEL;
            for dx in -1..2 {
                for dy in -1..2 {
                    let new_pos = pos.add_raw(dx, dy);
                    ret = min(
                        *self.pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL) + 1,
                        ret,
                    );
                }
            }
            ret
        }

        fn run_increase_update(&mut self, pos: ChunkPos, level: i8) {
            // TODO super slow
            debug_assert!(level < Self::MAX_LEVEL);
            let range = Self::MAX_LEVEL - level - 1;
            for dx in -range..range {
                for dy in -range..range {
                    let new_pos = pos.add_raw(dx as i32, dy as i32);
                    self.pos_level.remove(&new_pos);
                }
            }
            for dst in (1..range).rev() {
                let min_x = pos.x - dst as i32;
                let max_x = pos.x - dst as i32;
                let min_y = pos.y - dst as i32;
                let max_y = pos.y - dst as i32;
                for y in min_y..max_y {
                    let new_pos = ChunkPos::new(max_x, y);
                    let new_level = self.get_level_from_neighbour(new_pos);
                    if new_level < Self::MAX_LEVEL {
                        self.pos_level.insert(new_pos, new_level);
                    }
                }
                for x in min_x..max_x {
                    let new_pos = ChunkPos::new(x, max_y);
                    let new_level = self.get_level_from_neighbour(new_pos);
                    if new_level < Self::MAX_LEVEL {
                        self.pos_level.insert(new_pos, new_level);
                    }
                }
                for y in (min_y + 1)..=max_y {
                    let new_pos = ChunkPos::new(min_x, y);
                    let new_level = self.get_level_from_neighbour(new_pos);
                    if new_level < Self::MAX_LEVEL {
                        self.pos_level.insert(new_pos, new_level);
                    }
                }
                for x in (min_x + 1)..=max_x {
                    let new_pos = ChunkPos::new(x, min_y);
                    let new_level = self.get_level_from_neighbour(new_pos);
                    if new_level < Self::MAX_LEVEL {
                        self.pos_level.insert(new_pos, new_level);
                    }
                }
            }
            let new_level = self.get_level_from_neighbour(pos);
            if new_level < Self::MAX_LEVEL {
                self.pos_level.insert(pos, new_level);
            }
            for (ticket_pos, levels) in &self.ticket {
                if abs(ticket_pos.x - pos.x) <= range as i32
                    && abs(ticket_pos.y - pos.y) <= range as i32
                {
                    Self::run_decrease_update(
                        &mut self.pos_level,
                        pos,
                        *levels.iter().max().unwrap(),
                    );
                }
            }
        }
        pub fn add_ticket(&mut self, pos: ChunkPos, level: i8) {
            debug_assert!(level < Self::MAX_LEVEL);
            let old_level = *self.pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL);
            use std::collections::hash_map::Entry;
            match self.ticket.entry(pos) {
                Entry::Occupied(mut vec) => {
                    vec.get_mut().push(level);
                }
                Entry::Vacant(empty) => {
                    empty.insert(vec![level]);
                }
            }
            if level < old_level {
                ChunkLoading::run_decrease_update(&mut self.pos_level, pos, level);
            }
        }
        pub fn remove_ticket(&mut self, pos: ChunkPos, level: i8) {
            debug_assert!(level < Self::MAX_LEVEL);
            let old_level = *self.pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL);
            let Some(vec) = self.ticket.get_mut(&pos) else {
                log::warn!("No ticket found at {pos:?}");
                return;
            };
            vec.remove(vec.iter().find_position(|x| **x == level).unwrap().0);
            let source = *vec.iter().max().unwrap_or(&Self::MAX_LEVEL);
            if vec.is_empty() {
                self.ticket.remove(&pos);
            }
            if level == old_level && source != level {
                self.run_increase_update(pos, old_level);
            }
        }
        pub fn get_cloned_level(&self) -> ChunkLevel {
            self.pos_level.clone()
        }
    }

    #[repr(u8)]
    #[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
    pub enum StagedChunkEnum {
        None,
        /// Initial empty chunk, ready for biome population
        Empty = 1, // EMPTY STRUCTURE_STARTS STRUCTURE_REFERENCES
        /// Chunk with biomes populated, ready for noise generation
        Biomes,
        /// Chunk with terrain noise generated, ready for surface building
        Noise,
        /// Chunk with surface built, ready for features and structures
        Surface, // SURFACE CARVERS
        /// Chunk with features and structures, ready for finalization
        Features, // FEATURES INITIALIZE_LIGHT LIGHT SPAWN
        /// Fully generated chunk
        Full,
    }

    impl From<u8> for StagedChunkEnum {
        fn from(v: u8) -> Self {
            match v {
                1 => Self::Empty,
                2 => Self::Biomes,
                3 => Self::Noise,
                4 => Self::Surface,
                5 => Self::Features,
                6 => Self::Full,
                _ => panic!(),
            }
        }
    }
    impl StagedChunkEnum {
        const MAX_STAGE_ID: u8 = Self::Full as u8;
        fn level_to_stage(level: i8) -> Self {
            if level <= 33 {
                Self::Full
            } else if level <= 35 {
                Self::Features
            } else if level <= 36 {
                Self::Surface
            } else if level <= 37 {
                Self::Biomes
            } else if level <= 45 {
                Self::Empty
            } else {
                Self::None
            }
        }
        fn get_radius(self) -> i32 {
            // 这个是不包括自己的范围
            match self {
                StagedChunkEnum::Empty => 0,
                StagedChunkEnum::Biomes => 8,
                StagedChunkEnum::Noise => 9,
                StagedChunkEnum::Surface => 9,
                StagedChunkEnum::Features => 10,
                StagedChunkEnum::Full => 11,
                _ => panic!(),
            }
        }
        fn get_dependencies(self) -> &'static [StagedChunkEnum] {
            match self {
                StagedChunkEnum::Empty => &[],
                StagedChunkEnum::Biomes => &[
                    Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                ],
                StagedChunkEnum::Noise => &[
                    Biomes, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                ],
                StagedChunkEnum::Surface => &[
                    Noise, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                ],
                StagedChunkEnum::Features => &[
                    Surface, Surface, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
                    Empty,
                ],
                StagedChunkEnum::Full => &[
                    Features, Features, Surface, Biomes, Empty, Empty, Empty, Empty, Empty, Empty,
                    Empty, Empty,
                ],
                _ => panic!(),
            }
        }
    }

    pub struct LevelChannel {
        pub value: Mutex<Option<ChunkLevel>>,
        pub notify: Condvar,
    }

    impl LevelChannel {
        pub fn new() -> Self {
            Self {
                value: Mutex::new(None),
                notify: Condvar::new(),
            }
        }

        pub fn set(&self, new_value: ChunkLevel) {
            *self.value.lock().unwrap() = Some(new_value);
            self.notify.notify_one();
        }
        pub fn get(&self) -> Option<ChunkLevel> {
            let mut lock = self.value.lock().unwrap();
            let mut ret = None;
            swap(&mut ret, &mut *lock);
            ret
        }
        pub fn wait_and_get(&self) -> Option<ChunkLevel> {
            // TODO if this return None, stop thread
            let mut lock = self.value.lock().unwrap();
            while lock.is_none() {
                lock = self.notify.wait(lock).unwrap();
            }
            let mut ret = None;
            swap(&mut ret, &mut *lock);
            ret
        }
        pub fn notify(&self) {
            // TODO ensure thread stop
            self.notify.notify_one();
        }
    }

    pub enum Chunk {
        Level(SyncChunk),
        Proto(StagedChunk),
    }

    impl Chunk {
        fn to_stage_id(chunk: &Option<Self>) -> u8 {
            match chunk {
                None => 0,
                Some(Chunk::Proto(data)) => data.stage_id(),
                Some(Chunk::Level(_)) => 6,
            }
        }
    }

    struct Cache {
        x: i32,
        y: i32,
        size: i32,
        pub chunks: Vec<Chunk>,
        rt: Runtime,
    }

    impl HeightLimitView for Cache {
        fn height(&self) -> u16 {
            let mid = (self.size * self.size >> 1) as usize;
            match &self.chunks[mid] {
                Chunk::Proto(chunk) => chunk.proto_chunk().height(),
                _ => panic!(),
            }
        }

        fn bottom_y(&self) -> i8 {
            let mid = (self.size * self.size >> 1) as usize;
            match &self.chunks[mid] {
                Chunk::Proto(chunk) => chunk.proto_chunk().bottom_y(),
                _ => panic!(),
            }
        }
    }

    #[async_trait]
    impl BlockAccessor for Cache {
        async fn get_block(&self, position: &BlockPos) -> &'static Block {
            GenerationCache::get_block_state(self, &position.0).to_block()
        }

        async fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            GenerationCache::get_block_state(self, &position.0).to_state()
        }

        async fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            let id = GenerationCache::get_block_state(self, &position.0);
            (id.to_block(), id.to_state())
        }
    }

    impl GenerationCache for Cache {
        fn get_center_chunk_mut(&mut self) -> &mut ProtoChunk {
            let mid = (self.size * self.size >> 1) as usize;
            match &mut self.chunks[mid] {
                Chunk::Proto(chunk) => chunk.proto_chunk_mut(),
                _ => panic!(),
            }
        }

        fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState {
            let dx = (pos.x >> 4) - self.x;
            let dz = (pos.z >> 4) - self.y;
            debug_assert!(dx < self.size && dz < self.size);
            match &self.chunks[(dx * self.size + dz) as usize] {
                Chunk::Level(data) => {
                    let chunk = self.rt.block_on(data.read());
                    RawBlockState(
                        chunk
                            .section
                            .get_block_absolute_y(
                                (pos.x & 15) as usize,
                                pos.y - chunk.section.min_y,
                                (pos.z & 15) as usize,
                            )
                            .unwrap_or(0),
                    )
                }
                Chunk::Proto(data) => data.proto_chunk().get_block_state(pos),
            }
        }
        fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState) {
            let dx = (pos.x >> 4) - self.x;
            let dz = (pos.z >> 4) - self.y;
            debug_assert!(dx < self.size && dz < self.size);
            match &mut self.chunks[(dx * self.size + dz) as usize] {
                Chunk::Level(data) => {
                    let mut chunk = self.rt.block_on(data.write());
                    let min_y = chunk.section.min_y;
                    chunk.section.set_block_absolute_y(
                        (pos.x & 15) as usize,
                        pos.y - min_y,
                        (pos.z & 15) as usize,
                        block_state.id,
                    );
                }
                Chunk::Proto(data) => {
                    data.proto_chunk_mut().set_block_state(pos, block_state);
                }
            }
        }

        fn get_top_y(&self, heightmap: &HeightMap, pos: &Vector2<i32>) -> i32 {
            match heightmap {
                HeightMap::WorldSurfaceWg => self.top_block_height_exclusive(pos),
                HeightMap::WorldSurface => self.top_block_height_exclusive(pos),
                HeightMap::OceanFloorWg => self.ocean_floor_height_exclusive(pos),
                HeightMap::OceanFloor => self.ocean_floor_height_exclusive(pos),
                HeightMap::MotionBlocking => self.top_motion_blocking_block_height_exclusive(pos),
                HeightMap::MotionBlockingNoLeaves => {
                    self.top_motion_blocking_block_no_leaves_height_exclusive(pos)
                }
            }
        }

        fn top_motion_blocking_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
            let dx = (pos.x >> 4) - self.x;
            let dy = (pos.y >> 4) - self.y;
            debug_assert!(dx < self.size && dy < self.size);
            match &self.chunks[(dx * self.size + dy) as usize] {
                Chunk::Level(data) => {
                    let chunk = self.rt.block_on(data.read());
                    chunk.heightmap.get_height(
                        ChunkHeightmapType::MotionBlocking,
                        pos.x,
                        pos.y,
                        chunk.section.min_y,
                    )
                }
                Chunk::Proto(data) => data
                    .proto_chunk()
                    .top_motion_blocking_block_height_exclusive(pos),
            }
        }

        fn top_motion_blocking_block_no_leaves_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
            let dx = (pos.x >> 4) - self.x;
            let dy = (pos.y >> 4) - self.y;
            debug_assert!(dx < self.size && dy < self.size);
            match &self.chunks[(dx * self.size + dy) as usize] {
                Chunk::Level(data) => {
                    let chunk = self.rt.block_on(data.read());
                    chunk.heightmap.get_height(
                        ChunkHeightmapType::MotionBlockingNoLeaves,
                        pos.x,
                        pos.y,
                        chunk.section.min_y,
                    )
                }
                Chunk::Proto(data) => data
                    .proto_chunk()
                    .top_motion_blocking_block_no_leaves_height_exclusive(pos),
            }
        }

        fn top_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
            let dx = (pos.x >> 4) - self.x;
            let dy = (pos.y >> 4) - self.y;
            debug_assert!(dx < self.size && dy < self.size);
            match &self.chunks[(dx * self.size + dy) as usize] {
                Chunk::Level(data) => {
                    let chunk = self.rt.block_on(data.read());
                    chunk.heightmap.get_height(
                        ChunkHeightmapType::WorldSurface,
                        pos.x,
                        pos.y,
                        chunk.section.min_y,
                    ) // can we return this?
                }
                Chunk::Proto(data) => data.proto_chunk().top_block_height_exclusive(pos),
            }
        }

        fn ocean_floor_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
            let dx = (pos.x >> 4) - self.x;
            let dy = (pos.y >> 4) - self.y;
            debug_assert!(dx < self.size && dy < self.size);
            match &self.chunks[(dx * self.size + dy) as usize] {
                Chunk::Level(data) => {
                    0 // todo missing
                }
                Chunk::Proto(data) => data.proto_chunk().ocean_floor_height_exclusive(pos),
            }
        }

        fn get_biome_for_terrain_gen(&self, global_block_pos: &Vector3<i32>) -> &'static Biome {
            let dx = (global_block_pos.x >> 4) - self.x;
            let dy = (global_block_pos.z >> 4) - self.y;
            debug_assert!(dx < self.size && dy < self.size);
            match &self.chunks[(dx * self.size + dy) as usize] {
                Chunk::Level(data) => {
                    // Could this happen?
                    &pumpkin_data::biome::Biome::PLAINS
                }
                Chunk::Proto(data) => data
                    .proto_chunk()
                    .get_biome_for_terrain_gen(global_block_pos),
            }
        }

        fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
            GenerationCache::get_block_state(self, local_pos)
                .to_state()
                .is_air()
        }
    }

    impl Cache {
        fn new(x: i32, y: i32, size: i32) -> Cache {
            let rt = Builder::new_current_thread().enable_all().build().unwrap();
            Cache {
                x,
                y,
                size,
                chunks: Vec::with_capacity((size * size) as usize),
                rt,
            }
        }
        pub fn advance(
            &mut self,
            stage: StagedChunkEnum,
            block_registry: &dyn BlockRegistryExt,
            settings: &GenerationSettings,
            random_config: &GlobalRandomConfig,
            terrain_cache: &TerrainCache,
            noise_router: &ProtoNoiseRouters,
            dimension: Dimension,
        ) {
            let mid = (self.size * self.size >> 1) as usize;
            match stage {
                Empty => panic!(),
                Biomes => match &mut self.chunks[mid] {
                    Chunk::Proto(chunk) => {
                        chunk.populate_biomes(noise_router, dimension);
                    }
                    _ => panic!(),
                },
                Noise => match &mut self.chunks[mid] {
                    Chunk::Proto(chunk) => {
                        chunk.populate_noise(settings, random_config, noise_router);
                    }
                    _ => panic!(),
                },
                Surface => match &mut self.chunks[mid] {
                    Chunk::Proto(chunk) => {
                        chunk.build_surface(settings, random_config, terrain_cache, noise_router);
                    }
                    _ => panic!(),
                },
                Features => {
                    ProtoChunk::generate_features_and_structure(
                        self,
                        block_registry,
                        random_config,
                    );
                    match &mut self.chunks[mid] {
                        Chunk::Proto(chunk) => unsafe {
                            let data = match ptr::read(chunk) {
                                StagedChunk::Surface(s) => s,
                                _ => panic!(),
                            };
                            ptr::write(chunk, StagedChunk::Features(data));
                        },
                        _ => panic!(),
                    }
                }
                StagedChunkEnum::Full => {
                    let ptr = &mut self.chunks[mid];
                    unsafe {
                        let data = match ptr::read(ptr) {
                            Chunk::Proto(chunk) => chunk.finalize(settings, dimension).unwrap(),
                            _ => panic!(),
                        };

                        ptr::write(ptr, Chunk::Level(Arc::new(RwLock::new(data))));
                    }
                }
                _ => panic!(),
            }
        }
    }

    enum RecvChunk {
        IO(Chunk),
        Generation(Cache),
    }

    pub struct GenerationSchedule {
        pub queue: Vec<(ChunkPos, i8, StagedChunkEnum)>,
        pub last_level: ChunkLevel,
        pub send_level: Arc<LevelChannel>,
        pub loaded_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,
        pub proto_chunks: HashMap<ChunkPos, StagedChunk>,
        pub unload_chunks: HashMap<ChunkPos, Chunk>,
        pub occupied: HashSet<ChunkPos>,
        pub task_mark: HashMap<ChunkPos, u8>,
        pub running_task_count: u16,
        pub recv_chunk: Receiver<(ChunkPos, RecvChunk)>,
        pub io: Sender<ChunkPos>,
        pub generate: Sender<(ChunkPos, Cache, StagedChunkEnum)>,
    }

    impl GenerationSchedule {
        pub(crate) fn new(
            tracker: &TaskTracker,
            oi_thread_count: u8,
            gen_thread_count: u8,
            level: Arc<Level>,
        ) -> Arc<LevelChannel> {
            let (send_chunk, recv_chunk) = crossbeam::channel::unbounded();
            let (send_io, recv_io) = crossbeam::channel::bounded(4);
            let (send_gen, recv_gen) = crossbeam::channel::bounded(20);
            let send_level = Arc::new(LevelChannel::new());
            for _ in 0..oi_thread_count {
                tracker.spawn(Self::io_work(
                    recv_io.clone(),
                    send_chunk.clone(),
                    level.clone(),
                ));
            }
            for _ in 0..gen_thread_count {
                let recv_gen = recv_gen.clone();
                let send_chunk = send_chunk.clone();
                let level = level.clone();
                std::thread::spawn(move || {
                    Self::generation_work(recv_gen, send_chunk, level);
                });
            }
            let send_level_clone = send_level.clone();
            std::thread::spawn(move || {
                Self {
                    queue: Vec::new(),
                    last_level: ChunkLevel::new(),
                    send_level: send_level_clone,
                    loaded_chunks: level.loaded_chunks.clone(),
                    proto_chunks: HashMap::new(),
                    unload_chunks: HashMap::new(),
                    occupied: HashSet::new(),
                    task_mark: HashMap::new(),
                    running_task_count: 0,
                    recv_chunk,
                    io: send_io,
                    generate: send_gen,
                }
                .work();
            });
            send_level
        }

        fn get_chunk(
            loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
            proto_chunks: &mut HashMap<ChunkPos, StagedChunk>,
            pos: ChunkPos,
        ) -> Option<Chunk> {
            if let Some(data) = loaded_chunks.get(&pos) {
                Some(Chunk::Level(data.clone()))
            } else if let Some(data) = proto_chunks.remove(&pos) {
                Some(Chunk::Proto(data))
            } else {
                None
            }
        }

        fn get_chunk_stage_id(
            loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
            proto_chunks: &mut HashMap<ChunkPos, StagedChunk>,
            pos: ChunkPos,
        ) -> u8 {
            if loaded_chunks.contains_key(&pos) {
                6
            } else if let Some(data) = proto_chunks.get(&pos) {
                data.stage_id()
            } else {
                0
            }
        }
        fn add_chunk(
            loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
            proto_chunks: &mut HashMap<ChunkPos, StagedChunk>,
            pos: ChunkPos,
            chunk: Chunk,
        ) {
            match chunk {
                Chunk::Level(data) => {
                    loaded_chunks.insert(pos, data);
                }
                Chunk::Proto(data) => {
                    proto_chunks.insert(pos, data);
                }
            }
        }

        fn resort_work(&mut self, new_level: ChunkLevel) {
            for (pos, _) in &self.last_level {
                if !new_level.contains_key(pos) {
                    if let Some(chunk) =
                        Self::get_chunk(&self.loaded_chunks, &mut self.proto_chunks, *pos)
                    {
                        self.unload_chunks.insert(*pos, chunk);
                    }
                }
            }
            for (pos, level) in &new_level {
                let old_level = *self
                    .last_level
                    .get(&pos)
                    .unwrap_or(&ChunkLoading::MAX_LEVEL);
                if old_level == ChunkLoading::MAX_LEVEL {
                    if let Some(chunk) = self.unload_chunks.remove(&pos) {
                        Self::add_chunk(&self.loaded_chunks, &mut self.proto_chunks, *pos, chunk);
                    }
                }
                if old_level != *level {
                    if !self.loaded_chunks.contains_key(pos) {
                        let next_stage = StagedChunkEnum::level_to_stage(old_level);
                        let new_highest_stage = StagedChunkEnum::level_to_stage(*level);
                        if next_stage == new_highest_stage {
                            continue;
                        }
                        if let Some(chunk) = self.proto_chunks.get(pos)
                            && chunk.stage_id() >= (new_highest_stage as u8)
                        {
                            continue;
                        }
                        match self.task_mark.entry(*pos) {
                            Entry::Occupied(mut entry) => {
                                let mark = entry.get_mut();
                                for i in (next_stage as u8 + 1)..=(new_highest_stage as u8) {
                                    if (*mark >> i & 1) == 0 {
                                        // no task before
                                        self.queue.push((*pos, i8::MAX, i.into()));
                                        *mark |= 1 << i;
                                    }
                                }
                            }
                            Entry::Vacant(entry) => {
                                let mut mark = 0;
                                for i in (next_stage as u8 + 1)..=(new_highest_stage as u8) {
                                    self.queue.push((*pos, i8::MAX, i.into()));
                                    mark |= 1 << i;
                                }
                                entry.insert(mark);
                            }
                        };
                    }
                }
            }
            for (pos, level, _) in self.queue.iter_mut() {
                *level = *new_level.get(pos).unwrap();
            }
            self.queue
                .sort_unstable_by(|(l_pos, l_level, l_stage), (r_pos, r_level, r_stage)| {
                    if l_level != r_level {
                        l_level.cmp(r_level)
                    } else {
                        l_stage.cmp(r_stage)
                    }
                });
            self.last_level = new_level;
        }
        fn try_unload() {
            // TODO
        }

        async fn io_work(
            recv: Receiver<ChunkPos>,
            send: Sender<(ChunkPos, RecvChunk)>,
            level: Arc<Level>,
        ) {
            log::info!("io thread start");
            use crate::biome::hash_seed;
            let biome_mixer_seed = hash_seed(level.world_gen.random_config.seed);
            let (t_send, mut t_recv) = tokio::sync::mpsc::channel(2);
            while let Ok(pos) = recv.recv() {
                debug!("io thread receive chunk pos {pos:?}");
                level
                    .chunk_saver
                    .fetch_chunks(&level.level_folder, &[pos], t_send.clone())
                    .await;
                if let Some(Loaded(chunk)) = t_recv.recv().await {
                    if send
                        .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                        .is_err()
                    {
                        log::info!("io thread stop");
                        break;
                    }
                } else {
                    if send
                        .send((
                            pos,
                            RecvChunk::IO(Chunk::Proto(StagedChunk::new(
                                pos,
                                gen_settings_from_dimension(&level.world_gen.dimension),
                                level.world_gen.default_block,
                                biome_mixer_seed,
                            ))),
                        ))
                        .is_err()
                    {
                        log::info!("io thread stop");
                        break;
                    }
                }
            }
        }

        fn generation_work(
            recv: Receiver<(ChunkPos, Cache, StagedChunkEnum)>,
            send: Sender<(ChunkPos, RecvChunk)>,
            level: Arc<Level>,
        ) {
            log::info!("generation thread start");
            while let Ok((pos, mut cache, stage)) = recv.recv() {
                debug!("generation thread receive chunk pos {pos:?} to stage {stage:?}");
                cache.advance(
                    stage,
                    level.block_registry.as_ref(),
                    gen_settings_from_dimension(&level.world_gen.dimension),
                    &level.world_gen.random_config,
                    &level.world_gen.terrain_cache,
                    &level.world_gen.base_router,
                    level.world_gen.dimension,
                );
                if send.send((pos, RecvChunk::Generation(cache))).is_err() {
                    log::info!("generation thread stop");
                    break;
                }
            }
        }

        fn drop_mark(&mut self, stage: StagedChunkEnum, pos: ChunkPos) {
            let mut mark = self.task_mark.get_mut(&pos).unwrap();
            debug_assert!((*mark >> (stage as u8) & 1) == 1);
            *mark -= 1 << (stage as u8);
            if *mark == 0 {
                self.task_mark.remove(&pos);
            }
        }

        fn work(mut self) {
            log::info!("schedule thread start");
            if let Some(new_level) = self.send_level.wait_and_get() {
                debug!("receive new level");
                self.resort_work(new_level);
            }
            let mut clock = Instant::now();
            loop {
                // TODO lock the thread
                let mut len = self.queue.len();
                let mut i = 0;
                let now = Instant::now();
                if now - clock > Duration::from_secs(5) {
                    debug!("queue len {len}");
                    debug!("running tasks {}", self.running_task_count);
                    for x in -20..=20 {
                        let mut s = String::new();
                        for y in -20..=20 {
                            s += Self::get_chunk_stage_id(
                                &self.loaded_chunks,
                                &mut self.proto_chunks,
                                ChunkPos::new(x, y),
                            )
                            .to_string()
                            .as_str();
                            s += " ";
                        }
                        log::info!("{s}");
                    }
                    clock = now;
                }
                // debug!("queue {:?}", self.queue);
                'outer: while i < len {
                    while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                        debug!("receive chunk pos {pos:?}");
                        match data {
                            RecvChunk::IO(chunk) => match chunk {
                                Chunk::Level(data) => {
                                    self.loaded_chunks.insert(pos, data);
                                    self.task_mark.remove(&pos);
                                }
                                Chunk::Proto(data) => {
                                    self.proto_chunks.insert(pos, data);
                                    // 感觉不在这里移除mark不会有什么副作用
                                }
                            },
                            RecvChunk::Generation(data) => {
                                let mut id = 0;
                                for chunk in data.chunks {
                                    let pos = ChunkPos::new(
                                        data.x + id / data.size,
                                        data.y + id % data.size,
                                    );
                                    match chunk {
                                        Chunk::Level(data) => {
                                            log::debug!("loaded chunk add {pos:?}");
                                            self.loaded_chunks.insert(pos, data);
                                            self.task_mark.remove(&pos);
                                        }
                                        Chunk::Proto(chunk) => {
                                            if id == data.size * data.size / 2 {
                                                log::debug!(
                                                    "proto chunk add {pos:?} {:?}",
                                                    StagedChunkEnum::from(chunk.stage_id())
                                                );
                                            }
                                            self.proto_chunks.insert(pos, chunk);
                                            // 感觉不在这里移除mark不会有什么副作用
                                        }
                                    }
                                    self.occupied.remove(&pos);
                                    id += 1;
                                }
                            }
                        }
                        self.running_task_count -= 1;
                    }
                    let (pos, _, stage) = self.queue[i];
                    // debug!("receive request pos {pos:?} stage {stage:?} queue len {len}");
                    let level = *self
                        .last_level
                        .get(&pos)
                        .unwrap_or(&ChunkLoading::MAX_LEVEL);
                    if level == ChunkLoading::MAX_LEVEL {
                        self.drop_mark(stage, pos);
                        self.queue.remove(i);
                        len -= 1;
                        continue;
                    }
                    let highest_stage = StagedChunkEnum::level_to_stage(level);
                    if (highest_stage as u8) < (stage as u8) {
                        self.drop_mark(stage, pos);
                        self.queue.remove(i);
                        len -= 1;
                        continue;
                    }
                    let current_stage =
                        Self::get_chunk_stage_id(&self.loaded_chunks, &mut self.proto_chunks, pos);
                    if current_stage >= (stage as u8) {
                        self.drop_mark(stage, pos);
                        self.queue.remove(i);
                        len -= 1;
                        continue;
                    }
                    if stage == StagedChunkEnum::Empty {
                        self.running_task_count += 1;
                        self.io.send(pos).expect("oi thread close unexpectedly");
                        self.queue.remove(i);
                        len -= 1;
                        continue;
                    }
                    let radius = stage.get_radius();
                    let depend = stage.get_dependencies();
                    for dx in -radius..=radius {
                        for dy in -radius..=radius {
                            let new_pos = pos.add_raw(dx, dy);
                            let dst = max(abs(dx), abs(dy)) as usize;
                            if self.occupied.contains(&new_pos)
                                || Self::get_chunk_stage_id(
                                    &self.loaded_chunks,
                                    &mut self.proto_chunks,
                                    new_pos,
                                ) < (depend[dst] as u8)
                            {
                                i += 1;
                                continue 'outer;
                            }
                        }
                    }
                    let mut cache = Cache::new(pos.x - radius, pos.y - radius, radius * 2 + 1);
                    for dx in -radius..=radius {
                        for dy in -radius..=radius {
                            let new_pos = pos.add_raw(dx, dy);
                            cache.chunks.push(
                                Self::get_chunk(
                                    &self.loaded_chunks,
                                    &mut self.proto_chunks,
                                    new_pos,
                                )
                                .expect("Chunk missing"),
                            );
                            self.occupied.insert(new_pos);
                        }
                    }
                    self.running_task_count += 1;
                    self.generate
                        .send((pos, cache, stage))
                        .expect("oi thread close unexpectedly");
                    self.queue.remove(i);
                    len -= 1;
                }
                if len == 0 {
                    if let Some(new_level) = self.send_level.wait_and_get() {
                        debug!("receive new level");
                        self.resort_work(new_level);
                    }
                } else {
                    if let Some(new_level) = self.send_level.get() {
                        debug!("receive new level");
                        self.resort_work(new_level);
                    }
                }
            }
        }
    }
}
pub struct ChunkRequest {
    pub pos: Vector2<i32>,
    pub response: oneshot::Sender<(SyncChunk, bool)>, // bool = is_new
}

/// The `Level` module provides functionality for working with chunks within or outside a Minecraft world.
///
/// Key features include:
///
/// - **Chunk Loading:** Efficiently loads chunks from disk.
/// - **Chunk Caching:** Stores accessed chunks in memory for faster access.
/// - **Chunk Generation:** Generates new chunks on-demand using a specified `WorldGenerator`.
///
/// For more details on world generation, refer to the `WorldGenerator` module.
pub struct Level {
    pub seed: Seed,
    block_registry: Arc<dyn BlockRegistryExt>,
    level_folder: LevelFolder,

    /// Counts the number of ticks that have been scheduled for this world
    schedule_tick_counts: AtomicU64,

    // Chunks that are paired with chunk watchers. When a chunk is no longer watched, it is removed
    // from the loaded chunks map and sent to the underlying ChunkIO
    pub loaded_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    loaded_entity_chunks: Arc<DashMap<Vector2<i32>, SyncEntityChunk>>,
    pub chunk_loading: Mutex<ChunkLoading>,

    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,

    chunk_saver: Arc<dyn FileIO<Data = SyncChunk>>,
    entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>>,

    world_gen: Arc<VanillaGenerator>,

    /// Tracks tasks associated with this world instance
    tasks: TaskTracker,
    /// Notification that interrupts tasks for shutdown
    pub shutdown_notifier: Notify,
    pub is_shutting_down: AtomicBool,

    gen_request_tx: Sender<Vector2<i32>>,
    pending_generations: Arc<DashMap<Vector2<i32>, Vec<oneshot::Sender<SyncChunk>>>>,

    gen_entity_request_tx: Sender<Vector2<i32>>,
    pending_entity_generations: Arc<DashMap<Vector2<i32>, Vec<oneshot::Sender<SyncEntityChunk>>>>,

    pub sender: Mutex<Option<Arc<LevelChannel>>>,
}

pub struct TickData {
    pub block_ticks: Vec<OrderedTick<&'static Block>>,
    pub fluid_ticks: Vec<OrderedTick<&'static Fluid>>,
    pub random_ticks: Vec<ScheduledTick<()>>,
    pub block_entities: Vec<Arc<dyn BlockEntity>>,
}

#[derive(Clone)]
pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
    pub entities_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(
        root_folder: PathBuf,
        block_registry: Arc<dyn BlockRegistryExt>,
        seed: i64,
        dimension: Dimension,
    ) -> Arc<Self> {
        // If we are using an already existing world we want to read the seed from the level.dat, If not we want to check if there is a seed in the config, if not lets create a random one
        let region_folder = root_folder.join("region");
        if !region_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        }
        let entities_folder = root_folder.join("entities");
        if !entities_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Entities folder");
        }
        let level_folder = LevelFolder {
            root_folder,
            region_folder,
            entities_folder,
        };

        // TODO: Load info correctly based on world format type

        let seed = Seed(seed as u64);
        let world_gen = get_world_gen(seed, dimension).into();

        let chunk_saver: Arc<dyn FileIO<Data = SyncChunk>> = match advanced_config().chunk.format {
            ChunkFormat::Linear => Arc::new(ChunkFileManager::<LinearFile<ChunkData>>::default()),
            ChunkFormat::Anvil => {
                Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkData>>::default())
            }
        };
        let entity_saver: Arc<dyn FileIO<Data = SyncEntityChunk>> =
            match advanced_config().chunk.format {
                ChunkFormat::Linear => {
                    Arc::new(ChunkFileManager::<LinearFile<ChunkEntityData>>::default())
                }
                ChunkFormat::Anvil => {
                    Arc::new(ChunkFileManager::<AnvilChunkFile<ChunkEntityData>>::default())
                }
            };

        let (gen_request_tx, gen_request_rx) = crossbeam::channel::unbounded();
        let pending_generations = Arc::new(DashMap::new());

        let (gen_entity_request_tx, gen_entity_request_rx) = crossbeam::channel::unbounded();
        let pending_entity_generations = Arc::new(DashMap::new());
        let level_ref = Arc::new(Self {
            seed,
            block_registry,
            world_gen,
            level_folder,
            chunk_saver,
            entity_saver,
            schedule_tick_counts: AtomicU64::new(0),
            loaded_chunks: Arc::new(DashMap::new()),
            loaded_entity_chunks: Arc::new(DashMap::new()),
            chunk_loading: Mutex::new(ChunkLoading::new()),
            chunk_watchers: Arc::new(DashMap::new()),
            tasks: TaskTracker::new(),
            shutdown_notifier: Notify::new(),
            is_shutting_down: AtomicBool::new(false),
            gen_request_tx,
            pending_generations: pending_generations.clone(),
            gen_entity_request_tx,
            pending_entity_generations: pending_entity_generations.clone(),
            sender: Mutex::new(None),
        });

        let b = GenerationSchedule::new(&level_ref.tasks, 1, 12, level_ref.clone());
        *level_ref.sender.lock().unwrap() = Some(b);

        //TODO: Investigate optimal number of threads
        let num_threads = num_cpus::get().saturating_sub(1).max(1);

        // Normal Chunks
        for thread_id in 0..num_threads {
            let level_clone = level_ref.clone();
            let pending_clone = pending_generations.clone();
            let rx = gen_request_rx.clone();

            std::thread::spawn(move || {
                while let Ok(pos) = rx.recv() {
                    if level_clone.is_shutting_down.load(Ordering::Relaxed) {
                        break;
                    }

                    log::debug!(
                        "Generating chunk {pos:?}, worker thread {thread_id:?}, queue length {}",
                        rx.len()
                    );

                    // Generate chunk
                    let chunk = level_clone.world_gen.generate_chunk(
                        &level_clone,
                        level_clone.block_registry.as_ref(),
                        &pos,
                    );
                    let arc_chunk = Arc::new(RwLock::new(chunk));

                    // Insert into loaded chunks
                    level_clone.loaded_chunks.insert(pos, arc_chunk.clone());

                    // Fulfill all waiters
                    if let Some(waiters) = pending_clone.remove(&pos) {
                        for tx in waiters.1 {
                            let _ = tx.send(arc_chunk.clone());
                        }
                    }
                }
            });
        }

        // Entity Chunks
        for thread_id in 0..num_threads {
            let level_clone = level_ref.clone();
            let pending_clone = pending_entity_generations.clone();
            let rx = gen_entity_request_rx.clone();

            std::thread::spawn(move || {
                while let Ok(pos) = rx.recv() {
                    if level_clone.is_shutting_down.load(Ordering::Relaxed) {
                        break;
                    }

                    log::debug!(
                        "Generating entity chunk {pos:?}, worker thread {thread_id:?}, queue length {}",
                        rx.len()
                    );

                    let chunk = ChunkEntityData {
                        chunk_position: pos,
                        data: HashMap::new(),
                        dirty: true,
                    };
                    let arc_chunk = Arc::new(RwLock::new(chunk));

                    level_clone
                        .loaded_entity_chunks
                        .insert(pos, arc_chunk.clone());

                    if let Some(waiters) = pending_clone.remove(&pos) {
                        for tx in waiters.1 {
                            let _ = tx.send(arc_chunk.clone());
                        }
                    }
                }
            });
        }
        level_ref
            .chunk_loading
            .lock()
            .unwrap()
            .add_ticket(Vector2::<i32>::new(0, 0), 17);
        level_ref
    }

    async fn load_single_chunk(
        &self,
        pos: Vector2<i32>,
    ) -> Result<(SyncChunk, bool), ChunkReadingError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // Call the existing fetch_chunks with a single chunk
        self.chunk_saver
            .fetch_chunks(&self.level_folder, &[pos], tx)
            .await;

        // Wait for the result
        match rx.recv().await {
            Some(LoadedData::Loaded(chunk)) => Ok((chunk, false)),
            Some(LoadedData::Missing(_)) => Err(ChunkReadingError::ChunkNotExist),
            Some(LoadedData::Error((_, err))) => Err(err),
            None => Err(ChunkReadingError::ChunkNotExist),
        }
    }

    /// Spawns a task associated with this world. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable (no looping) amount of time.
    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.tasks.spawn(task)
    }

    pub async fn shutdown(&self) {
        log::info!("Saving level...");

        self.is_shutting_down.store(true, Ordering::Relaxed);
        self.shutdown_notifier.notify_waiters();

        self.tasks.close();
        log::debug!("Awaiting level tasks");
        self.tasks.wait().await;
        log::debug!("Done awaiting level chunk tasks");

        // wait for chunks currently saving in other threads
        self.chunk_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        self.chunk_saver.clear_watched_chunks().await;
        self.write_chunks(chunks_to_write).await;

        log::debug!("Done awaiting level entity tasks");

        // wait for chunks currently saving in other threads
        self.entity_saver.block_and_await_ongoing_tasks().await;

        // save all chunks currently in memory
        let chunks_to_write = self
            .loaded_entity_chunks
            .iter()
            .map(|chunk| (*chunk.key(), chunk.value().clone()))
            .collect::<Vec<_>>();
        self.loaded_entity_chunks.clear();

        // TODO: I think the chunk_saver should be at the server level
        self.entity_saver.clear_watched_chunks().await;
        self.write_entity_chunks(chunks_to_write).await;
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub async fn clean_up_log(&self) {
        self.chunk_saver.clean_up_log().await;
        self.entity_saver.clean_up_log().await;
    }

    pub fn list_cached(&self) {
        for entry in self.loaded_chunks.iter() {
            log::debug!("In map: {:?}", entry.key());
        }
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub async fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        for chunk in chunks {
            log::trace!("{chunk:?} marked as newly watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    if let Some(new_value) = value.checked_add(1) {
                        *value = new_value;
                        //log::debug!("Watch value for {:?}: {}", chunk, value);
                    } else {
                        log::error!("Watching overflow on chunk {chunk:?}");
                    }
                }
                Entry::Vacant(vacant) => {
                    vacant.insert(1);
                }
            }
        }

        self.chunk_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
        self.entity_saver
            .watch_chunks(&self.level_folder, chunks)
            .await;
    }

    #[inline]
    pub async fn mark_chunk_as_newly_watched(&self, chunk: Vector2<i32>) {
        self.mark_chunks_as_newly_watched(&[chunk]).await;
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub async fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        let mut chunks_to_clean = Vec::new();

        for chunk in chunks {
            log::trace!("{chunk:?} marked as no longer watched");
            match self.chunk_watchers.entry(*chunk) {
                Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    *value = value.saturating_sub(1);

                    if *value == 0 {
                        occupied.remove_entry();
                        chunks_to_clean.push(*chunk);
                    }
                }
                Entry::Vacant(_) => {
                    // This can be:
                    // - Player disconnecting before all packets have been sent
                    // - Player moving so fast that the chunk leaves the render distance before it
                    // is loaded into memory
                }
            }
        }

        self.chunk_saver
            .unwatch_chunks(&self.level_folder, chunks)
            .await;
        self.entity_saver
            .unwatch_chunks(&self.level_folder, chunks)
            .await;
        chunks_to_clean
    }

    /// Returns whether the chunk should be removed from memory
    #[inline]
    pub async fn mark_chunk_as_not_watched(&self, chunk: Vector2<i32>) -> bool {
        !self.mark_chunks_as_not_watched(&[chunk]).await.is_empty()
    }

    pub async fn clean_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_chunks.remove(pos).map(|chunk| (*pos, chunk.1))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();

            level.write_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, chunk) in chunks_to_remove {
                // Add them back if they have watchers
                if level.chunk_watchers.get(&pos).is_some() {
                    let entry = level.loaded_chunks.entry(pos);
                    if let Entry::Vacant(vacant) = entry {
                        vacant.insert(chunk);
                    }
                }
            }
        });
    }

    pub async fn clean_entity_chunks(self: &Arc<Self>, chunks: &[Vector2<i32>]) {
        // Care needs to be take here because of interweaving case:
        // 1) Remove chunk from cache
        // 2) Another player wants same chunk
        // 3) Load (old) chunk from serializer
        // 4) Write (new) chunk from serializer
        // Now outdated chunk data is cached and will be written later

        let chunks_with_no_watchers = chunks
            .iter()
            .filter_map(|pos| {
                // Only chunks that have no entry in the watcher map or have 0 watchers
                if self
                    .chunk_watchers
                    .get(pos)
                    .is_none_or(|count| count.is_zero())
                {
                    self.loaded_entity_chunks
                        .get(pos)
                        .map(|chunk| (*pos, chunk.value().clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let level = self.clone();
        self.spawn_task(async move {
            let chunks_to_remove = chunks_with_no_watchers.clone();
            level.write_entity_chunks(chunks_with_no_watchers).await;
            // Only after we have written the chunks to the serializer do we remove them from the
            // cache
            for (pos, _) in chunks_to_remove {
                let _ = level.loaded_entity_chunks.remove_if(&pos, |_, _| {
                    // Recheck that there is no one watching
                    level
                        .chunk_watchers
                        .get(&pos)
                        .is_none_or(|count| count.is_zero())
                });
            }
        });
    }

    // Gets random ticks, block ticks and fluid ticks
    pub async fn get_tick_data(&self) -> TickData {
        let mut ticks = TickData {
            block_ticks: Vec::new(),
            fluid_ticks: Vec::new(),
            random_ticks: Vec::with_capacity(self.loaded_chunks.len() * 3 * 16 * 16),
            block_entities: Vec::new(),
        };

        let mut rng = SmallRng::from_os_rng();
        for chunk in self.loaded_chunks.iter() {
            let mut chunk = chunk.write().await;
            ticks.block_ticks.append(&mut chunk.block_ticks.step_tick());
            ticks.fluid_ticks.append(&mut chunk.fluid_ticks.step_tick());

            let chunk = chunk.downgrade();

            let chunk_x_base = chunk.position.x * 16;
            let chunk_z_base = chunk.position.y * 16;

            let mut section_blocks = Vec::new();
            for i in 0..chunk.section.sections.len() {
                let mut section_block_data = Vec::new();

                //TODO use game rules to determine how many random ticks to perform
                for _ in 0..3 {
                    let r = rng.random::<u32>();
                    let x_offset = (r & 0xF) as i32;
                    let y_offset = ((r >> 4) & 0xF) as i32 - 32;
                    let z_offset = (r >> 8 & 0xF) as i32;

                    let random_pos = BlockPos::new(
                        chunk_x_base + x_offset,
                        i as i32 * 16 + y_offset,
                        chunk_z_base + z_offset,
                    );

                    let block_id = chunk
                        .section
                        .get_block_absolute_y(x_offset as usize, random_pos.0.y, z_offset as usize)
                        .unwrap_or(Block::AIR.default_state.id);

                    section_block_data.push((random_pos, block_id));
                }
                section_blocks.push(section_block_data);
            }

            for section_data in section_blocks {
                for (random_pos, block_id) in section_data {
                    if has_random_ticks(block_id) {
                        ticks.random_ticks.push(ScheduledTick {
                            position: random_pos,
                            delay: 0,
                            priority: TickPriority::Normal,
                            value: (),
                        });
                    }
                }
            }

            ticks
                .block_entities
                .extend(chunk.block_entities.values().cloned());
        }

        ticks.block_ticks.sort_unstable();
        ticks.fluid_ticks.sort_unstable();

        ticks
    }

    pub async fn clean_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        // self.clean_chunks(&[*chunk]).await;
    }

    pub async fn clean_entity_chunk(self: &Arc<Self>, chunk: &Vector2<i32>) {
        self.clean_entity_chunks(&[*chunk]).await;
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self) {
        self.chunk_watchers.retain(|_, watcher| !watcher.is_zero());
        self.loaded_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());
        self.loaded_entity_chunks
            .retain(|at, _| self.chunk_watchers.get(at).is_some());

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.chunk_watchers.capacity() - self.chunk_watchers.len() >= 4096 {
            self.chunk_watchers.shrink_to_fit();
        }

        // if the difference is too big, we can shrink the loaded chunks
        // (1024 chunks is the equivalent to a 32x32 chunks area)
        if self.loaded_chunks.capacity() - self.loaded_chunks.len() >= 4096 {
            self.loaded_chunks.shrink_to_fit();
        }

        if self.loaded_entity_chunks.capacity() - self.loaded_entity_chunks.len() >= 4096 {
            self.loaded_entity_chunks.shrink_to_fit();
        }
    }

    pub async fn get_chunk(self: &Arc<Self>, pos: Vector2<i32>) -> SyncChunk {
        // Already loaded?
        if let Some(chunk) = self.loaded_chunks.get(&pos) {
            return chunk.clone();
        } else {
            panic!("not chunk found at {pos:?}");
        }

        // Try to load from disk
        match self.load_single_chunk(pos).await {
            Ok((chunk, _)) => {
                self.loaded_chunks.insert(pos, chunk.clone());
                chunk
            }
            Err(_) => {
                // Need to generate
                let (tx, rx) = oneshot::channel();

                // Deduplication
                match self.pending_generations.entry(pos) {
                    dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(tx);
                    }
                    dashmap::mapref::entry::Entry::Vacant(entry) => {
                        entry.insert(vec![tx]);
                        let _ = self.gen_request_tx.send(pos);
                    }
                }

                rx.await.expect("Generation worker dropped")
            }
        }
    }

    // Stream the chunks (don't collect them and then do stuff with them)
    /// Spawns a tokio task to stream chunks.
    /// Important: must be called from an async function (or changed to accept a tokio runtime
    /// handle)
    pub fn receive_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let level = self.clone();

        log::trace!("Receiving chunks: {}", chunks.len());

        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();

            let fetch_task = async {
                // Separate already-loaded chunks from ones we need to fetch
                let mut to_fetch = Vec::new();
                for pos in &chunks {
                    if let Some(chunk) = level.loaded_chunks.get(pos) {
                        let _ = sender.send((chunk.clone(), false));
                    } else {
                        to_fetch.push(*pos);
                        panic!("not chunk found at {pos:?}");
                    }
                }

                if !to_fetch.is_empty() {
                    // Channel for fetch_chunks to send results
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<
                        LoadedData<SyncChunk, ChunkReadingError>,
                    >(to_fetch.len());

                    // Fetch all missing chunks from disk in one go
                    level
                        .chunk_saver
                        .fetch_chunks(&level.level_folder, &to_fetch, tx)
                        .await;

                    // Process loaded/missing/error results
                    while let Some(data) = rx.recv().await {
                        match data {
                            LoadedData::Loaded(chunk) => {
                                let pos = chunk.read().await.position;
                                level.loaded_chunks.insert(pos, chunk.clone());
                                let _ = sender.send((chunk, false));
                            }
                            LoadedData::Missing(pos) | LoadedData::Error((pos, _)) => {
                                panic!("chunk found at {pos:?}");
                                let sender_clone = sender.clone();

                                // Need to generate — but don't block here
                                let level_clone = level.clone();

                                tokio::spawn(async move {
                                    let (tx, rx) = oneshot::channel();

                                    match level_clone.pending_generations.entry(pos) {
                                        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                                            entry.get_mut().push(tx);
                                        }
                                        dashmap::mapref::entry::Entry::Vacant(entry) => {
                                            entry.insert(vec![tx]);
                                            let _ = level_clone.gen_request_tx.send(pos);
                                        }
                                    }

                                    if let Ok(chunk) = rx.await {
                                        let _ = sender_clone.send((chunk, true));
                                    }
                                });
                            }
                        }
                    }
                }
            };

            // Stop early if shutting down
            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            };
        });

        receiver
    }

    async fn load_single_entity_chunk(
        &self,
        pos: Vector2<i32>,
    ) -> Result<(SyncEntityChunk, bool), ChunkReadingError> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        self.entity_saver
            .fetch_chunks(&self.level_folder, &[pos], tx)
            .await;

        match rx.recv().await {
            Some(LoadedData::Loaded(chunk)) => Ok((chunk, false)),
            Some(LoadedData::Missing(_)) => Err(ChunkReadingError::ChunkNotExist),
            Some(LoadedData::Error((_, err))) => Err(err),
            None => Err(ChunkReadingError::ChunkNotExist),
        }
    }

    pub fn receive_entity_chunks(
        self: &Arc<Self>,
        chunks: Vec<Vector2<i32>>,
    ) -> UnboundedReceiver<(SyncEntityChunk, bool)> {
        let (sender, receiver) = mpsc::unbounded_channel();
        let level = self.clone();

        self.spawn_task(async move {
            let cancel_notifier = level.shutdown_notifier.notified();

            let fetch_task = async {
                let mut to_fetch = Vec::new();
                for pos in &chunks {
                    if let Some(chunk) = level.loaded_entity_chunks.get(pos) {
                        let _ = sender.send((chunk.clone(), false));
                    } else {
                        to_fetch.push(*pos);
                    }
                }

                if !to_fetch.is_empty() {
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<
                        LoadedData<SyncEntityChunk, ChunkReadingError>,
                    >(to_fetch.len());

                    level
                        .entity_saver
                        .fetch_chunks(&level.level_folder, &to_fetch, tx)
                        .await;

                    while let Some(data) = rx.recv().await {
                        match data {
                            LoadedData::Loaded(chunk) => {
                                let pos = chunk.read().await.chunk_position;
                                level.loaded_entity_chunks.insert(pos, chunk.clone());
                                let _ = sender.send((chunk, false));
                            }
                            LoadedData::Missing(pos) | LoadedData::Error((pos, _)) => {
                                let sender_clone = sender.clone();
                                let level_clone = level.clone();

                                tokio::spawn(async move {
                                    let (tx, rx) = oneshot::channel();
                                    match level_clone.pending_entity_generations.entry(pos) {
                                        dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                                            entry.get_mut().push(tx);
                                        }
                                        dashmap::mapref::entry::Entry::Vacant(entry) => {
                                            entry.insert(vec![tx]);
                                            let _ = level_clone.gen_entity_request_tx.send(pos);
                                        }
                                    }
                                    if let Ok(chunk) = rx.await {
                                        let _ = sender_clone.send((chunk, true));
                                    }
                                });
                            }
                        }
                    }
                }
            };

            select! {
                () = cancel_notifier => {},
                () = fetch_task => {}
            }
        });

        receiver
    }

    pub async fn get_entity_chunk(self: &Arc<Self>, pos: Vector2<i32>) -> SyncEntityChunk {
        if let Some(chunk) = self.loaded_entity_chunks.get(&pos) {
            return chunk.clone();
        }

        match self.load_single_entity_chunk(pos).await {
            Ok((chunk, _)) => {
                self.loaded_entity_chunks.insert(pos, chunk.clone());
                chunk
            }
            Err(_) => {
                let (tx, rx) = oneshot::channel();
                match self.pending_entity_generations.entry(pos) {
                    dashmap::mapref::entry::Entry::Occupied(mut entry) => {
                        entry.get_mut().push(tx);
                    }
                    dashmap::mapref::entry::Entry::Vacant(entry) => {
                        entry.insert(vec![tx]);
                        let _ = self.gen_entity_request_tx.send(pos);
                    }
                }
                rx.await.expect("Entity generation worker dropped")
            }
        }
    }

    pub async fn get_block_state(self: &Arc<Self>, position: &BlockPos) -> RawBlockState {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;

        let Some(id) = chunk.read().await.section.get_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
        ) else {
            return RawBlockState(Block::VOID_AIR.default_state.id);
        };

        RawBlockState(id)
    }
    pub async fn get_rough_biome(self: &Arc<Self>, position: &BlockPos) -> &'static Biome {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;

        let Some(id) = chunk.read().await.section.get_rough_biome_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
        ) else {
            return &Biome::THE_VOID;
        };

        Biome::from_id(id).unwrap()
    }

    pub async fn set_block_state(
        self: &Arc<Self>,
        position: &BlockPos,
        block_state_id: BlockStateId,
    ) -> BlockStateId {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = self.get_chunk(chunk_coordinate).await;
        let mut chunk = chunk.write().await;

        let replaced_block_state_id = chunk.section.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            block_state_id,
        );
        if replaced_block_state_id != block_state_id {
            chunk.mark_dirty(true);
        }
        replaced_block_state_id
    }

    pub async fn write_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.chunk_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    pub async fn write_entity_chunks(&self, chunks_to_write: Vec<(Vector2<i32>, SyncEntityChunk)>) {
        if chunks_to_write.is_empty() {
            return;
        }

        let chunk_saver = self.entity_saver.clone();
        let level_folder = self.level_folder.clone();

        trace!("Sending chunks to ChunkIO {:}", chunks_to_write.len());
        if let Err(error) = chunk_saver
            .save_chunks(&level_folder, chunks_to_write)
            .await
        {
            log::error!("Failed writing Chunk to disk {error}");
        }
    }

    pub fn try_get_chunk(
        &self,
        coordinates: &Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkData>>>> {
        self.loaded_chunks.try_get(coordinates).try_unwrap()
    }

    pub fn try_get_entity_chunk(
        &self,
        coordinates: Vector2<i32>,
    ) -> Option<dashmap::mapref::one::Ref<'_, Vector2<i32>, Arc<RwLock<ChunkEntityData>>>> {
        self.loaded_entity_chunks.try_get(&coordinates).try_unwrap()
    }

    pub async fn schedule_block_tick(
        self: &Arc<Self>,
        block: &Block,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk = chunk.write().await;
        chunk.block_ticks.schedule_tick(
            ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*(block as *const Block) },
            },
            self.schedule_tick_counts.load(Ordering::Relaxed),
        );
        self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn schedule_fluid_tick(
        self: &Arc<Self>,
        fluid: &Fluid,
        block_pos: BlockPos,
        delay: u8,
        priority: TickPriority,
    ) {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let mut chunk = chunk.write().await;
        chunk.fluid_ticks.schedule_tick(
            ScheduledTick {
                delay,
                position: block_pos,
                priority,
                value: unsafe { &*(fluid as *const Fluid) },
            },
            self.schedule_tick_counts.load(Ordering::Relaxed),
        );
        self.schedule_tick_counts.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn is_block_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        block: &Block,
    ) -> bool {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let chunk = chunk.read().await;
        chunk.block_ticks.is_scheduled(*block_pos, block)
    }

    pub async fn is_fluid_tick_scheduled(
        self: &Arc<Self>,
        block_pos: &BlockPos,
        fluid: &Fluid,
    ) -> bool {
        let chunk = self
            .get_chunk(block_pos.chunk_and_chunk_relative_position().0)
            .await;
        let chunk = chunk.read().await;
        chunk.fluid_ticks.is_scheduled(*block_pos, fluid)
    }
}
