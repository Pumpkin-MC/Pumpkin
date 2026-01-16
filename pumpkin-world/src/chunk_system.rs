/*
TODO
1. add proto chunk dirty flag
2. better priority
5. add lifetime to loading ticket
6. solve entity not unload problem
*/

use crate::block::RawBlockState;
use crate::chunk::io::LoadedData::Loaded;
use crate::chunk::{ChunkData, ChunkHeightmapType, ChunkLight, ChunkSections, SubChunk};
use pumpkin_data::dimension::Dimension;
use std::default::Default;
use std::pin::Pin;

use crate::generation::height_limit::HeightLimitView;

use crate::generation::proto_chunk::{GenerationCache, TerrainCache};
use crate::generation::settings::{GenerationSettings, gen_settings_from_dimension};
use crate::level::{Level, SyncChunk};
use crate::world::{BlockAccessor, BlockRegistryExt};
use crate::{BlockStateId, GlobalRandomConfig, ProtoChunk, ProtoNoiseRouters};
use crossbeam::channel::{Receiver, Sender};
use dashmap::DashMap;
use itertools::Itertools;
use log::{debug, error};
use num_traits::abs;
use pumpkin_data::biome::Biome;

use pumpkin_data::fluid::{Fluid, FluidState};
use pumpkin_data::{Block, BlockState};
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use std::cmp::{Ordering, PartialEq, max, min};
use std::collections::hash_map::Entry;
use std::collections::{BinaryHeap, HashMap};
use std::mem::swap;
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex};

use crate::chunk::format::LightContainer;
use crate::chunk::io::LoadedData;
use crate::chunk_system::Chunk::Proto;
use crate::chunk_system::StagedChunkEnum::{Biomes, Empty, Features, Full, Noise, Surface};
use crossfire::AsyncRx;
use pumpkin_data::chunk::ChunkStatus;
use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::{Key, SlotMap, new_key_type};
use std::sync::atomic::Ordering::{Relaxed, SeqCst};
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::{RwLock, oneshot};

type HashMapType<K, V> = FxHashMap<K, V>;
type HashSetType<K> = FxHashSet<K>;
type ChunkPos = Vector2<i32>;
type ChunkLevel = HashMapType<ChunkPos, i8>;
type IOLock = Arc<(Mutex<HashMapType<ChunkPos, u8>>, Condvar)>;

pub struct HeapNode(i8, ChunkPos);
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

struct LevelCache(i32, i32, usize, [(i8, i8); 256 * 256]);

impl LevelCache {
    fn new() -> Self {
        Self(0, 0, 0, [(0, 0); 256 * 256])
    }
}
impl LevelCache {
    fn clean(&mut self, pos: ChunkPos, level: i8) {
        let dst = ChunkLoading::MAX_LEVEL - level + 1;
        self.0 = pos.x - dst as i32;
        self.1 = pos.y - dst as i32;
        self.2 = (dst as usize) << 1 | 1;
        self.3[..self.2 * self.2].fill((-127, -127));
    }
    fn get(&mut self, map: &ChunkLevel, pos: ChunkPos) -> i8 {
        let dx = (pos.x - self.0) as usize;
        let dy = (pos.y - self.1) as usize;
        debug_assert!(pos.x >= self.0 && pos.y >= self.1);
        let value = &mut self.3[dx * self.2 + dy];
        if value.0 == -127 {
            value.0 = *map.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL);
            value.1 = value.0;
        }
        value.1
    }
    fn set(&mut self, map: &ChunkLevel, pos: ChunkPos, level: i8) {
        let dx = (pos.x - self.0) as usize;
        let dy = (pos.y - self.1) as usize;
        debug_assert!(pos.x >= self.0 && pos.y >= self.1);
        let value = &mut self.3[dx * self.2 + dy];
        if value.0 == -127 {
            value.0 = *map.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL);
        }
        value.1 = level;
    }
    fn write(
        &mut self,
        map: &mut ChunkLevel,
        change: &mut HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
    ) {
        for i in 0..self.2 {
            for j in 0..self.2 {
                let value = self.3[i * self.2 + j];
                if value.0 != value.1 {
                    let pos = ChunkPos::new(i as i32 + self.0, j as i32 + self.1);
                    if value.1 == ChunkLoading::MAX_LEVEL {
                        map.remove(&pos);
                    } else {
                        map.insert(pos, value.1);
                    }
                    let value = (
                        StagedChunkEnum::level_to_stage(value.0),
                        StagedChunkEnum::level_to_stage(value.1),
                    );
                    if value.0 == value.1 {
                        continue;
                    }
                    match change.entry(pos) {
                        Entry::Occupied(mut entry) => {
                            let i = entry.get_mut();
                            debug_assert!(i.1 == value.0);
                            if i.0 == value.1 {
                                entry.remove();
                            } else {
                                i.1 = value.1;
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(value);
                        }
                    }
                }
            }
        }
    }
}

pub struct ChunkLoading {
    pub is_priority_dirty: bool,
    pub pos_level: ChunkLevel,
    change: HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
    pub ticket: HashMapType<ChunkPos, Vec<i8>>, // TODO lifetime & id
    pub high_priority: Vec<ChunkPos>,
    pub sender: Arc<LevelChannel>,
    pub increase_update: BinaryHeap<HeapNode>,
    pub decrease_update: BinaryHeap<HeapNode>,
    cache: LevelCache,
}

#[test]
fn test() {
    let mut a = ChunkLoading::new(Arc::new(LevelChannel::new()));

    a.add_ticket((0, 0).into(), 44);
    a.add_ticket((0, 1).into(), 44);
    a.remove_ticket((0, 0).into(), 44);

    a.add_ticket((0, 0).into(), 30);
    a.add_ticket((0, 10).into(), 25);
    a.add_ticket((10, 10).into(), 26);
    a.add_ticket((10, 10).into(), 26);
    a.remove_ticket((0, 0).into(), 30);
    a.remove_ticket((0, 10).into(), 25);
    a.remove_ticket((10, 10).into(), 26);
    a.remove_ticket((10, 10).into(), 26);
    a.add_ticket((-72, 457).into(), 24);
    a.add_ticket((-72, 455).into(), 33);
    a.add_ticket((-72, 456).into(), 24);
    a.remove_ticket((-72, 457).into(), 24);
    a.add_ticket((-72, 455).into(), 24);

    a.add_ticket((-59, 495).into(), 33);
    a.add_ticket((-51, 504).into(), 24);

    a.remove_ticket((-51, 504).into(), 24);

    let sx = -59;
    let tx = -51;
    let sy = 495;
    let ty = 504;
    {
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
                                a.pos_level
                                    .get(&ChunkPos::new(x, y))
                                    .unwrap_or(&ChunkLoading::MAX_LEVEL)
                            )
                        })
                        .collect::<String>(),
                );
                row
            })
            .collect::<Vec<_>>()
            .join("\n");

        println!("\nloading level:\n{header}\n{grid}");
    }
}

impl ChunkLoading {
    // pub const FULL_CHUNK_LEVEL: i8 = 33;
    pub const FULL_CHUNK_LEVEL: i8 = 43;
    pub const MAX_LEVEL: i8 = 46; // level 46 will be unloaded.
    fn debug_check_error(&self) -> bool {
        let mut temp = ChunkLevel::default();
        for (ticket_pos, levels) in &self.ticket {
            let level = *levels.iter().min().unwrap();
            let range = Self::MAX_LEVEL - level - 1;
            for dx in -range..=range {
                for dy in -range..=range {
                    let new_pos = ticket_pos.add_raw(dx as i32, dy as i32);
                    let level_from_source = level + abs(dx).max(abs(dy));
                    let i = temp.entry(new_pos).or_insert(Self::MAX_LEVEL);
                    *i = min(*i, level_from_source);
                }
            }
        }
        if temp.len() != self.pos_level.len() {
            debug!("temp: \n{temp:?}");
            debug!("pos_level: \n{:?}", self.pos_level);
        }
        assert_eq!(temp.len(), self.pos_level.len());
        for val in &temp {
            if val != self.pos_level.get_key_value(val.0).unwrap() {
                Self::dump_level_debug(
                    &self.high_priority,
                    &self.pos_level,
                    val.0.x - 40,
                    val.0.x + 40,
                    val.0.y - 40,
                    val.0.y + 40,
                );
            }
            assert_eq!(val, self.pos_level.get_key_value(val.0).unwrap());
        }
        true
    }
    pub fn dump_level_debug(
        pri: &Vec<ChunkPos>,
        map: &ChunkLevel,
        sx: i32,
        tx: i32,
        sy: i32,
        ty: i32,
    ) {
        debug!("high_priority {pri:?}");

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

    #[inline]
    pub const fn get_level_from_view_distance(view_distance: u8) -> i8 {
        Self::FULL_CHUNK_LEVEL - (view_distance as i8)
    }

    pub fn new(sender: Arc<LevelChannel>) -> Self {
        Self {
            is_priority_dirty: true,
            pos_level: ChunkLevel::default(),
            change: HashMapType::default(),
            ticket: HashMapType::default(),
            high_priority: Vec::new(),
            sender,
            increase_update: BinaryHeap::default(),
            decrease_update: BinaryHeap::default(),
            cache: LevelCache::new(),
        }
    }

    pub fn send_change(&mut self) {
        // debug!("sending change");
        if !self.change.is_empty() {
            let mut tmp = HashMapType::default();
            swap(&mut tmp, &mut self.change);
            if self.is_priority_dirty {
                self.is_priority_dirty = false;
                self.sender
                    .set_both((tmp, self.pos_level.clone()), self.high_priority.clone());
            } else {
                self.sender.set_level((tmp, self.pos_level.clone()));
            }
        }
        if self.is_priority_dirty {
            self.is_priority_dirty = false;
            self.sender.set_priority(self.high_priority.clone());
        }
    }

    fn run_increase_update(&mut self) {
        while let Some(node) = self.increase_update.pop() {
            let (pos, level) = node.into();
            debug_assert!(level < Self::MAX_LEVEL);
            if level > self.cache.get(&self.pos_level, pos) {
                continue;
            }
            debug_assert_eq!(level, self.cache.get(&self.pos_level, pos));
            let spread_level = level + 1;
            if spread_level >= Self::MAX_LEVEL {
                continue;
            }
            for dx in -1..2 {
                for dy in -1..2 {
                    let new_pos = pos.add_raw(dx, dy);
                    if new_pos != pos {
                        self.check_then_push(new_pos, spread_level);
                    }
                }
            }
        }
    }

    fn check_then_push(&mut self, pos: ChunkPos, level: i8) {
        debug_assert!(level < Self::MAX_LEVEL);
        let old = self.cache.get(&self.pos_level, pos);
        if old <= level {
            return;
        }
        self.cache.set(&self.pos_level, pos, level);
        self.increase_update.push((pos, level).into());
    }

    fn run_decrease_update(&mut self, pos: ChunkPos, range: i32) {
        while let Some(node) = self.decrease_update.pop() {
            let (pos, level) = node.into();
            debug_assert!(level < Self::MAX_LEVEL);
            let spread_level = level + 1;
            for dx in -1..2 {
                for dy in -1..2 {
                    let new_pos = pos.add_raw(dx, dy);
                    if new_pos == pos {
                        continue;
                    }
                    let new_pos_level = self.cache.get(&self.pos_level, new_pos);
                    if new_pos_level == Self::MAX_LEVEL {
                        continue;
                    }
                    debug_assert!(new_pos_level <= spread_level);
                    if new_pos_level == spread_level {
                        self.cache.set(&self.pos_level, new_pos, Self::MAX_LEVEL);
                        if spread_level < Self::MAX_LEVEL {
                            self.decrease_update.push((new_pos, spread_level).into());
                        }
                    } else {
                        self.increase_update.push((new_pos, new_pos_level).into());
                    }
                }
            }
        }

        for (ticket_pos, levels) in &self.ticket {
            if abs(ticket_pos.x - pos.x) <= range && abs(ticket_pos.y - pos.y) <= range {
                let level = *levels.iter().min().unwrap();
                debug_assert!(level < Self::MAX_LEVEL);
                let old = self.cache.get(&self.pos_level, *ticket_pos);
                if old <= level {
                    continue;
                }
                self.cache.set(&self.pos_level, *ticket_pos, level);
                self.increase_update.push((*ticket_pos, level).into());
            }
        }
        self.run_increase_update();
    }

    pub fn add_force_ticket(&mut self, pos: ChunkPos) {
        // log::debug!("add force ticket at {pos:?}");
        self.high_priority.push(pos);
        self.is_priority_dirty = true;
        self.add_ticket(pos, ChunkLoading::FULL_CHUNK_LEVEL);
    }
    pub fn remove_force_ticket(&mut self, pos: ChunkPos) {
        // log::debug!("remove force ticket at {pos:?}");
        let index = self
            .high_priority
            .iter()
            .find_position(|x| **x == pos)
            .unwrap()
            .0;
        self.high_priority.remove(index);
        self.is_priority_dirty = true;
        self.remove_ticket(pos, ChunkLoading::FULL_CHUNK_LEVEL);
    }
    pub fn add_ticket(&mut self, pos: ChunkPos, level: i8) {
        // log::debug!("add ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        match self.ticket.entry(pos) {
            Entry::Occupied(mut vec) => {
                vec.get_mut().push(level);
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![level]);
            }
        }

        let old = *self.pos_level.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL);
        if old <= level {
            return;
        }
        self.cache.clean(pos, level);
        self.cache.set(&self.pos_level, pos, level);

        debug_assert!(self.increase_update.is_empty());
        self.increase_update.push((pos, level).into());
        self.run_increase_update();
        self.cache.write(&mut self.pos_level, &mut self.change);
        debug_assert!(self.debug_check_error());
    }
    pub fn remove_ticket(&mut self, pos: ChunkPos, level: i8) {
        // log::debug!("remove ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        let Some(vec) = self.ticket.get_mut(&pos) else {
            // log::warn!("No ticket found at {pos:?}");
            return;
        };
        let Some((index, _)) = vec.iter().find_position(|x| **x == level) else {
            // log::warn!("No ticket found at {pos:?}");
            return;
        };
        vec.remove(index);
        match self.pos_level.entry(pos) {
            Entry::Occupied(entry) => {
                let old_level = *entry.get();
                let source = *vec.iter().min().unwrap_or(&Self::MAX_LEVEL);
                if vec.is_empty() {
                    self.ticket.remove(&pos);
                }
                if level == old_level && source != level {
                    self.cache.clean(pos, old_level);
                    self.cache.set(&self.pos_level, pos, Self::MAX_LEVEL);
                    debug_assert!(self.decrease_update.is_empty());
                    self.decrease_update.push((pos, level).into());
                    self.run_decrease_update(pos, (Self::MAX_LEVEL - level - 1) as i32);
                    self.cache.write(&mut self.pos_level, &mut self.change);
                }
            }
            Entry::Vacant(_) => panic!(),
        }
        debug_assert!(self.debug_check_error());
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
    StructureStart,
    StructureReferences,
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
            3 => Self::StructureStart,
            4 => Self::StructureReferences,
            5 => Self::Noise,
            6 => Self::Surface,
            7 => Self::Features,
            8 => Self::Full,
            _ => panic!(),
        }
    }
}

impl From<ChunkStatus> for StagedChunkEnum {
    fn from(status: ChunkStatus) -> Self {
        match status {
            ChunkStatus::Empty => Empty,
            ChunkStatus::StructureStarts => StagedChunkEnum::StructureStart,
            ChunkStatus::StructureReferences => StagedChunkEnum::StructureReferences,
            ChunkStatus::Biomes => Biomes,
            ChunkStatus::Noise => Noise,
            ChunkStatus::Surface => Surface,
            ChunkStatus::Carvers => Surface,
            ChunkStatus::Features => Features,
            ChunkStatus::InitializeLight => Features,
            ChunkStatus::Light => Features,
            ChunkStatus::Spawn => Features,
            ChunkStatus::Full => Full,
        }
    }
}

impl From<StagedChunkEnum> for ChunkStatus {
    fn from(status: StagedChunkEnum) -> Self {
        match status {
            StagedChunkEnum::Empty => ChunkStatus::Empty,
            StagedChunkEnum::StructureStart => ChunkStatus::StructureStarts,
            StagedChunkEnum::StructureReferences => ChunkStatus::StructureReferences,
            StagedChunkEnum::Biomes => ChunkStatus::Biomes,
            StagedChunkEnum::Noise => ChunkStatus::Noise,
            StagedChunkEnum::Surface => ChunkStatus::Surface,
            StagedChunkEnum::Features => ChunkStatus::Features,
            StagedChunkEnum::Full => ChunkStatus::Full,
            _ => panic!(),
        }
    }
}

impl StagedChunkEnum {
    const fn level_to_stage(level: i8) -> Self {
        if level <= 43 {
            Full
        } else if level <= 44 {
            Features
        } else if level <= 45 {
            Surface
        } else {
            Self::None
        }
    }
    const FULL_DEPENDENCIES: &'static [Self] = &[Full, Features, Surface];
    const FULL_RADIUS: i32 = 2;
    const fn get_direct_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Features => 1,
            Self::Full => 1,
            _ => panic!(),
        }
    }
    const fn get_write_radius(self) -> i32 {
        // self exclude
        match self {
            Self::Empty => 0,
            Self::StructureStart => 0,
            Self::StructureReferences => 0,
            Self::Biomes => 0,
            Self::Noise => 0,
            Self::Surface => 0,
            Self::Features => 1,
            Self::Full => 0,
            _ => panic!(),
        }
    }
    const fn get_direct_dependencies(self) -> &'static [StagedChunkEnum] {
        match self {
            // In vanilla StructureStart is first, but since it needs the biome in Vanilla it gets computed in StructureStart and
            // the Biome Step, this should be more efficient
            Self::Biomes => &[Self::Empty],
            Self::StructureStart => &[Self::Biomes],
            Self::StructureReferences => &[Self::StructureStart],
            Self::Noise => &[Self::StructureReferences],
            Self::Surface => &[Self::Noise],
            Self::Features => &[Self::Surface, Self::Surface],
            Self::Full => &[Self::Features, Self::Features],
            _ => panic!(),
        }
    }
}

type LevelChange = (
    HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
    ChunkLevel,
);

pub struct LevelChannel {
    pub value: Mutex<(Option<LevelChange>, Option<Vec<ChunkPos>>)>,
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
            value: Mutex::new((None, None)),
            notify: Condvar::new(),
        }
    }
    pub fn set_both(
        &self,
        new_value: (
            HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
            ChunkLevel,
        ),
        pos: Vec<ChunkPos>,
    ) {
        // debug!("set new level and priority");
        let mut value = self.value.lock().unwrap();
        value.1 = Some(pos);
        if let Some(old) = &mut value.0 {
            for (pos, change) in new_value.0 {
                match old.0.entry(pos) {
                    Entry::Occupied(mut entry) => {
                        let tmp = entry.get_mut();
                        debug_assert_eq!(tmp.1, change.0);
                        if tmp.0 == change.1 {
                            entry.remove();
                        } else {
                            tmp.1 = change.1;
                        }
                    }
                    Entry::Vacant(entry) => {
                        debug_assert_ne!(change.0, change.1);
                        entry.insert(change);
                    }
                }
            }
            old.1 = new_value.1;
        } else {
            value.0 = Some(new_value);
        }
        self.notify.notify_one();
    }
    pub fn set_level(
        &self,
        new_value: (
            HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
            ChunkLevel,
        ),
    ) {
        // debug!("set new level");
        let mut value = self.value.lock().unwrap();
        if let Some(old) = &mut value.0 {
            for (pos, change) in new_value.0 {
                match old.0.entry(pos) {
                    Entry::Occupied(mut entry) => {
                        let tmp = entry.get_mut();
                        debug_assert_eq!(tmp.1, change.0);
                        if tmp.0 == change.1 {
                            entry.remove();
                        } else {
                            tmp.1 = change.1;
                        }
                    }
                    Entry::Vacant(entry) => {
                        debug_assert_ne!(change.0, change.1);
                        entry.insert(change);
                    }
                }
            }
            old.1 = new_value.1;
        } else {
            value.0 = Some(new_value);
        }
        self.notify.notify_one();
    }
    pub fn set_priority(&self, pos: Vec<ChunkPos>) {
        // debug!("set new priority");
        self.value.lock().unwrap().1 = Some(pos);
        self.notify.notify_one();
    }
    pub fn get(&self) -> (Option<LevelChange>, Option<Vec<ChunkPos>>) {
        let mut lock = self.value.lock().unwrap();
        let mut ret = (None, None);
        swap(&mut ret, &mut *lock);
        ret
    }
    pub fn wait_and_get(&self, level: &Arc<Level>) -> (Option<LevelChange>, Option<Vec<ChunkPos>>) {
        let mut lock = self.value.lock().unwrap();
        while lock.0.is_none()
            && lock.1.is_none()
            && !level.should_unload.load(SeqCst)
            && !level.should_save.load(SeqCst)
            && !level.shut_down_chunk_system.load(SeqCst)
        {
            lock = self.notify.wait(lock).unwrap();
        }
        let mut ret = (None, None);
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
    Proto(Box<ProtoChunk>),
}

impl Chunk {
    fn get_stage_id(&self) -> u8 {
        match self {
            Chunk::Proto(data) => data.stage_id(),
            Chunk::Level(_) => 8,
        }
    }
    fn get_proto_chunk_mut(&mut self) -> &mut ProtoChunk {
        match self {
            Chunk::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Proto(chunk) => chunk,
        }
    }
    fn get_proto_chunk(&self) -> &ProtoChunk {
        match self {
            Chunk::Level(_) => panic!("chunk isn't a ProtoChunk"),
            Proto(chunk) => chunk,
        }
    }
    fn upgrade_to_level_chunk(&mut self, dimension: &Dimension) {
        let proto_chunk = self.get_proto_chunk();

        let total_sections = dimension.height as usize / 16;
        let mut sections = ChunkSections::new(
            vec![SubChunk::default(); total_sections].into_boxed_slice(),
            dimension.min_y,
        );

        let proto_biome_height = (dimension.height / 4) as usize;
        let biome_min_y = dimension.min_y / 4;

        for y_offset in 0..proto_biome_height {
            let section_index = y_offset / 4;
            let relative_biome_y = y_offset % 4;

            if let Some(section) = sections.sections.get_mut(section_index) {
                let absolute_biome_y = biome_min_y + y_offset as i32;

                for z in 0..4 {
                    for x in 0..4 {
                        let biome = proto_chunk.get_biome(x as i32, absolute_biome_y, z as i32);
                        section.biomes.set(x, relative_biome_y, z, biome.id);
                    }
                }
            }
        }

        let proto_block_height = proto_chunk.height();

        for y_offset in 0..proto_block_height {
            let section_index = (y_offset as usize) / 16;
            let relative_y = (y_offset as usize) % 16;

            if let Some(section) = sections.sections.get_mut(section_index) {
                for z in 0..16 {
                    for x in 0..16 {
                        let block =
                            proto_chunk.get_block_state_raw(x as i32, y_offset as i32, z as i32);
                        section.block_states.set(x, relative_y, z, block);
                    }
                }
            }
        }
        let mut chunk = ChunkData {
            light_engine: ChunkLight {
                sky_light: (0..sections.sections.len())
                    .map(|_| {
                        if dimension.has_skylight {
                            // Overworld: Start with full sky light before occlusion
                            LightContainer::new_filled(15)
                        } else {
                            // Nether/End: No sky light permitted
                            LightContainer::new_empty(0)
                        }
                    })
                    .collect(),
                block_light: (0..sections.sections.len())
                    .map(|_| LightContainer::new_empty(0))
                    .collect(),
            },
            section: sections,
            heightmap: Default::default(),
            x: proto_chunk.x,
            z: proto_chunk.z,
            dirty: true,
            block_ticks: Default::default(),
            fluid_ticks: Default::default(),
            block_entities: Default::default(),
            status: proto_chunk.stage.into(),
        };

        chunk.heightmap = chunk.calculate_heightmap();
        *self = Chunk::Level(Arc::new(RwLock::new(chunk)));
    }
}

struct Cache {
    x: i32,
    z: i32,
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

        if dx >= 0 && dx < self.size && dz >= 0 && dz < self.size {
            Some(self.chunks[(dx * self.size + dz) as usize].get_proto_chunk_mut())
        } else {
            None
        }
    }

    fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&ProtoChunk> {
        let dx = chunk_x - self.x;
        let dz = chunk_z - self.z;

        if dx >= 0 && dx < self.size && dz >= 0 && dz < self.size {
            Some(self.chunks[(dx * self.size + dz) as usize].get_proto_chunk())
        } else {
            None
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
            log::error!(
                "illegal get_block_state {pos:?} cache pos ({}, {}) size {}",
                self.x,
                self.z,
                self.size
            );
            return RawBlockState::AIR;
        }
        match &self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => {
                let chunk = data.blocking_read();
                RawBlockState(
                    chunk
                        .section
                        .get_block_absolute_y((pos.x & 15) as usize, pos.y, (pos.z & 15) as usize)
                        .unwrap_or(0),
                )
            }
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
            log::error!(
                "illegal set_block_state {pos:?} cache pos ({}, {}) size {}",
                self.x,
                self.z,
                self.size
            );
            return;
        }
        match &mut self.chunks[(dx * self.size + dz) as usize] {
            Chunk::Level(data) => {
                let mut chunk = data.blocking_write();
                chunk.section.set_block_absolute_y(
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
                let chunk = data.blocking_read();
                chunk.heightmap.get(
                    ChunkHeightmapType::MotionBlocking,
                    x,
                    z,
                    chunk.section.min_y,
                )
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
                let chunk = data.blocking_read();
                chunk.heightmap.get(
                    ChunkHeightmapType::MotionBlockingNoLeaves,
                    x,
                    z,
                    chunk.section.min_y,
                )
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
                let chunk = data.blocking_read();
                chunk
                    .heightmap
                    .get(ChunkHeightmapType::WorldSurface, x, z, chunk.section.min_y) // can we return this?
            }
            Chunk::Proto(data) => data.top_block_height_exclusive(x, z),
        }
    }

    fn ocean_floor_height_exclusive(&self, x: i32, z: i32) -> i32 {
        let dx = (x >> 4) - self.x;
        let dy = (z >> 4) - self.z;
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
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
        debug_assert!(dx < self.size && dy < self.size);
        debug_assert!(dx >= 0 && dy >= 0);
        match &self.chunks[(dx * self.size + dy) as usize] {
            Chunk::Level(data) => {
                // Could this happen?
                Biome::from_id(
                    data.blocking_read()
                        .section
                        .get_rough_biome_absolute_y((x & 15) as usize, y, (z & 15) as usize)
                        .unwrap_or(0),
                )
                .unwrap()
            }
            Chunk::Proto(data) => data.get_biome_for_terrain_gen(x, y, z),
        }
    }

    fn is_air(&self, local_pos: &Vector3<i32>) -> bool {
        GenerationCache::get_block_state(self, local_pos)
            .to_state()
            .is_air()
    }
}

impl Cache {
    fn new(x: i32, z: i32, size: i32) -> Cache {
        Cache {
            x,
            z,
            size,
            chunks: Vec::with_capacity((size * size) as usize),
        }
    }
    #[expect(clippy::too_many_arguments)]
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
            StagedChunkEnum::StructureStart => self.chunks[mid]
                .get_proto_chunk_mut()
                .set_structure_starts(random_config, settings),
            StagedChunkEnum::StructureReferences => ProtoChunk::set_structure_references(self),
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
            Full => {
                let chunk = self.chunks[mid].get_proto_chunk_mut();
                debug_assert_eq!(chunk.stage, Features);
                chunk.stage = Full;
                self.chunks[mid].upgrade_to_level_chunk(&dimension);
            }
            StagedChunkEnum::None => {}
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
                    // log::debug!("single listener {i} send {pos:?}");
                    len -= 1;
                    continue;
                }
                if single[i].1.is_closed() {
                    // let listener_pos = single[i].0;
                    single.remove(i);
                    // log::debug!("single listener dropped {listener_pos:?}");
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
                    Ok(_) => {
                        // log::debug!("global listener {i} send {pos:?}");
                    }
                    Err(_) => {
                        // log::debug!("one global listener dropped");
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

struct ChunkHolder {
    pub target_stage: StagedChunkEnum,
    pub current_stage: StagedChunkEnum,
    pub chunk: Option<Chunk>,
    pub occupied: NodeKey,
    pub occupied_by: EdgeKey,
    pub public: bool,
    pub tasks: [NodeKey; 9],
}

impl Default for ChunkHolder {
    fn default() -> Self {
        Self {
            target_stage: StagedChunkEnum::None,
            current_stage: StagedChunkEnum::None,
            chunk: None,
            occupied: NodeKey::null(),
            occupied_by: EdgeKey::null(),
            public: false,
            tasks: [NodeKey::null(); 9],
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    pub pos: ChunkPos,
    pub stage: StagedChunkEnum,
    pub in_degree: u32,
    pub in_queue: bool,
    pub edge: EdgeKey,
}

impl Node {
    fn new(pos: ChunkPos, stage: StagedChunkEnum) -> Self {
        Self {
            pos,
            stage,
            in_degree: 0,
            in_queue: false,
            edge: EdgeKey::null(),
        }
    }
}

#[derive(Clone, Debug)]
struct Edge {
    pub to: NodeKey,
    pub next: EdgeKey,
}

impl Edge {
    fn new(to: NodeKey, next: EdgeKey) -> Self {
        Self { to, next }
    }
}

new_key_type! { struct NodeKey; }
new_key_type! { struct EdgeKey; }

#[derive(Default)]
#[expect(clippy::upper_case_acronyms)]
struct DAG {
    pub nodes: SlotMap<NodeKey, Node>,
    pub edges: SlotMap<EdgeKey, Edge>,
}

impl DAG {
    fn fast_drop_node(&mut self, node: NodeKey) {
        let mut edge = self.nodes.remove(node).unwrap().edge;
        // debug!("drop node {node:?}");
        while !edge.is_null() {
            edge = self.edges.remove(edge).unwrap().next;
        }
    }
    fn add_edge(&mut self, from: NodeKey, to: NodeKey) {
        self.nodes.get_mut(to).unwrap().in_degree += 1;
        let edge = &mut self.nodes.get_mut(from).unwrap().edge;
        *edge = self.edges.insert(Edge::new(to, *edge));
    }
}

struct TaskHeapNode(i8, NodeKey);
impl PartialEq for TaskHeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for TaskHeapNode {}
impl PartialOrd for TaskHeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TaskHeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

pub struct GenerationSchedule {
    queue: BinaryHeap<TaskHeapNode>,
    graph: DAG,

    last_level: ChunkLevel,
    last_high_priority: Vec<ChunkPos>,
    send_level: Arc<LevelChannel>,

    public_chunk_map: Arc<DashMap<Vector2<i32>, SyncChunk>>,
    chunk_map: HashMap<ChunkPos, ChunkHolder>,
    unload_chunks: HashSetType<ChunkPos>,

    io_lock: IOLock,
    running_task_count: u16,
    recv_chunk: crossfire::MRx<(ChunkPos, RecvChunk)>,
    io_read: crossfire::MTx<ChunkPos>,
    io_write: crossfire::Tx<Vec<(ChunkPos, Chunk)>>,
    generate: crossfire::MTx<(ChunkPos, Cache, StagedChunkEnum)>,
    listener: Arc<ChunkListener>,
}

impl GenerationSchedule {
    pub fn create(
        oi_read_thread_count: usize,
        gen_thread_count: usize,
        level: Arc<Level>,
        level_channel: Arc<LevelChannel>,
        listener: Arc<ChunkListener>,
        thread_tracker: &mut Vec<JoinHandle<()>>,
    ) {
        let tracker = &level.chunk_system_tasks;
        let (send_chunk, recv_chunk) = crossfire::mpmc::unbounded_blocking();
        let (send_read_io, recv_read_io) =
            crossfire::mpmc::bounded_tx_blocking_rx_async(oi_read_thread_count + 2);
        let (send_write_io, recv_write_io) = crossfire::spsc::unbounded_async();
        let (send_gen, recv_gen) = crossfire::mpmc::bounded_blocking(gen_thread_count + 5);
        let io_lock = Arc::new((Mutex::new(HashMapType::default()), Condvar::new()));
        for _ in 0..oi_read_thread_count {
            tracker.spawn(Self::io_read_work(
                recv_read_io.clone(),
                send_chunk.clone(),
                level.clone(),
                io_lock.clone(),
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

        tracker.spawn(Self::io_write_work(
            recv_write_io,
            level.clone(),
            io_lock.clone(),
        ));

        let builder = thread::Builder::new().name("Schedule Thread".to_string());
        thread_tracker.push(
            builder
                .spawn(move || {
                    Self {
                        queue: BinaryHeap::new(),
                        graph: DAG::default(),
                        last_level: ChunkLevel::default(),
                        last_high_priority: Vec::new(),
                        send_level: level_channel,
                        public_chunk_map: level.loaded_chunks.clone(),
                        unload_chunks: HashSetType::default(),
                        io_lock,
                        running_task_count: 0,
                        recv_chunk,
                        io_read: send_read_io,
                        io_write: send_write_io,
                        generate: send_gen,
                        listener,
                        chunk_map: Default::default(),
                    }
                    .work(level);
                })
                .unwrap(),
        )
    }

    fn calc_priority(
        last_level: &ChunkLevel,
        last_high_priority: &Vec<ChunkPos>,
        pos: ChunkPos,
        stage: StagedChunkEnum,
    ) -> i8 {
        if last_high_priority.is_empty() {
            return last_level.get(&pos).unwrap() + (stage as i8);
        }
        for i in last_high_priority {
            let dst = max(abs(i.x - pos.x), abs(i.y - pos.y));
            if dst <= StagedChunkEnum::FULL_RADIUS
                && stage <= StagedChunkEnum::FULL_DEPENDENCIES[dst as usize]
            {
                return last_level.get(&pos).unwrap() + (stage as i8) - 100;
            }
        }
        last_level.get(&pos).unwrap() + (stage as i8)
    }

    fn sort_queue(&mut self) {
        let mut new_queue = BinaryHeap::with_capacity(self.queue.len());
        for i in &self.queue {
            if let Some(node) = self.graph.nodes.get(i.1) {
                new_queue.push(TaskHeapNode(
                    Self::calc_priority(
                        &self.last_level,
                        &self.last_high_priority,
                        node.pos,
                        node.stage,
                    ),
                    i.1,
                ));
            }
        }
        self.queue = new_queue;
    }

    fn resort_work(&mut self, new_data: (Option<LevelChange>, Option<Vec<ChunkPos>>)) -> bool {
        // true -> updated | false -> not update
        if new_data.0.is_none() && new_data.1.is_none() {
            return false;
        }
        if let Some(high_priority) = new_data.1 {
            // log::debug!("receive new priority");
            self.last_high_priority = high_priority;
        }
        let Some(new_level) = new_data.0 else {
            self.sort_queue();
            return true;
        };
        // log::debug!("receive new level");
        for (pos, (old_stage, new_stage)) in new_level.0 {
            debug_assert_ne!(old_stage, new_stage);
            debug_assert_eq!(
                new_stage,
                StagedChunkEnum::level_to_stage(
                    *new_level.1.get(&pos).unwrap_or(&ChunkLoading::MAX_LEVEL)
                )
            );
            let mut holder = self.chunk_map.remove(&pos).unwrap_or_default();
            debug_assert_eq!(holder.target_stage, old_stage);
            holder.target_stage = new_stage;
            if old_stage > new_stage {
                for i in (new_stage.max(holder.current_stage) as usize + 1)..=(old_stage as usize) {
                    let task = &mut holder.tasks[i];
                    self.graph.fast_drop_node(*task);
                    *task = NodeKey::null();
                }
                if new_stage == StagedChunkEnum::None {
                    self.unload_chunks.insert(pos);
                }
            } else {
                if old_stage == StagedChunkEnum::None {
                    self.unload_chunks.remove(&pos);
                    if holder.current_stage == Full && !holder.public {
                        holder.public = true;
                        match holder.chunk.as_ref().unwrap() {
                            Chunk::Level(chunk) => {
                                self.public_chunk_map.insert(pos, chunk.clone());
                                self.listener.process_new_chunk(pos, chunk);
                            }
                            Proto(_) => panic!(),
                        }
                    }
                }
                for i in (old_stage.max(holder.current_stage) as u8 + 1)..=(new_stage as u8) {
                    let task = &mut holder.tasks[i as usize];
                    if task.is_null() {
                        *task = self.graph.nodes.insert(Node::new(pos, i.into()));
                        if !holder.occupied.is_null() {
                            self.graph.add_edge(holder.occupied, *task);
                        }
                    }
                    let task = *task;
                    if i > 1 {
                        let stage = StagedChunkEnum::from(i);
                        let dependency = stage.get_direct_dependencies();
                        let radius = stage.get_direct_radius();
                        for dx in -radius..=radius {
                            for dz in -radius..=radius {
                                let new_pos = pos.add_raw(dx, dz);
                                let req_stage = dependency[dx.abs().max(dz.abs()) as usize];
                                if new_pos == pos {
                                    // TODO
                                    holder.occupied_by = self
                                        .graph
                                        .edges
                                        .insert(Edge::new(task, holder.occupied_by));
                                    if holder.current_stage >= req_stage {
                                        continue;
                                    }
                                    let ano_task = &mut holder.tasks[req_stage as usize];
                                    if ano_task.is_null() {
                                        *ano_task =
                                            self.graph.nodes.insert(Node::new(new_pos, i.into()));
                                    }
                                    self.graph.add_edge(*ano_task, task); // task depend on ano_task
                                    continue;
                                }
                                let ano_chunk = self.chunk_map.entry(new_pos).or_default();
                                ano_chunk.occupied_by = self
                                    .graph
                                    .edges
                                    .insert(Edge::new(task, ano_chunk.occupied_by));

                                if !ano_chunk.occupied.is_null() {
                                    self.graph.add_edge(ano_chunk.occupied, task);
                                }

                                if ano_chunk.current_stage >= req_stage {
                                    continue;
                                }
                                let ano_task = &mut ano_chunk.tasks[req_stage as usize];
                                if ano_task.is_null() {
                                    *ano_task =
                                        self.graph.nodes.insert(Node::new(new_pos, req_stage));
                                }
                                self.graph.add_edge(*ano_task, task); // task depend on ano_task
                            }
                        }
                    }
                    let node = self.graph.nodes.get_mut(task).unwrap();
                    if node.in_degree == 0 {
                        node.in_queue = true;
                        self.queue.push(TaskHeapNode(0, task));
                    }
                }
            }
            self.chunk_map.insert(pos, holder);
        }
        self.last_level = new_level.1;
        self.sort_queue();
        true
    }

    async fn io_read_work(
        recv: crossfire::MAsyncRx<ChunkPos>,
        send: crossfire::MTx<(ChunkPos, RecvChunk)>,
        level: Arc<Level>,
        lock: IOLock,
    ) {
        log::debug!("io read thread start");
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(level.world_gen.random_config.seed);
        let dimension = &level.world_gen.dimension;
        let (t_send, mut t_recv) = tokio::sync::mpsc::channel(2);
        while let Ok(pos) = recv.recv().await {
            // debug!("io read thread receive chunk pos {pos:?}");
            {
                let mut data = lock.0.lock().unwrap();
                while data.contains_key(&pos) {
                    data = tokio::task::block_in_place(|| lock.1.wait(data).unwrap());
                }
            }
            level
                .chunk_saver
                .fetch_chunks(&level.level_folder, &[pos], t_send.clone())
                .await;
            let data = t_recv.recv().await.unwrap();
            match data {
                Loaded(chunk) => {
                    if chunk.read().await.status == ChunkStatus::Full {
                        if send
                            .send((pos, RecvChunk::IO(Chunk::Level(chunk))))
                            .is_err()
                        {
                            break;
                        }
                    } else {
                        // debug!("io read thread receive proto chunk {pos:?}",);
                        let val =
                            RecvChunk::IO(Chunk::Proto(Box::new(ProtoChunk::from_chunk_data(
                                chunk.read().await.deref(),
                                dimension,
                                level.world_gen.default_block,
                                biome_mixer_seed,
                            ))));
                        if send.send((pos, val)).is_err() {
                            break;
                        }
                    }
                    continue;
                }
                LoadedData::Missing(_) => {}
                LoadedData::Error(_) => {
                    log::warn!("chunk data read error pos: {pos:?}. regenerating");
                }
            }
            if send
                .send((
                    pos,
                    RecvChunk::IO(Proto(Box::new(ProtoChunk::new(
                        pos.x,
                        pos.y,
                        dimension,
                        level.world_gen.default_block,
                        biome_mixer_seed,
                    )))),
                ))
                .is_err()
            {
                break;
            }
        }
        log::debug!("io read thread stop");
    }

    async fn io_write_work(recv: AsyncRx<Vec<(ChunkPos, Chunk)>>, level: Arc<Level>, lock: IOLock) {
        log::info!("io write thread start",);
        while let Ok(data) = recv.recv().await {
            // debug!("io write thread receive chunks size {}", data.len());
            let mut vec = Vec::with_capacity(data.len());
            for (pos, chunk) in data {
                match chunk {
                    Chunk::Level(chunk) => vec.push((pos, chunk)),
                    Proto(chunk) => {
                        let mut temp = Proto(chunk);
                        temp.upgrade_to_level_chunk(&level.world_gen.dimension);
                        let Chunk::Level(chunk) = temp else { panic!() };
                        vec.push((pos, chunk));
                    }
                }
            }
            let pos = vec.iter().map(|(pos, _)| *pos).collect_vec();
            level
                .chunk_saver
                .save_chunks(&level.level_folder, vec)
                .await
                .unwrap();
            for i in pos {
                let mut data = lock.0.lock().unwrap();
                match data.entry(i) {
                    Entry::Occupied(mut entry) => {
                        let rc = entry.get_mut();
                        if *rc == 1 {
                            entry.remove();
                            drop(data);
                            lock.1.notify_all();
                        } else {
                            *rc -= 1;
                        }
                    }
                    Entry::Vacant(_) => panic!(),
                }
            }
        }
        log::info!(
            "io write thread stop id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
    }

    fn generation_work(
        recv: crossfire::MRx<(ChunkPos, Cache, StagedChunkEnum)>,
        send: crossfire::MTx<(ChunkPos, RecvChunk)>,
        level: Arc<Level>,
    ) {
        log::debug!(
            "generation thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );

        let settings = gen_settings_from_dimension(&level.world_gen.dimension);
        while let Ok((pos, mut cache, stage)) = recv.recv() {
            // debug!("generation thread receive chunk pos {pos:?} to stage {stage:?}");
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
        log::debug!(
            "generation thread stop id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
    }

    fn unload_chunk(&mut self) {
        let mut unload_chunks = HashSetType::default();
        swap(&mut unload_chunks, &mut self.unload_chunks);
        let mut chunks = Vec::with_capacity(unload_chunks.len());
        for pos in unload_chunks {
            let holder = self.chunk_map.get_mut(&pos).unwrap();
            debug_assert_eq!(holder.target_stage, StagedChunkEnum::None);
            if holder.occupied.is_null() {
                let mut tmp = None;
                swap(&mut holder.chunk, &mut tmp);
                let Some(tmp) = tmp else {
                    continue;
                };
                match tmp {
                    Chunk::Level(chunk) => {
                        if holder.public {
                            self.public_chunk_map.remove(&pos);
                            holder.public = false;
                        }
                        if Arc::strong_count(&chunk) != 1 {
                            // log::debug!(
                            //     "chunk {pos:?} is still used somewhere. it can't be unloaded"
                            // );
                            self.unload_chunks.insert(pos);
                            holder.chunk = Some(Chunk::Level(chunk));
                        } else {
                            // log::debug!("unload chunk {pos:?} to file");
                            chunks.push((pos, Chunk::Level(chunk)));
                            self.chunk_map.remove(&pos);
                        }
                    }
                    Chunk::Proto(chunk) => {
                        debug_assert!(!holder.public);
                        // log::debug!("unload proto chunk {pos:?} to file");
                        chunks.push((pos, Proto(chunk)));
                        self.chunk_map.remove(&pos);
                    }
                }
            }
        }
        // log::debug!("send {} unloaded chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        self.io_write.send(chunks).expect("io write thread stop");
    }

    fn save_all_chunk(&self, save_proto_chunk: bool) {
        let mut chunks = Vec::with_capacity(self.chunk_map.len());
        for (pos, chunk) in &self.chunk_map {
            if let Some(chunk) = &chunk.chunk {
                match chunk {
                    Chunk::Level(chunk) => {
                        chunks.push((*pos, Chunk::Level(chunk.clone())));
                    }
                    Chunk::Proto(chunk) => {
                        if save_proto_chunk {
                            chunks.push((*pos, Chunk::Proto(chunk.clone())));
                        }
                    }
                }
            }
        }
        // log::debug!("send {} chunks to io write", chunks.len());
        if chunks.is_empty() {
            return;
        }
        let mut data = self.io_lock.0.lock().unwrap();
        for (pos, _chunk) in &chunks {
            *data.entry(*pos).or_insert(0) += 1;
        }
        drop(data);
        self.io_write.send(chunks).expect("io write thread stop");
    }

    fn drop_node(&mut self, node: NodeKey) {
        let Some(old) = self.graph.nodes.remove(node) else {
            return;
        };
        // debug!("drop node {node:?}");
        let mut edge = old.edge;
        while !edge.is_null() {
            let cur = self.graph.edges.remove(edge).unwrap();
            if let Some(node) = self.graph.nodes.get_mut(cur.to) {
                debug_assert!(node.in_degree >= 1);
                node.in_degree -= 1;
                if node.in_degree == 0 && !node.in_queue {
                    self.queue.push(TaskHeapNode(
                        Self::calc_priority(
                            &self.last_level,
                            &self.last_high_priority,
                            node.pos,
                            node.stage,
                        ),
                        cur.to,
                    ));
                    node.in_queue = true;
                }
            }
            edge = cur.next;
        }
    }

    fn receive_chunk(&mut self, pos: ChunkPos, data: RecvChunk) {
        match data {
            RecvChunk::IO(chunk) => {
                // debug!("receive io chunk pos {pos:?}");
                let mut holder = self.chunk_map.remove(&pos).unwrap();
                debug_assert!(holder.chunk.is_none());
                debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);

                for i in (holder.current_stage as usize + 1)..=(chunk.get_stage_id() as usize) {
                    self.drop_node(holder.tasks[i]);
                    holder.tasks[i] = NodeKey::null();
                }
                holder.current_stage = StagedChunkEnum::from(chunk.get_stage_id());
                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                self.drop_node(holder.occupied);
                holder.occupied = NodeKey::null();

                debug_assert!(!holder.public);
                match &chunk {
                    Chunk::Level(data) => {
                        let result = self.public_chunk_map.insert(pos, data.clone());
                        debug_assert!(result.is_none());
                        holder.public = true;
                        self.listener.process_new_chunk(pos, data);
                    }
                    Chunk::Proto(_) => {}
                }
                holder.chunk = Some(chunk);
                self.chunk_map.insert(pos, holder);
            }
            RecvChunk::Generation(data) => {
                // debug!("receive gen chunk pos {pos:?}");
                let mut dx = 0;
                let mut dy = 0;
                for chunk in data.chunks {
                    let new_pos = ChunkPos::new(data.x + dx, data.z + dy);
                    match chunk {
                        Chunk::Level(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();
                            if new_pos == pos {
                                debug_assert_eq!(holder.current_stage, Features);
                                self.drop_node(holder.tasks[Full as usize]);
                                holder.tasks[Full as usize] = NodeKey::null();
                                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                                self.drop_node(holder.occupied);
                                holder.current_stage = Full;

                                holder.chunk = Some(Chunk::Level(chunk.clone()));
                                debug_assert!(!holder.public);
                                let result = self.public_chunk_map.insert(new_pos, chunk.clone());
                                holder.public = true;
                                debug_assert!(result.is_none());
                                self.listener.process_new_chunk(new_pos, &chunk);
                            }

                            holder.occupied = NodeKey::null();
                            self.chunk_map.insert(new_pos, holder);
                        }
                        Chunk::Proto(chunk) => {
                            let mut holder = self.chunk_map.remove(&new_pos).unwrap();
                            debug_assert!(holder.chunk.is_none());
                            debug_assert_eq!(
                                holder.current_stage as u8,
                                if new_pos == pos {
                                    chunk.stage_id() - 1
                                } else {
                                    chunk.stage_id()
                                }
                            );

                            if new_pos == pos {
                                debug_assert_ne!(holder.current_stage, StagedChunkEnum::None);
                                let stage = chunk.stage_id();
                                self.drop_node(holder.tasks[stage as usize]);
                                holder.tasks[stage as usize] = NodeKey::null();
                                debug_assert!(self.graph.nodes.contains_key(holder.occupied));
                                self.drop_node(holder.occupied);
                                holder.current_stage = StagedChunkEnum::from(stage);
                            }
                            holder.occupied = NodeKey::null();
                            holder.chunk = Some(Chunk::Proto(chunk));
                            self.chunk_map.insert(new_pos, holder);
                        }
                    }
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

    fn work(mut self, level: Arc<Level>) {
        log::info!(
            "schedule thread start id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
        // let mut clock = Instant::now();
        loop {
            if level.should_unload.load(Relaxed) {
                // log::debug!("unload chunk signal");
                self.unload_chunk();
                level.should_unload.store(false, Relaxed);
            }
            if level.should_save.load(Relaxed) {
                // log::debug!("save all chunk signal");
                self.save_all_chunk(false);
                level.should_save.store(false, Relaxed);
            }
            if level.shut_down_chunk_system.load(Relaxed) {
                // log::debug!("shut down signal");
                break;
            }

            'out2: while let Some(task) = self.queue.pop() {
                if self.resort_work(self.send_level.get()) {
                    self.queue.push(task);
                    break 'out2;
                }
                while let Ok((pos, data)) = self.recv_chunk.try_recv() {
                    self.receive_chunk(pos, data);
                }
                if let Some(node) = self.graph.nodes.get_mut(task.1) {
                    if node.in_degree != 0 {
                        node.in_queue = false;
                        continue;
                    }
                    let node = node.clone();
                    if node.stage == Empty {
                        self.running_task_count += 1;
                        let holder = self.chunk_map.get_mut(&node.pos).unwrap();
                        debug_assert!(holder.occupied.is_null());
                        debug_assert_eq!(holder.current_stage, StagedChunkEnum::None);
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));
                        for i in
                            (holder.current_stage as usize + 1)..=(holder.target_stage as usize)
                        {
                            self.graph.add_edge(occupy, holder.tasks[i]);
                        }
                        holder.occupied = occupy;

                        // debug!("send task {:?} {node:?}", task.1);

                        self.io_read
                            .send(node.pos)
                            .expect("io thread close unexpectedly");
                    } else {
                        let occupy = self.graph.nodes.insert(Node::new(
                            ChunkPos::new(i32::MAX, i32::MAX),
                            StagedChunkEnum::None,
                        ));
                        let write_radius = node.stage.get_write_radius();
                        let mut cache = Cache::new(
                            node.pos.x - write_radius,
                            node.pos.y - write_radius,
                            write_radius << 1 | 1,
                        );
                        #[cfg(debug_assertions)]
                        {
                            let dp = node.stage.get_direct_dependencies();
                            let r = node.stage.get_direct_radius();
                            for dx in -r..=r {
                                for dy in -r..=r {
                                    let new_pos = node.pos.add_raw(dx, dy);
                                    let holder = self.chunk_map.get(&new_pos).unwrap();
                                    let dst = dy.abs().max(dx.abs());
                                    debug_assert!(holder.current_stage >= dp[dst as usize]);
                                    if dx == 0 && dy == 0 {
                                        debug_assert_eq!(holder.current_stage, dp[0]);
                                    }
                                }
                            }
                        }

                        for dx in -write_radius..=write_radius {
                            for dy in -write_radius..=write_radius {
                                let new_pos = node.pos.add_raw(dx, dy);
                                let holder = self.chunk_map.get_mut(&new_pos).unwrap();
                                let mut tmp = None;
                                swap(&mut tmp, &mut holder.chunk);
                                match tmp.unwrap() {
                                    Chunk::Level(chunk) => {
                                        cache.chunks.push(Chunk::Level(chunk.clone()));
                                        holder.chunk = Some(Chunk::Level(chunk));
                                    }
                                    Proto(chunk) => cache.chunks.push(Proto(chunk)),
                                }

                                debug_assert!(holder.occupied.is_null());

                                let mut cur_edge = holder.occupied_by;
                                let mut prev_edge = EdgeKey::null();
                                let mut change_head = None;
                                while !cur_edge.is_null() {
                                    let edge = self.graph.edges.get(cur_edge).unwrap();
                                    if self.graph.nodes.contains_key(edge.to) {
                                        prev_edge = cur_edge;
                                        cur_edge = edge.next;
                                        self.graph.add_edge(occupy, edge.to);
                                    } else {
                                        let next = edge.next;
                                        self.graph.edges.remove(cur_edge);
                                        cur_edge = next;
                                        if prev_edge.is_null() {
                                            change_head = Some(next);
                                        } else {
                                            self.graph.edges.get_mut(prev_edge).unwrap().next =
                                                next;
                                        }
                                    }
                                }
                                if let Some(next) = change_head {
                                    holder.occupied_by = next;
                                }

                                holder.occupied = occupy;
                            }
                        }

                        // debug!("send task {:?} {node:?}", task.1);

                        self.running_task_count += 1;
                        self.generate
                            .send((node.pos, cache, node.stage))
                            .expect("generate thread close unexpectedly");
                    }
                }
            }

            if self.queue.is_empty() {
                // debug!("the queue is empty. thread sleep");
                while self.running_task_count > 0 && self.queue.is_empty() {
                    let (pos, data) = self.recv_chunk.recv().expect("recv_chunk stop");
                    self.receive_chunk(pos, data);
                    self.resort_work(self.send_level.get());
                }
                if self.queue.is_empty() {
                    // debug!("no work to do. thread sleep");
                    debug_assert!(self.debug_check());
                    debug_assert_eq!(self.running_task_count, 0);
                    self.resort_work(self.send_level.wait_and_get(&level));
                }
            }
        }
        log::info!("waiting all generation task finished");
        while self.running_task_count > 0 {
            let (pos, data) = self.recv_chunk.recv().expect("recv_chunk stop");
            self.receive_chunk(pos, data);
        }
        log::info!("saving all chunks");
        self.save_all_chunk(true);
        log::info!("there are {} chunks to write", self.io_write.len());
        log::info!(
            "schedule thread stop id: {:?} name: {}",
            thread::current().id(),
            thread::current().name().unwrap_or("unknown")
        );
    }

    fn debug_check(&self) -> bool {
        if !self.graph.nodes.is_empty() {
            // error!("nodes: {:?}", self.graph.nodes);
            for (key, value) in &self.graph.nodes {
                error!("unrelease node {key:?}: {value:?}");
            }
            panic!("nodes count error");
        }
        for (pos, holder) in &self.chunk_map {
            for i in &holder.tasks {
                debug_assert!(i.is_null());
            }
            debug_assert_eq!(
                holder.target_stage,
                StagedChunkEnum::level_to_stage(
                    *self.last_level.get(pos).unwrap_or(&ChunkLoading::MAX_LEVEL)
                )
            );
            debug_assert!(holder.current_stage >= holder.target_stage);
            debug_assert!(holder.occupied.is_null());
            if holder.current_stage != StagedChunkEnum::None {
                debug_assert_eq!(
                    holder.chunk.as_ref().unwrap().get_stage_id(),
                    holder.current_stage as u8
                );
            }
        }
        true
    }
}
