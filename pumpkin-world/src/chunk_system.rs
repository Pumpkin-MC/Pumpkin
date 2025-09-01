use crate::block::RawBlockState;
use crate::chunk::io::LoadedData::Loaded;
use crate::chunk::{ChunkData, ChunkHeightmapType, ChunkLight, ChunkSections, SubChunk};
use crate::dimension::Dimension;

use crate::generation::height_limit::HeightLimitView;

use crate::generation::proto_chunk::{GenerationCache, TerrainCache};
use crate::generation::settings::{GenerationSettings, gen_settings_from_dimension};
use crate::level::{Level, SyncChunk};
use crate::world::{BlockAccessor, BlockRegistryExt};
use crate::{GlobalRandomConfig, ProtoChunk, ProtoNoiseRouters};
use async_trait::async_trait;
use crossbeam::channel::{Receiver, Sender};
use dashmap::DashMap;
use itertools::Itertools;
use log::debug;
use num_traits::abs;
use pumpkin_data::biome::Biome;

use pumpkin_data::{Block, BlockState};
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use std::cmp::{Ordering, PartialEq, max};
use std::collections::BinaryHeap;
use std::collections::hash_map::Entry;
use std::mem::swap;
use std::sync::{Arc, Condvar, Mutex};

use crate::chunk::format::LightContainer;
use crate::chunk::palette::{BiomePalette, BlockPalette};
use crate::chunk_system::Chunk::Proto;
use crate::chunk_system::StagedChunkEnum::{Biomes, Empty, Features, Full, Noise, Surface};
use crate::generation::biome_coords;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::atomic::Ordering::Relaxed;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, oneshot};
use tokio::task;
use tokio_util::task::TaskTracker;

type HashMapType<K, V> = FxHashMap<K, V>;
type HashSetType<K> = FxHashSet<K>;
type ChunkPos = Vector2<i32>;
type ChunkLevel = HashMapType<ChunkPos, i8>;

struct HeapNode(i8, ChunkPos);
impl PartialEq for HeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for HeapNode {}
impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}
impl From<(ChunkPos, i8)> for HeapNode {
    fn from(value: (ChunkPos, i8)) -> Self {
        Self(value.1, value.0)
    }
}
impl From<HeapNode> for (ChunkPos, i8) {
    fn from(val: HeapNode) -> Self {
        (val.1, val.0)
    }
}

pub struct ChunkLoading {
    pub is_dirty: bool,
    pub pos_level: ChunkLevel,
    pub ticket: HashMapType<ChunkPos, Vec<i8>>, // TODO lifetime & id
    pub sender: Arc<LevelChannel>,
}

impl ChunkLoading {
    pub const FULL_CHUNK_LEVEL: i8 = 33;
    pub const MAX_LEVEL: i8 = 46; // level 46 will be unloaded.
    pub fn dump_level_debug(map: &ChunkLevel, sx: i32, tx: i32, sy: i32, ty: i32) {
        let mut header = "X/Y".to_string();
        for y in sy..=ty {
            header.push_str(&format!("{y:4}"));
        }

        let grid: String = (sx..=tx)
            .map(|x| {
                let mut row = format!("{x:3}");
                row.push_str(
                    &(sy..=ty)
                        .map(|y| {
                            format!(
                                "{:4}",
                                map.get(&ChunkPos::new(x, y))
                                    .unwrap_or(&ChunkLoading::MAX_LEVEL)
                            )
                        })
                        .collect::<String>(),
                );
                row
            })
            .collect::<Vec<_>>()
            .join("\n");

        debug!("\nloading level:\n{header}\n{grid}");
    }

    pub const fn get_level_from_view_distance(view_distance: u8) -> i8 {
        Self::FULL_CHUNK_LEVEL + 1 - (view_distance as i8)
    }

    pub fn new(sender: Arc<LevelChannel>) -> Self {
        Self {
            is_dirty: true,
            pos_level: ChunkLevel::default(),
            ticket: HashMapType::default(),
            sender,
        }
    }

    pub fn send_change(&mut self) {
        if self.is_dirty {
            self.is_dirty = false;
            self.sender.set(self.get_cloned_level());
        }
    }

    fn run_update(pos_level: &mut ChunkLevel, mut queue: BinaryHeap<HeapNode>) {
        while let Some(node) = queue.pop() {
            let (pos, level) = node.into();
            debug_assert!(level < Self::MAX_LEVEL);
            if level > *pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL) {
                continue;
            }
            debug_assert_eq!(level, *pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL));
            let spread_level = level + 1;
            if spread_level >= Self::MAX_LEVEL {
                continue;
            }
            for dx in -1..2 {
                for dy in -1..2 {
                    let new_pos = pos.add_raw(dx, dy);
                    if new_pos != pos {
                        Self::check_then_push(pos_level, &mut queue, new_pos, spread_level);
                    }
                }
            }
        }
    }

    fn check_then_push(
        pos_level: &mut ChunkLevel,
        queue: &mut BinaryHeap<HeapNode>,
        pos: ChunkPos,
        level: i8,
    ) {
        match pos_level.entry(pos) {
            Entry::Occupied(mut entry) => {
                let old = entry.get_mut();
                if *old <= level {
                    return;
                }
                *old = level;
            }
            Entry::Vacant(empty) => {
                empty.insert(level);
            }
        }
        queue.push((pos, level).into());
    }

    fn run_increase_update(&mut self, pos: ChunkPos, level: i8) {
        // TODO there will be a faster way
        debug_assert!(level < Self::MAX_LEVEL);
        let range = Self::MAX_LEVEL - level - 1;
        let mut queue = BinaryHeap::new();
        for dx in -range..=range {
            for dy in -range..=range {
                let new_pos = pos.add_raw(dx as i32, dy as i32);
                let level_from_source = level + abs(dx).max(abs(dy));
                match self.pos_level.entry(new_pos) {
                    Entry::Occupied(entry) => {
                        let old = *entry.get();
                        if old < level_from_source {
                            queue.push((new_pos, old).into());
                            continue;
                        }
                        debug_assert!(old == level_from_source);
                        entry.remove();
                    }
                    Entry::Vacant(_) => {
                        panic!("pos {new_pos:?} should contain a level");
                    }
                }
            }
        }
        let min_x = pos.x - range as i32 - 1;
        let max_x = pos.x + range as i32 + 1;
        let min_y = pos.y - range as i32 - 1;
        let max_y = pos.y + range as i32 + 1;
        for y in min_y..max_y {
            let mut new_pos = ChunkPos::new(max_x, y);
            let level = self.pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL) + 1;
            if level < Self::MAX_LEVEL {
                new_pos.x -= 1;
                Self::check_then_push(&mut self.pos_level, &mut queue, new_pos, level);
            }
        }
        for x in min_x..max_x {
            let mut new_pos = ChunkPos::new(x, min_y);
            let level = self.pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL) + 1;
            if level < Self::MAX_LEVEL {
                new_pos.y += 1;
                Self::check_then_push(&mut self.pos_level, &mut queue, new_pos, level);
            }
        }
        for y in (min_y + 1)..=max_y {
            let mut new_pos = ChunkPos::new(min_x, y);
            let level = self.pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL) + 1;
            if level < Self::MAX_LEVEL {
                new_pos.x += 1;
                Self::check_then_push(&mut self.pos_level, &mut queue, new_pos, level);
            }
        }
        for x in (min_x + 1)..=max_x {
            let mut new_pos = ChunkPos::new(x, max_y);
            let level = self.pos_level.get(&new_pos).unwrap_or(&Self::MAX_LEVEL) + 1;
            if level < Self::MAX_LEVEL {
                new_pos.y -= 1;
                Self::check_then_push(&mut self.pos_level, &mut queue, new_pos, level);
            }
        }
        for (ticket_pos, levels) in &self.ticket {
            if abs(ticket_pos.x - pos.x) <= range as i32
                && abs(ticket_pos.y - pos.y) <= range as i32
            {
                Self::check_then_push(
                    &mut self.pos_level,
                    &mut queue,
                    *ticket_pos,
                    *levels.iter().max().unwrap(),
                );
            }
        }
        Self::run_update(&mut self.pos_level, queue);
    }
    pub fn add_ticket(&mut self, pos: ChunkPos, level: i8) {
        log::debug!("add ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        match self.ticket.entry(pos) {
            Entry::Occupied(mut vec) => {
                vec.get_mut().push(level);
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![level]);
            }
        }
        match self.pos_level.entry(pos) {
            Entry::Occupied(mut entry) => {
                let old = entry.get_mut();
                if *old < level {
                    return;
                }
                self.is_dirty = true;
                *old = level;
            }
            Entry::Vacant(empty) => {
                empty.insert(level);
            }
        }
        let mut queue: BinaryHeap<HeapNode> = BinaryHeap::new();
        queue.push((pos, level).into());
        Self::run_update(&mut self.pos_level, queue);
    }
    pub fn remove_ticket(&mut self, pos: ChunkPos, level: i8) {
        log::debug!("remove ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        let Some(vec) = self.ticket.get_mut(&pos) else {
            log::warn!("No ticket found at {pos:?}");
            return;
        };
        let Some((index, _)) = vec.iter().find_position(|x| **x == level) else {
            log::warn!("No ticket found at {pos:?}");
            return;
        };
        vec.remove(index);
        let old_level = *self.pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL);
        let source = *vec.iter().max().unwrap_or(&Self::MAX_LEVEL);
        if vec.is_empty() {
            self.ticket.remove(&pos);
        }
        if level == old_level && source != level {
            self.is_dirty = true;
            self.run_increase_update(pos, old_level);
        }
    }
    fn get_cloned_level(&self) -> ChunkLevel {
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
            1 => Empty,
            2 => Biomes,
            3 => Noise,
            4 => Surface,
            5 => Features,
            6 => Full,
            _ => panic!(),
        }
    }
}
impl StagedChunkEnum {
    fn level_to_stage(level: i8) -> Self {
        if level <= 33 {
            Full
        } else if level <= 35 {
            Features
        } else if level <= 36 {
            Surface
        } else if level <= 37 {
            Biomes
        } else if level <= 45 {
            Empty
        } else {
            Self::None
        }
    }
    fn get_radius(self) -> i32 {
        // self exclude
        match self {
            Empty => 0,
            Biomes => 8,
            Noise => 9,
            Surface => 9,
            Features => 10,
            Full => 11,
            _ => panic!(),
        }
    }
    fn get_write_radius(self) -> i32 {
        // self exclude
        match self {
            Empty => 0,
            Biomes => 0,
            Noise => 0,
            Surface => 0,
            Features => 1,
            Full => 0,
            _ => panic!(),
        }
    }
    fn get_dependencies(self) -> &'static [StagedChunkEnum] {
        match self {
            Biomes => &[
                Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ],
            Noise => &[
                Biomes, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ],
            Surface => &[
                Noise, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ],
            Features => &[
                Surface, Surface, Biomes, Empty, Empty, Empty, Empty, Empty, Empty, Empty, Empty,
            ],
            Full => &[
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

impl Default for LevelChannel {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn wait_and_get(&self, level: &Arc<Level>) -> Option<ChunkLevel> {
        let mut lock = self.value.lock().unwrap();
        while lock.is_none() && !level.should_unload.load(Relaxed) && !level.should_save.load(Relaxed) && !level.shut_down_chunk_system.load(Relaxed) {
            lock = self.notify.wait(lock).unwrap();
        }
        
        let mut ret = None;
        swap(&mut ret, &mut *lock);
        ret
    }
    pub fn notify(&self) {
        let val = self.value.lock().unwrap();
        drop(val);
        self.notify.notify_one();
    }
}

pub enum Chunk {
    Level(SyncChunk),
    Proto(ProtoChunk),
}

impl Chunk {
    fn get_stage_id(&self) -> u8 {
        match self {
            Chunk::Proto(data) => data.stage_id(),
            Chunk::Level(_) => 6,
        }
    }
    fn get_proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            Chunk::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Proto(chunk) => chunk,
        }
    }
    fn upgrade_to_full(&mut self, generation_settings: &GenerationSettings, dimension: Dimension) {
        let proto_chunk = self.get_proto_chunk_mut();
        debug_assert_eq!(proto_chunk.stage, StagedChunkEnum::Features);
        let height: usize = match dimension {
            Dimension::Overworld => 384,
            Dimension::Nether | Dimension::End => 256,
        };
        let sub_chunks = height / BlockPalette::SIZE;
        let sections = (0..sub_chunks).map(|_| SubChunk::default()).collect();
        let mut sections = ChunkSections::new(sections, generation_settings.shape.min_y as i32);

        for y in 0..biome_coords::from_block(generation_settings.shape.height) {
            let relative_y = y as usize;
            let section_index = relative_y / BiomePalette::SIZE;
            let relative_y = relative_y % BiomePalette::SIZE;
            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..BiomePalette::SIZE {
                    for x in 0..BiomePalette::SIZE {
                        let absolute_y =
                            biome_coords::from_block(generation_settings.shape.min_y as i32)
                                + y as i32;
                        let biome =
                            proto_chunk.get_biome(&Vector3::new(x as i32, absolute_y, z as i32));
                        section.biomes.set(x, relative_y, z, biome.id);
                    }
                }
            }
        }
        for y in 0..generation_settings.shape.height {
            let relative_y = (y as i32 - sections.min_y) as usize;
            let section_index = relative_y / BlockPalette::SIZE;
            let relative_y = relative_y % BlockPalette::SIZE;
            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..BlockPalette::SIZE {
                    for x in 0..BlockPalette::SIZE {
                        let absolute_y = generation_settings.shape.min_y as i32 + y as i32;
                        let block = proto_chunk
                            .get_block_state(&Vector3::new(x as i32, absolute_y, z as i32));
                        section.block_states.set(x, relative_y, z, block.0);
                    }
                }
            }
        }
        let mut chunk = ChunkData {
            light_engine: ChunkLight {
                sky_light: (0..sections.sections.len() + 2)
                    .map(|_| LightContainer::new_filled(15))
                    .collect(),
                block_light: (0..sections.sections.len() + 2)
                    .map(|_| LightContainer::new_empty(15))
                    .collect(),
            },
            section: sections,
            heightmap: Default::default(),
            position: proto_chunk.chunk_pos,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            block_entities: Default::default(),
        };

        chunk.heightmap = chunk.calculate_heightmap();
        *self = Chunk::Level(Arc::new(RwLock::new(chunk)));
    }
}

struct Cache {
    x: i32,
    y: i32,
    size: i32,
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
        let mid = ((self.size * self.size) >> 1) as usize;
        self.chunks[mid].get_proto_chunk_mut()
    }

    fn get_block_state(&self, pos: &Vector3<i32>) -> RawBlockState {
        let dx = (pos.x >> 4) - self.x;
        let dz = (pos.z >> 4) - self.y;
        debug_assert!(dx < self.size && dz < self.size);
        debug_assert!(dx >= 0 && dz >= 0);
        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => {
                let chunk = data.blocking_read();
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
            Chunk::Proto(data) => data.get_block_state(pos),
        }
    }
    fn set_block_state(&mut self, pos: &Vector3<i32>, block_state: &BlockState) {
        let dx = (pos.x >> 4) - self.x;
        let dz = (pos.z >> 4) - self.y;
        debug_assert!(dx < self.size && dz < self.size);
        debug_assert!(dx >= 0 && dz >= 0);
        match &mut self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => {
                let mut chunk = data.blocking_write();
                let min_y = chunk.section.min_y;
                chunk.section.set_block_absolute_y(
                    (pos.x & 15) as usize,
                    pos.y - min_y,
                    (pos.z & 15) as usize,
                    block_state.id,
                );
            }
            Chunk::Proto(data) => {
                data.set_block_state(pos, block_state);
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
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let chunk = data.blocking_read();
                chunk.heightmap.get_height(
                    ChunkHeightmapType::MotionBlocking,
                    pos.x,
                    pos.y,
                    chunk.section.min_y,
                )
            }
            Chunk::Proto(data) => data.top_motion_blocking_block_height_exclusive(pos),
        }
    }

    fn top_motion_blocking_block_no_leaves_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let dx = (pos.x >> 4) - self.x;
        let dy = (pos.y >> 4) - self.y;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let chunk = data.blocking_read();
                chunk.heightmap.get_height(
                    ChunkHeightmapType::MotionBlockingNoLeaves,
                    pos.x,
                    pos.y,
                    chunk.section.min_y,
                )
            }
            Chunk::Proto(data) => data.top_motion_blocking_block_no_leaves_height_exclusive(pos),
        }
    }

    fn top_block_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let dx = (pos.x >> 4) - self.x;
        let dy = (pos.y >> 4) - self.y;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                let chunk = data.blocking_read();
                chunk.heightmap.get_height(
                    ChunkHeightmapType::WorldSurface,
                    pos.x,
                    pos.y,
                    chunk.section.min_y,
                ) // can we return this?
            }
            Chunk::Proto(data) => data.top_block_height_exclusive(pos),
        }
    }

    fn ocean_floor_height_exclusive(&self, pos: &Vector2<i32>) -> i32 {
        let dx = (pos.x >> 4) - self.x;
        let dy = (pos.y >> 4) - self.y;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(_data) => {
                0 // todo missing
            }
            Chunk::Proto(data) => data.ocean_floor_height_exclusive(pos),
        }
    }

    fn get_biome_for_terrain_gen(&self, global_block_pos: &Vector3<i32>) -> &'static Biome {
        let dx = (global_block_pos.x >> 4) - self.x;
        let dy = (global_block_pos.z >> 4) - self.y;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                // Could this happen?
                Biome::from_id(
                    data.blocking_read()
                        .section
                        .get_rough_biome_absolute_y(
                            (global_block_pos.x & 15) as usize,
                            global_block_pos.y,
                            (global_block_pos.z & 15) as usize,
                        )
                        .unwrap_or(0),
                )
                    .unwrap()
            }
            Chunk::Proto(data) => data.get_biome_for_terrain_gen(global_block_pos),
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
        Cache {
            x,
            y,
            size,
            chunks: Vec::with_capacity((size * size) as usize),
        }
    }
    #[allow(clippy::too_many_arguments)]
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
        let mid = ((self.size * self.size) >> 1) as usize;
        match stage {
            Empty => panic!("empty stage"),
            Biomes => self.chunks[mid]
                .get_proto_chunk_mut()
                .step_to_biomes(dimension, noise_router),
            Noise => self.chunks[mid].get_proto_chunk_mut().step_to_noise(
                settings,
                random_config,
                noise_router,
            ),
            Surface => self.chunks[mid].get_proto_chunk_mut().step_to_surface(
                settings,
                random_config,
                terrain_cache,
                noise_router,
            ),
            Features => {
                ProtoChunk::generate_features_and_structure(self, block_registry, random_config)
            }
            Full => self.chunks[mid].upgrade_to_full(settings, dimension),
            _ => panic!("unknown stage {stage:?}"),
        }
    }
}

enum RecvChunk {
    IO(Chunk),
    Generation(Cache),
}

pub struct ChunkListener {
    single: Mutex<Vec<(ChunkPos, oneshot::Sender<SyncChunk>)>>,
    global: Mutex<Vec<Sender<(ChunkPos, SyncChunk)>>>,
}
impl Default for ChunkListener {
    fn default() -> Self {
        Self::new()
    }
}
impl ChunkListener {
    pub fn new() -> Self {
        Self {
            single: Mutex::new(Vec::new()),
            global: Mutex::new(Vec::new()),
        }
    }
    pub fn add_single_chunk_listener(&self, pos: ChunkPos) -> oneshot::Receiver<SyncChunk> {
        let (tx, rx) = oneshot::channel();
        self.single.lock().unwrap().push((pos, tx));
        rx
    }
    pub fn add_global_chunk_listener(&self) -> Receiver<(ChunkPos, SyncChunk)> {
        let (tx, rx) = crossbeam::channel::unbounded();
        self.global.lock().unwrap().push(tx);
        rx
    }
    fn process_new_chunk(&self, pos: ChunkPos, chunk: &SyncChunk) {
        {
            let mut single = self.single.lock().unwrap();
            let mut i = 0;
            let mut len = single.len();
            while i < len {
                if single[i].0 == pos {
                    let (_, send) = single.remove(i);
                    let _ = send.send(chunk.clone());
                    log::debug!("single listener send {pos:?}");
                    len -= 1;
                    continue;
                }
                if single[i].1.is_closed() {
                    let listener_pos = single[i].0;
                    single.remove(i);
                    log::debug!("single listener dropped {listener_pos:?}");
                    len -= 1;
                    continue;
                }
                i += 1;
            }
        }
        {
            let mut global = self.global.lock().unwrap();
            let mut i = 0;
            let mut len = global.len();
            while i < len {
                match global[i].send((pos, chunk.clone())) {
                    Ok(_) => {}
                    Err(_) => {
                        log::debug!("one global listener dropped");
                        global.remove(i);
                        len -= 1;
                        continue;
                    }
                }
                i += 1;
            }
        }
    }
}

pub struct GenerationSchedule {
    queue: Vec<(ChunkPos, i8, StagedChunkEnum)>,
    last_level: ChunkLevel,
    send_level: Arc<LevelChannel>,
    loaded_chunks: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    proto_chunks: HashMapType<ChunkPos, ProtoChunk>,
    unload_chunks: HashMapType<ChunkPos, Chunk>,
    occupied: HashSetType<ChunkPos>,
    task_mark: HashMapType<ChunkPos, (u8, u8)>,
    running_task_count: u16,
    recv_chunk: Receiver<(ChunkPos, RecvChunk)>,
    io_read: Sender<ChunkPos>,
    io_write: Sender<Vec<(ChunkPos, SyncChunk)>>,
    generate: Sender<(ChunkPos, Cache, StagedChunkEnum)>,
    listener: Arc<ChunkListener>,
}

impl GenerationSchedule {
    pub fn create(
        tracker: &TaskTracker,
        oi_read_thread_count: usize,
        gen_thread_count: usize,
        level: Arc<Level>,
        level_channel: Arc<LevelChannel>,
        listener: Arc<ChunkListener>,
        thread_tracker: &mut Vec<JoinHandle<()>>,
    ) {
        let (send_chunk, recv_chunk) = crossbeam::channel::unbounded();
        let (send_read_io, recv_read_io) = crossbeam::channel::bounded(oi_read_thread_count + 2);
        let (send_write_io, recv_write_io) = crossbeam::channel::unbounded();
        let (send_gen, recv_gen) = crossbeam::channel::bounded(gen_thread_count + 5);
        for _ in 0..oi_read_thread_count {
            tracker.spawn(Self::io_read_work(
                recv_read_io.clone(),
                send_chunk.clone(),
                level.clone(),
            ));
        }
        for i in 0..gen_thread_count {
            let recv_gen = recv_gen.clone();
            let send_chunk = send_chunk.clone();
            let level = level.clone();
            let builder = thread::Builder::new().name(format!("Generation Thread {i}"));
            thread_tracker.push(
                builder
                    .spawn(move || {
                        Self::generation_work(recv_gen, send_chunk, level);
                    })
                    .unwrap(),
            );
        }

        tracker.spawn(Self::io_write_work(recv_write_io, level.clone()));

        let builder = thread::Builder::new().name("Schedule Thread".to_string());
        thread_tracker.push(
            builder
                .spawn(move || {
                    Self {
                        queue: Vec::new(),
                        last_level: ChunkLevel::default(),
                        send_level: level_channel,
                        loaded_chunks: level.loaded_chunks.clone(),
                        proto_chunks: HashMapType::default(),
                        unload_chunks: HashMapType::default(),
                        occupied: HashSetType::default(),
                        task_mark: HashMapType::default(),
                        running_task_count: 0,
                        recv_chunk,
                        io_read: send_read_io,
                        io_write: send_write_io,
                        generate: send_gen,
                        listener,
                    }
                        .work(level);
                })
                .unwrap(),
        )
    }

    fn get_chunk(
        loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
        proto_chunks: &mut HashMapType<ChunkPos, ProtoChunk>,
        pos: ChunkPos,
    ) -> Option<Chunk> {
        if let Some(data) = loaded_chunks.get(&pos) {
            Some(Chunk::Level(data.clone()))
        } else {
            proto_chunks.remove(&pos).map(Chunk::Proto)
        }
    }

    fn remove_chunk(
        loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
        proto_chunks: &mut HashMapType<ChunkPos, ProtoChunk>,
        pos: ChunkPos,
    ) -> Option<Chunk> {
        if let Some(data) = loaded_chunks.remove(&pos) {
            Some(Chunk::Level(data.1))
        } else {
            proto_chunks.remove(&pos).map(Chunk::Proto)
        }
    }

    fn get_chunk_stage_id(
        loaded_chunks: &Arc<DashMap<Vector2<i32>, SyncChunk>>,
        proto_chunks: &HashMapType<ChunkPos, ProtoChunk>,
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
        proto_chunks: &mut HashMapType<ChunkPos, ProtoChunk>,
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
        for pos in self.last_level.keys() {
            if !new_level.contains_key(pos)
                && let Some(chunk) =
                Self::remove_chunk(&self.loaded_chunks, &mut self.proto_chunks, *pos)
            {
                log::debug!("unload chunk {pos:?}");
                self.unload_chunks.insert(*pos, chunk);
            }
        }
        for (pos, level) in &new_level {
            let old_level = *self.last_level.get(pos).unwrap_or(&ChunkLoading::MAX_LEVEL);
            if old_level == ChunkLoading::MAX_LEVEL
                && let Some(chunk) = self.unload_chunks.remove(pos)
            {
                Self::add_chunk(&self.loaded_chunks, &mut self.proto_chunks, *pos, chunk);
            }

            if old_level != *level {
                match self.task_mark.entry(*pos) {
                    Entry::Occupied(mut entry) => {
                        let (mark, stage) = entry.get_mut();
                        if stage == &(Full as u8) {
                            continue;
                        }
                        let next_stage =
                            (StagedChunkEnum::level_to_stage(old_level) as u8).max(*stage);
                        let new_highest_stage = StagedChunkEnum::level_to_stage(*level) as u8;
                        if next_stage >= new_highest_stage {
                            continue;
                        }
                        for i in (next_stage + 1)..=new_highest_stage {
                            if (*mark >> i & 1) == 0 {
                                // no task before
                                self.queue.push((*pos, i8::MAX, i.into()));
                                *mark |= 1 << i;
                            }
                        }
                    }
                    Entry::Vacant(entry) => {
                        let mut mark = 0;
                        let new_highest_stage = StagedChunkEnum::level_to_stage(*level) as u8;
                        for i in 1..=new_highest_stage {
                            self.queue.push((*pos, i8::MAX, i.into()));
                            mark |= 1 << i;
                        }
                        entry.insert((mark, 0));
                    }
                };
            }
        }
        for (pos, level, _) in self.queue.iter_mut() {
            *level = *new_level.get(pos).unwrap_or(&ChunkLoading::MAX_LEVEL);
        }
        self.queue
            .sort_unstable_by(|(_l_pos, l_level, l_stage), (_r_pos, r_level, r_stage)| {
                if l_level != r_level {
                    l_level.cmp(r_level)
                } else {
                    l_stage.cmp(r_stage)
                }
            });
        self.last_level = new_level;
    }

    async fn io_read_work(
        recv: Receiver<ChunkPos>,
        send: Sender<(ChunkPos, RecvChunk)>,
        level: Arc<Level>,
    ) {
        log::info!("io read thread start");
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(level.world_gen.random_config.seed);
        let (t_send, mut t_recv) = tokio::sync::mpsc::channel(2);
        while let Ok(pos) = task::block_in_place(|| recv.recv()) {
            debug!("io read thread receive chunk pos {pos:?}");
            level
                .chunk_saver
                .fetch_chunks(&level.level_folder, &[pos], t_send.clone())
                .await;
            if let Some(Loaded(chunk)) = t_recv.recv().await {
                if send
                    .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                    .is_err()
                {
                    break;
                }
            } else if send
                .send((
                    pos,
                    RecvChunk::IO(Proto(ProtoChunk::new(
                        pos,
                        gen_settings_from_dimension(&level.world_gen.dimension),
                        level.world_gen.default_block,
                        biome_mixer_seed,
                    ))),
                ))
                .is_err()
            {
                break;
            }
        }
        log::info!("io read thread stop");
    }

    async fn io_write_work(recv: Receiver<Vec<(ChunkPos, SyncChunk)>>, level: Arc<Level>) {
        log::info!(
            "io write thread start",
        );
        while let Ok(data) = task::block_in_place(|| recv.recv()) {
            debug!("io write thread receive chunks size {}", data.len());
            level
                .chunk_saver
                .save_chunks(&level.level_folder, data)
                .await
                .unwrap();
        }
        log::info!(
            "io write thread stop id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
    }

    fn generation_work(
        recv: Receiver<(ChunkPos, Cache, StagedChunkEnum)>,
        send: Sender<(ChunkPos, RecvChunk)>,
        level: Arc<Level>,
    ) {
        log::info!(
            "generation thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );

        let settings = gen_settings_from_dimension(&level.world_gen.dimension);
        while let Ok((pos, mut cache, stage)) = recv.recv() {
            debug!("generation thread receive chunk pos {pos:?} to stage {stage:?}");
            cache.advance(
                stage,
                level.block_registry.as_ref(),
                settings,
                &level.world_gen.random_config,
                &level.world_gen.terrain_cache,
                &level.world_gen.base_router,
                level.world_gen.dimension,
            );
            if send.send((pos, RecvChunk::Generation(cache))).is_err() {
                break;
            }
        }
        log::info!(
            "generation thread stop id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
    }

    fn drop_mark(&mut self, stage: StagedChunkEnum, pos: ChunkPos) {
        match self.task_mark.entry(pos) {
            Entry::Occupied(mut entry) => {
                let (mark, _) = entry.get_mut();
                debug_assert!((*mark >> (stage as u8) & 1) == 1);
                *mark -= 1 << (stage as u8);
                if *mark == 0 && !self.last_level.contains_key(&pos) {
                    entry.remove();
                }
            }
            Entry::Vacant(_) => panic!(),
        }
    }

    fn unload_chunk(&mut self) {
        let mut unload_chunks = HashMapType::default();
        swap(&mut unload_chunks, &mut self.unload_chunks);
        let mut chunks = Vec::with_capacity(unload_chunks.len());
        for (pos, data) in unload_chunks {
            match data {
                Chunk::Level(chunk) => {
                    if Arc::strong_count(&chunk) != 1 { // todo investigate whether check by ref count is safe.
                        log::warn!(
                            "chunk {pos:?} is still used somewhere. it can't be unloaded"
                        );
                        self.unload_chunks.insert(pos, Chunk::Level(chunk));
                    } else {
                        chunks.push((pos, chunk));
                    }
                }
                Chunk::Proto(chunk) => {
                    log::warn!("proto chunk saving is unimplemented {pos:?}");
                    self.unload_chunks.insert(pos, Proto(chunk));
                }
            }
        }
        log::debug!("send {} unloaded chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        self.io_write.send(chunks).expect("io write thread stop");
    }

    fn save_all_chunk(&self) {
        let mut chunks = Vec::with_capacity(self.unload_chunks.len() + self.proto_chunks.len() + self.loaded_chunks.len());
        for (pos, chunk) in &self.unload_chunks {
            match chunk {
                Chunk::Level(chunk) => {
                    chunks.push((*pos, chunk.clone()));
                }
                Chunk::Proto(_chunk) => {
                    log::warn!("proto chunk saving is unimplemented {pos:?}");
                }
            }
        }
        for (pos, _chunk) in &self.proto_chunks {
            log::warn!("proto chunk saving is unimplemented {pos:?}");
        }
        for i in self.loaded_chunks.iter() {
            chunks.push((*i.key(), i.value().clone()));
        }
        log::debug!("send {} chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        self.io_write.send(chunks).expect("io write thread stop");
    }

    fn receive_chunk(&mut self, pos: ChunkPos, data: RecvChunk) {
        debug!("receive chunk pos {pos:?}");
        match data {
            RecvChunk::IO(chunk) => match chunk {
                Chunk::Level(data) => {
                    match self.task_mark.entry(pos) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().1 = Full as u8;
                        }
                        Entry::Vacant(_) => panic!(),
                    }
                    if self.last_level.contains_key(&pos) {
                        self.loaded_chunks.insert(pos, data.clone());
                    } else {
                        log::debug!("receive chunk {pos:?} to unload chunks");
                        self.unload_chunks.insert(pos, Chunk::Level(data.clone()));
                    }
                    self.listener.process_new_chunk(pos, &data);
                }
                Chunk::Proto(data) => {
                    match self.task_mark.entry(pos) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().1 = data.stage_id();
                        }
                        Entry::Vacant(_) => panic!(),
                    }
                    if self.last_level.contains_key(&pos) {
                        self.proto_chunks.insert(pos, data);
                    } else {
                        log::debug!("receive chunk {pos:?} to unload chunks");
                        self.unload_chunks.insert(pos, Chunk::Proto(data));
                    }
                    // I think we don't need to remove mark here
                }
            },
            RecvChunk::Generation(data) => {
                let mut dx = 0;
                let mut dy = 0;
                for chunk in data.chunks {
                    let new_pos = ChunkPos::new(data.x + dx, data.y + dy);
                    match chunk {
                        Chunk::Level(chunk) => {
                            if new_pos == pos {
                                // other chunk is borrowed by arc. don't need to return
                                match self.task_mark.entry(pos) {
                                    Entry::Occupied(mut entry) => {
                                        entry.get_mut().1 = Full as u8;
                                    }
                                    Entry::Vacant(_) => panic!(),
                                }
                                if self.last_level.contains_key(&new_pos) {
                                    self.loaded_chunks.insert(new_pos, chunk.clone());
                                } else {
                                    log::debug!("receive chunk {new_pos:?} to unload chunks");
                                    self.unload_chunks.insert(new_pos, Chunk::Level(chunk.clone()));
                                }
                                self.listener.process_new_chunk(new_pos, &chunk);
                            }
                        }
                        Chunk::Proto(chunk) => {
                            if new_pos == pos {
                                match self.task_mark.entry(pos) {
                                    Entry::Occupied(mut entry) => {
                                        entry.get_mut().1 = chunk.stage_id();
                                    }
                                    Entry::Vacant(_) => panic!(),
                                }
                            }
                            if self.last_level.contains_key(&new_pos) {
                                self.proto_chunks.insert(new_pos, chunk);
                            } else {
                                log::debug!("receive chunk {new_pos:?} to unload chunks");
                                self.unload_chunks.insert(new_pos, Chunk::Proto(chunk));
                            }
                        }
                    }
                    self.occupied.remove(&new_pos);
                    dy += 1;
                    if dy == data.size {
                        dy = 0;
                        dx += 1;
                    }
                }
            }
        }
        self.running_task_count -= 1;
    }

    fn dump_debug_info(&self, sx: i32, tx: i32, sy: i32, ty: i32) {
        debug!("queue len {}", self.queue.len());
        debug!("proto chunk size {}", self.proto_chunks.len());
        debug!("unload chunk size {}", self.unload_chunks.len());
        // debug!("queue {:?}", self.queue);
        debug!("running tasks {}", self.running_task_count);
        debug!(
            "global listener count {}",
            self.listener.global.lock().unwrap().len()
        );
        debug!(
            "single listener count {}",
            self.listener.single.lock().unwrap().len()
        );
        let mut s = String::new();
        for x in sx..=tx {
            for y in sy..=ty {
                s += Self::get_chunk_stage_id(
                    &self.loaded_chunks,
                    &self.proto_chunks,
                    ChunkPos::new(x, y),
                )
                    .to_string()
                    .as_str();
                s += " ";
            }
            s += "\n";
        }
        debug!("chunk stage:\n{s}\n");

        ChunkLoading::dump_level_debug(&self.last_level, sx, tx, sy, ty);
    }

    fn work(mut self, level: Arc<Level>) {
        log::info!(
            "schedule thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
        if let Some(new_level) = self.send_level.wait_and_get(&level) {
            debug!("receive new level");
            self.resort_work(new_level);
        }
        let mut clock = Instant::now();
        loop {
            if level.should_unload.load(Relaxed) {
                log::debug!("unload chunk signal");
                self.unload_chunk();
                level.should_unload.store(false, Relaxed);
            }
            if level.should_save.load(Relaxed) {
                log::debug!("save all chunk signal");
                self.save_all_chunk();
                level.should_save.store(false, Relaxed);
            }
            if level.shut_down_chunk_system.load(Relaxed) {
                log::debug!("shut down signal");
                break;
            }
            let mut nothing = true;
            let mut len = self.queue.len();
            let mut i = 0;

            let now = Instant::now();
            if now - clock > Duration::from_secs(5) {
                self.dump_debug_info(-20, 20, -20, 20);
                clock = now;
            }
            'outer: while i < len {
                let mut have_recv = false;
                while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                    self.receive_chunk(pos, data);
                    have_recv = true;
                }
                if have_recv {
                    nothing = false;
                    break 'outer;
                }

                let (pos, _, stage) = self.queue[i];

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

                let (_, current_stage) = self.task_mark.get(&pos).unwrap(); // unwrap because we have checked MAX_LEVEL
                if *current_stage >= (stage as u8) {
                    self.drop_mark(stage, pos);
                    self.queue.remove(i);
                    len -= 1;
                    continue;
                }

                if stage == Empty {
                    nothing = false;
                    self.running_task_count += 1;
                    self.io_read.send(pos).expect("oi thread close unexpectedly");
                    self.queue.remove(i);
                    len -= 1;
                    continue;
                }

                let radius = stage.get_radius();
                let write_radius = stage.get_write_radius();
                let depend = stage.get_dependencies();
                for dx in -radius..=radius {
                    for dy in -radius..=radius {
                        let new_pos = pos.add_raw(dx, dy);
                        let dst = max(abs(dx), abs(dy)) as usize;
                        if self.task_mark.get(&new_pos).unwrap_or(&(0, 0)).1 < (depend[dst] as u8) {
                            i += 1;
                            continue 'outer;
                        }
                    }
                }
                for dx in -write_radius..=write_radius {
                    for dy in -write_radius..=write_radius {
                        let new_pos = pos.add_raw(dx, dy);
                        if self.occupied.contains(&new_pos) {
                            i += 1;
                            continue 'outer;
                        }
                    }
                }
                let mut cache = Cache::new(
                    pos.x - write_radius,
                    pos.y - write_radius,
                    (write_radius << 1) + 1,
                );
                for dx in -write_radius..=write_radius {
                    for dy in -write_radius..=write_radius {
                        let new_pos = pos.add_raw(dx, dy);
                        cache.chunks.push(
                            Self::get_chunk(&self.loaded_chunks, &mut self.proto_chunks, new_pos)
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
                debug!("the queue is empty. thread sleep");
                'out: while self.running_task_count > 0 {
                    let (pos, data) = self.recv_chunk.recv().expect("recv_chunk stop");
                    self.receive_chunk(pos, data);
                    if let Some(new_level) = self.send_level.get() {
                        debug!("receive new level");
                        self.resort_work(new_level);
                        break 'out;
                    }
                }
                if let Some(new_level) = self.send_level.wait_and_get(&level) {
                    debug!("receive new level");
                    self.resort_work(new_level);
                }
            } else if let Some(new_level) = self.send_level.get() {
                debug!("receive new level");
                self.resort_work(new_level);
            } else if nothing && self.running_task_count > 0 {
                debug!("nothing to do. thread sleep.");
                if let Ok((pos, data)) = self.recv_chunk.recv() {
                    self.receive_chunk(pos, data);
                }
            }
        }
        log::info!("waiting all generation task finished");
        while self.running_task_count > 0 {
            let (pos, data) = self.recv_chunk.recv().expect("recv_chunk stop");
            self.receive_chunk(pos, data);
        }
        self.save_all_chunk();
    }
}
