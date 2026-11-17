use super::{ChunkLevel, ChunkPos, HashMapType, LevelChannel};
use crate::chunk_system::chunk_state::StagedChunkEnum; // Fixed path
use std::cmp::min;
use std::collections::hash_map::Entry;
use std::fmt::Write;
use std::mem::take;
use std::sync::Arc;
use tracing::debug;

pub struct ChunkLoading {
    pub is_priority_dirty: bool,
    pub pos_level: ChunkLevel,
    change: HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
    /// Levels written to `pos_level` since the last send, in order, with `MAX_LEVEL` for
    /// removed entries.
    level_change: Vec<(ChunkPos, i8)>,
    pub ticket: HashMapType<ChunkPos, Vec<i8>>, // TODO lifetime & id
    pub high_priority: Vec<ChunkPos>,
    pub sender: Arc<LevelChannel>,
}

impl ChunkLoading {
    // pub const FULL_CHUNK_LEVEL: i8 = 33;
    pub const FULL_CHUNK_LEVEL: i8 = 43;
    pub const MAX_LEVEL: i8 = 49; // level 49 will be unloaded.
    fn debug_check_error(&self) -> bool {
        let mut temp = ChunkLevel::default();
        for (ticket_pos, levels) in &self.ticket {
            let Some(&level) = levels.iter().min() else {
                continue;
            };
            let range = (Self::MAX_LEVEL - level - 1).max(0) as u8;
            for r in 0..=range {
                let ring_level = level + r as i8;
                for &(dx, dy) in pumpkin_data::chunk_view_lut::get_chebyshev_ring(r) {
                    let new_pos = ticket_pos.add_raw(dx as i32, dy as i32);
                    let i = temp.entry(new_pos).or_insert(Self::MAX_LEVEL);
                    *i = min(*i, ring_level);
                }
            }
        }
        if temp.len() != self.pos_level.len() {
            debug!("temp: \n{temp:?}");
            debug!("pos_level: \n{:?}", self.pos_level);
        }
        assert_eq!(temp.len(), self.pos_level.len());
        for val in &temp {
            if val
                != self
                    .pos_level
                    .get_key_value(val.0)
                    .expect("key value exists")
            {
                Self::dump_level_debug(
                    &self.high_priority,
                    &self.pos_level,
                    val.0.x - 40,
                    val.0.x + 40,
                    val.0.y - 40,
                    val.0.y + 40,
                );
            }
            assert_eq!(
                val,
                self.pos_level
                    .get_key_value(val.0)
                    .expect("key value exists")
            );
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
            let _ = write!(header, "{y:4}");
        }

        let grid: String = (sx..=tx)
            .map(|x| {
                let mut row = format!("{x:3}");
                let mut cols = String::new();
                for y in sy..=ty {
                    let _ = write!(
                        cols,
                        "{:4}",
                        map.get(&ChunkPos::new(x, y)).unwrap_or(&Self::MAX_LEVEL)
                    );
                }
                row.push_str(&cols);
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

    #[inline]
    #[must_use]
    pub const fn get_level_from_simulation_distance(simulation_distance: u8) -> i8 {
        Self::FULL_CHUNK_LEVEL - (simulation_distance as i8)
    }

    pub fn new(sender: Arc<LevelChannel>) -> Self {
        Self {
            is_priority_dirty: true,
            pos_level: ChunkLevel::default(),
            change: HashMapType::default(),
            level_change: Vec::new(),
            ticket: HashMapType::default(),
            high_priority: Vec::new(),
            sender,
        }
    }

    fn record_change(
        change: &mut HashMapType<ChunkPos, (StagedChunkEnum, StagedChunkEnum)>,
        level_change: &mut Vec<(ChunkPos, i8)>,
        pos: ChunkPos,
        old_level: i8,
        new_level: i8,
    ) {
        if old_level == new_level {
            return;
        }
        level_change.push((pos, new_level));
        let old_stage = StagedChunkEnum::level_to_stage(old_level);
        let new_stage = StagedChunkEnum::level_to_stage(new_level);
        if old_stage == new_stage {
            return;
        }
        match change.entry(pos) {
            Entry::Occupied(mut entry) => {
                let i = entry.get_mut();
                debug_assert_eq!(i.1, old_stage);
                if i.0 == new_stage {
                    entry.remove();
                } else {
                    i.1 = new_stage;
                }
            }
            Entry::Vacant(entry) => {
                entry.insert((old_stage, new_stage));
            }
        }
    }

    pub fn send_change(&mut self) {
        // debug!("sending change");
        // Stage changes can cancel out while their level writes stay, so check both.
        if !self.change.is_empty() || !self.level_change.is_empty() {
            let change = (take(&mut self.change), take(&mut self.level_change));
            if self.is_priority_dirty {
                self.is_priority_dirty = false;
                self.sender.set_both(change, self.high_priority.clone());
            } else {
                self.sender.set_level(change);
            }
        }
        if self.is_priority_dirty {
            self.is_priority_dirty = false;
            self.sender.set_priority(self.high_priority.clone());
        }
    }

    pub fn add_force_ticket(&mut self, pos: ChunkPos) {
        // debug!("add force ticket at {pos:?}");
        self.high_priority.push(pos);
        self.is_priority_dirty = true;
        self.add_ticket(pos, Self::FULL_CHUNK_LEVEL);
    }

    pub fn remove_force_ticket(&mut self, pos: ChunkPos) {
        // debug!("remove force ticket at {pos:?}");
        if let Some(index) = self.high_priority.iter().position(|x| *x == pos) {
            self.high_priority.remove(index);
        }
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

        let max_range = (Self::MAX_LEVEL - level - 1).max(0) as u8;
        for r in 0..=max_range {
            let ring_level = level + r as i8;
            for &(dx, dy) in pumpkin_data::chunk_view_lut::get_chebyshev_ring(r) {
                let p = pos.add_raw(dx as i32, dy as i32);
                match self.pos_level.entry(p) {
                    Entry::Occupied(mut entry) => {
                        let cur = *entry.get();
                        if cur > ring_level {
                            *entry.get_mut() = ring_level;
                            Self::record_change(
                                &mut self.change,
                                &mut self.level_change,
                                p,
                                cur,
                                ring_level,
                            );
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(ring_level);
                        Self::record_change(
                            &mut self.change,
                            &mut self.level_change,
                            p,
                            Self::MAX_LEVEL,
                            ring_level,
                        );
                    }
                }
            }
        }
        debug_assert!(self.debug_check_error());
    }

    pub fn remove_ticket(&mut self, pos: ChunkPos, level: i8) {
        // debug!("remove ticket at {pos:?} level {level}");
        debug_assert!(level < Self::MAX_LEVEL);
        let Some(vec) = self.ticket.get_mut(&pos) else {
            // warn!("No ticket found at {pos:?}");
            return;
        };
        let Some(index) = vec.iter().position(|x| *x == level) else {
            // warn!("No ticket found at {pos:?}");
            return;
        };
        vec.remove(index);
        let source = *vec.iter().min().unwrap_or(&Self::MAX_LEVEL);
        if vec.is_empty() {
            self.ticket.remove(&pos);
        }

        let old_level = *self.pos_level.get(&pos).unwrap_or(&Self::MAX_LEVEL);
        if level != old_level || source == level {
            debug_assert!(self.debug_check_error());
            return;
        }

        let range = (Self::MAX_LEVEL - old_level - 1).max(0) as u8;

        let nearby_tickets: Vec<(ChunkPos, i8)> = self
            .ticket
            .iter()
            .filter_map(|(&t_pos, levels)| {
                let &min_lvl = levels.iter().min()?;
                let t_range = (Self::MAX_LEVEL - min_lvl - 1).max(0) as i32;
                ((t_pos.x - pos.x).abs() <= range as i32 + t_range
                    && (t_pos.y - pos.y).abs() <= range as i32 + t_range)
                    .then_some((t_pos, min_lvl))
            })
            .collect();

        for r in 0..=range {
            let removed_contrib = old_level + r as i8;
            for &(dx, dy) in pumpkin_data::chunk_view_lut::get_chebyshev_ring(r) {
                let p = pos.add_raw(dx as i32, dy as i32);
                let cur = *self.pos_level.get(&p).unwrap_or(&Self::MAX_LEVEL);
                if cur == removed_contrib {
                    let mut new_level = Self::MAX_LEVEL;
                    for &(t_pos, t_level) in &nearby_tickets {
                        let dist = (p.x - t_pos.x).abs().max((p.y - t_pos.y).abs()) as i8;
                        let lvl = t_level + dist;
                        if lvl < new_level {
                            new_level = lvl;
                        }
                    }
                    if new_level != cur {
                        if new_level == Self::MAX_LEVEL {
                            self.pos_level.remove(&p);
                        } else {
                            self.pos_level.insert(p, new_level);
                        }
                        Self::record_change(
                            &mut self.change,
                            &mut self.level_change,
                            p,
                            cur,
                            new_level,
                        );
                    }
                }
            }
        }
        debug_assert!(self.debug_check_error());
    }
}

#[test]
#[expect(clippy::print_stdout)]
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
            let _ = write!(header, "{y:4}");
        }

        let grid: String = (sx..=tx)
            .map(|x| {
                let mut row = format!("{x:3}");
                for y in sy..=ty {
                    let level = a
                        .pos_level
                        .get(&ChunkPos::new(x, y))
                        .unwrap_or(&ChunkLoading::MAX_LEVEL);

                    let _ = write!(row, "{level:4}");
                }
                row
            })
            .collect::<Vec<_>>()
            .join("\n");

        println!("\nloading level:\n{header}\n{grid}");
    }
}

// The scheduler rebuilds the level map from the sent deltas, so every send must leave its
// copy equal to `pos_level`, including levels that changed without a stage change.
#[test]
fn level_deltas_rebuild_pos_level() {
    let channel = Arc::new(LevelChannel::new());
    let mut loading = ChunkLoading::new(channel.clone());
    let mut mirror = ChunkLevel::default();
    let mut sends = 0;

    let steps: [(bool, (i32, i32), i8); 8] = [
        (true, (0, 0), ChunkLoading::FULL_CHUNK_LEVEL),
        // Also lowers levels near the center whose stage stays `Full`.
        (true, (0, 0), 30),
        (true, (3, 2), ChunkLoading::FULL_CHUNK_LEVEL),
        (false, (0, 0), 30),
        (true, (20, 20), 27),
        (false, (0, 0), ChunkLoading::FULL_CHUNK_LEVEL),
        (false, (20, 20), 27),
        (false, (3, 2), ChunkLoading::FULL_CHUNK_LEVEL),
    ];
    for (add, pos, level) in steps {
        if add {
            loading.add_ticket(pos.into(), level);
        } else {
            loading.remove_ticket(pos.into(), level);
        }
        loading.send_change();
        if let (Some((_, levels)), _) = channel.get() {
            sends += 1;
            for (pos, level) in levels {
                if level >= ChunkLoading::MAX_LEVEL {
                    mirror.remove(&pos);
                } else {
                    mirror.insert(pos, level);
                }
            }
            assert_eq!(mirror, loading.pos_level);
        }
    }
    assert_eq!(sends, 7);
    assert!(mirror.is_empty());
}

// Adding and removing a ticket in one batch cancels its stage changes, but its level writes
// still have to be sent, or they pile up until the next stage change.
#[test]
fn canceled_stage_changes_still_send_levels() {
    let channel = Arc::new(LevelChannel::new());
    let mut loading = ChunkLoading::new(channel.clone());
    loading.add_ticket((0, 0).into(), ChunkLoading::FULL_CHUNK_LEVEL);
    loading.remove_ticket((0, 0).into(), ChunkLoading::FULL_CHUNK_LEVEL);
    loading.send_change();
    let (Some((stages, levels)), _) = channel.get() else {
        panic!("level writes were not sent");
    };
    assert!(stages.is_empty());
    assert!(!levels.is_empty());
    assert!(loading.level_change.is_empty());
}
