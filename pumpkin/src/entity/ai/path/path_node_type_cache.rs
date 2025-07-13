use std::sync::Arc;
use pumpkin_util::math::position::BlockPos;
use crate::entity::ai::path::land_path_node_maker::LandPathNodeMaker;
use crate::entity::ai::path::path_node::PathNodeType;
use crate::world::World;

const LONG_PHI: u64 = 0x9e3779b97f4a7c15;

#[derive(Debug, Clone)]
pub struct PathNodeTypeCache {
    pub positions: Box<[i64; 4096]>,
    pub cache: Box<[PathNodeType; 4096]>
}

impl PathNodeTypeCache {
    pub fn new() -> Self {
        Self {
            positions: Box::new([0; 4096]),
            cache: Box::new([PathNodeType::Walkable; 4096])
        }
    }

    pub fn add(&mut self, world: Arc<World>, pos: BlockPos) -> PathNodeType {
        let long_pos = pos.as_long();
        let hash = self.hash(long_pos) as usize;
        let path_node_type = self.get(hash, long_pos);
        if let Some(path_node_type) = path_node_type {
            path_node_type
        } else {
            self.compute(world, pos, hash, long_pos)
        }
    }

    fn get(&self, index: usize, pos: i64) -> Option<PathNodeType> {
        if self.positions[index] == pos {
            Some(self.cache[index])
        } else {
            None
        }
    }

    fn compute(&mut self, world: Arc<World>, pos: BlockPos, index: usize, long_pos: i64) -> PathNodeType {
        let path_node_type = LandPathNodeMaker::get_common_node_type(world, pos);
        self.positions[index] = long_pos;
        self.cache[index] = path_node_type;
        path_node_type
    }

    fn hash(&self, x: i64) -> i64 {
        // HashCommon
        let mut hash_common = (x as u64).wrapping_mul(LONG_PHI);
        hash_common ^= hash_common >> 32;
        let h = (hash_common ^ (hash_common >> 16)) as i64;
        h & 4095
    }
}