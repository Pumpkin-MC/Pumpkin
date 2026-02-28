use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::pathfinder::node::PathType;
use std::collections::HashMap;

const CACHE_SIZE: usize = 4096;
const CACHE_MASK: usize = CACHE_SIZE - 1;

#[derive(Debug, Clone)]
pub struct PathTypeCache {
    positions: Vec<i64>,
    path_types: Vec<Option<PathType>>,
    overflow_cache: HashMap<Vector3<i32>, PathType>,
}

impl PathTypeCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            positions: vec![0; CACHE_SIZE],
            path_types: vec![None; CACHE_SIZE],
            overflow_cache: HashMap::new(),
        }
    }

    pub fn get_or_compute<F>(&mut self, pos: Vector3<i32>, compute_fn: F) -> PathType
    where
        F: FnOnce() -> PathType,
    {
        let key = Self::position_to_key(pos);
        let index = Self::key_to_index(key);

        if let Some(cached_type) = self.get_from_array(index, key) {
            return cached_type;
        }

        if let Some(&cached_type) = self.overflow_cache.get(&pos) {
            return cached_type;
        }

        let path_type = compute_fn();
        self.insert_internal(pos, key, index, path_type);
        path_type
    }

    #[must_use]
    pub fn get(&self, pos: Vector3<i32>) -> Option<PathType> {
        let key = Self::position_to_key(pos);
        let index = Self::key_to_index(key);

        if let Some(cached_type) = self.get_from_array(index, key) {
            return Some(cached_type);
        }

        self.overflow_cache.get(&pos).copied()
    }

    pub fn insert(&mut self, pos: Vector3<i32>, path_type: PathType) {
        let key = Self::position_to_key(pos);
        let index = Self::key_to_index(key);
        self.insert_internal(pos, key, index, path_type);
    }

    fn insert_internal(&mut self, pos: Vector3<i32>, key: i64, index: usize, path_type: PathType) {
        if self.path_types[index].is_none() || self.positions[index] != key {
            self.positions[index] = key;
            self.path_types[index] = Some(path_type);
        } else {
            self.overflow_cache.insert(pos, path_type);
        }
    }

    fn get_from_array(&self, index: usize, key: i64) -> Option<PathType> {
        if self.positions[index] == key {
            self.path_types[index]
        } else {
            None
        }
    }

    pub fn invalidate(&mut self, pos: Vector3<i32>) {
        let key = Self::position_to_key(pos);
        let index = Self::key_to_index(key);

        if self.positions[index] == key {
            self.path_types[index] = None;
        }

        self.overflow_cache.remove(&pos);
    }

    pub fn clear(&mut self) {
        for i in 0..CACHE_SIZE {
            self.path_types[i] = None;
        }
        self.overflow_cache.clear();
    }

    fn position_to_key(pos: Vector3<i32>) -> i64 {
        ((i64::from(pos.x) & 0x03FF_FFFF) << 38)
            | ((i64::from(pos.z) & 0x03FF_FFFF) << 12)
            | (i64::from(pos.y) & 0xFFF)
    }

    const fn key_to_index(key: i64) -> usize {
        let hash = Self::hash_long(key);
        (hash as usize) & CACHE_MASK
    }

    const fn hash_long(mut x: i64) -> i64 {
        x ^= x >> 32;
        x = x.wrapping_mul(0xd6e8_feb8_6659_fd93u64 as i64);
        x ^= x >> 32;
        x = x.wrapping_mul(0xd6e8_feb8_6659_fd93u64 as i64);
        x ^= x >> 32;
        x
    }
}

impl Default for PathTypeCache {
    fn default() -> Self {
        Self::new()
    }
}
