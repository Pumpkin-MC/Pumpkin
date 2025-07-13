use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path_node::PathNode;
use crate::entity::EntityBase;

#[derive(Debug)]
pub struct Path {
    pub nodes: Vec<PathNode>,
    current_node_index: AtomicUsize,
    pub target: BlockPos,
    pub manhattan_distance_from_target: f64,
    pub reaches_target: bool,
}

impl Path {

    #[must_use]
    pub fn new(nodes: Vec<PathNode>, target: BlockPos, reaches_target: bool) -> Self {
        let manhattan_distance_from_target = nodes.last().map_or(f64::MAX, |path| path.get_manhattan_distance_to_pos(target));
        Self {
            nodes,
            current_node_index: AtomicUsize::new(0),
            target,
            manhattan_distance_from_target,
            reaches_target,
        }
    }

    pub fn next(&self) {
        self.current_node_index.fetch_add(1, Relaxed);
    }

    pub fn is_start(&self) -> bool {
        self.current_node_index.load(Relaxed) <= 0
    }

    pub fn is_finished(&self) -> bool {
        self.current_node_index.load(Relaxed) >= self.nodes.len()
    }

    pub fn set_length(&mut self, length: usize) {
        self.nodes.truncate(length);
    }

    pub fn set_current_node_index(&mut self, index: usize) {
        self.current_node_index.store(index, Relaxed);
    }

    pub fn get_node_position_of_entity_from_index(&self, entity: &dyn EntityBase, index: usize) -> Vector3<f64> {
        let node = &self.nodes[index];
        let x = node.location.x + (entity.get_entity().width() as f64 + 1.0) * 0.5;
        let y = node.location.y;
        let z = node.location.z + (entity.get_entity().width() as f64 + 1.0) * 0.5;
        Vector3::new(x, y, z)
    }

    pub fn get_node_position_of_entity(&self, entity: &dyn EntityBase) -> Vector3<f64> {
        self.get_node_position_of_entity_from_index(entity, self.current_node_index.load(Relaxed))
    }

    pub fn copy(&self) -> Path {
        Self {
            nodes: self.nodes.clone(),
            current_node_index: AtomicUsize::new(self.current_node_index.load(Relaxed)),
            target: self.target.clone(),
            manhattan_distance_from_target: self.manhattan_distance_from_target,
            reaches_target: self.reaches_target,
        }
    }
}