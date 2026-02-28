use super::{ChunkLevel, ChunkPos, HashMapType, LevelChannel};
use crate::chunk_system::chunk_state::StagedChunkEnum; // Fixed path
use itertools::Itertools;
use std::cmp::{Ordering, PartialEq, min};
use std::collections::BinaryHeap;
use std::collections::hash_map::Entry;
use std::mem::swap;
use std::sync::Arc;
use tracing::debug;

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
    const fn new() -> Self {
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
        &self,
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

impl ChunkLoading {
    // pub const FULL_CHUNK_LEVEL: i8 = 33;
    pub const FULL_CHUNK_LEVEL: i8 = 43;
    pub const MAX_LEVEL: i8 = 47; // level 47 will be unloaded.
    fn debug_check_error(&self) -> bool {
        let mut temp = ChunkLevel::default();
        for (ticket_pos, levels) in &self.ticket {
            let level = *levels.iter().min().unwrap();
            let range = Self::MAX_LEVEL - level - 1;
            for dx in -range..=range {
                for dy in -range..=range {
                    let new_pos = ticket_pos.add_raw(dx as i32, dy as i32);
                    let level_from_source = level + dx.abs().max(dy.abs());
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
        pri: &[ChunkPos],
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
                                map.get(&ChunkPos::new(x, y)).unwrap_or(&Self::MAX_LEVEL)
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
    #[must_use]
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
            if (ticket_pos.x - pos.x).abs() <= range && (ticket_pos.y - pos.y).abs() <= range {
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
        // debug!("add force ticket at {pos:?}");
        self.high_priority.push(pos);
        self.is_priority_dirty = true;
        self.add_ticket(pos, Self::FULL_CHUNK_LEVEL);
    }
    pub fn remove_force_ticket(&mut self, pos: ChunkPos) {
        // debug!("remove force ticket at {pos:?}");
        let index = self
            .high_priority
            .iter()
            .find_position(|x| **x == pos)
            .unwrap()
            .0;
        self.high_priority.remove(index);
        self.is_priority_dirty = true;
        self.remove_ticket(pos, Self::FULL_CHUNK_LEVEL);
    }
    pub fn add_ticket(&mut self, pos: ChunkPos, level: i8) {
        // debug!("add ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        match self.ticket.entry(pos) {
            Entry::Occupied(mut vec) => {
                vec.get_mut().push(level);
            }
            Entry::Vacant(empty) => {
                empty.insert(vec![level]);
            }
        }

        let old = *self.pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL);
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
        // debug!("remove ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        let Some(vec) = self.ticket.get_mut(&pos) else {
            // warn!("No ticket found at {pos:?}");
            return;
        };
        let Some((index, _)) = vec.iter().find_position(|x| **x == level) else {
            // warn!("No ticket found at {pos:?}");
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
