use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;

/// A single named stopwatch tracking elapsed time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopwatch {
    pub start_time: u64,
    pub accumulated_elapsed_time: u64,
}

impl Stopwatch {
    #[must_use]
    pub const fn new(current_time: u64) -> Self {
        Self {
            start_time: current_time,
            accumulated_elapsed_time: 0,
        }
    }

    #[must_use]
    pub const fn new_with_accumulated(current_time: u64, accumulated_elapsed_time: u64) -> Self {
        Self {
            start_time: current_time,
            accumulated_elapsed_time,
        }
    }

    #[must_use]
    pub const fn elapsed_milliseconds(&self, current_time: u64) -> u64 {
        current_time
            .saturating_sub(self.start_time)
            .saturating_add(self.accumulated_elapsed_time)
    }

    #[must_use]
    pub fn elapsed_seconds(&self, current_time: u64) -> f64 {
        self.elapsed_milliseconds(current_time) as f64 / 1000.0
    }
}

/// Persistent manager for all server stopwatches.
#[derive(Debug, Clone, Default)]
pub struct Stopwatches {
    stopwatches: HashMap<String, Stopwatch>,
    dirty: bool,
}

impl Stopwatches {
    #[must_use]
    pub fn new() -> Self {
        Self {
            stopwatches: HashMap::new(),
            dirty: false,
        }
    }

    #[must_use]
    pub fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Stopwatch> {
        self.stopwatches.get(id)
    }

    pub fn add(&mut self, id: String, stopwatch: Stopwatch) -> bool {
        if let std::collections::hash_map::Entry::Vacant(e) = self.stopwatches.entry(id) {
            e.insert(stopwatch);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, id: &str, update: impl FnOnce(&Stopwatch) -> Stopwatch) -> bool {
        if let Some(stopwatch) = self.stopwatches.get_mut(id) {
            *stopwatch = update(stopwatch);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let removed = self.stopwatches.remove(id).is_some();
        if removed {
            self.dirty = true;
        }
        removed
    }

    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.stopwatches.keys().cloned().collect();
        ids.sort();
        ids
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty || !self.stopwatches.is_empty()
    }

    pub const fn set_dirty(&mut self) {
        self.dirty = true;
    }

    #[must_use]
    pub fn unpack(map: &HashMap<String, u64>) -> Self {
        let mut result = Self::new();
        let now = Self::current_time();
        for (id, accumulated) in map {
            result.stopwatches.insert(
                id.clone(),
                Stopwatch::new_with_accumulated(now, *accumulated),
            );
        }
        result
    }

    #[must_use]
    pub fn pack(&self) -> HashMap<String, u64> {
        let now = Self::current_time();
        let mut result = HashMap::new();
        for (id, stopwatch) in &self.stopwatches {
            result.insert(id.clone(), stopwatch.elapsed_milliseconds(now));
        }
        result
    }

    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let mut stopwatches_tag = NbtCompound::new();
        let now = Self::current_time();
        for (id, stopwatch) in &self.stopwatches {
            stopwatches_tag.put_long(id, stopwatch.elapsed_milliseconds(now) as i64);
        }
        let mut data = NbtCompound::new();
        data.put_compound("stopwatches", stopwatches_tag);
        data
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let mut result = Self::new();
        let now = Self::current_time();
        if let Some(NbtTag::Compound(stopwatches_tag)) = nbt.get("stopwatches") {
            for (id, tag) in &stopwatches_tag.child_tags {
                if let NbtTag::Long(accumulated) = tag {
                    result.stopwatches.insert(
                        id.to_string(),
                        Stopwatch::new_with_accumulated(now, *accumulated as u64),
                    );
                }
            }
        }
        result
    }
}
