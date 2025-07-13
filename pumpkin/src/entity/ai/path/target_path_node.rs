use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use crossbeam::atomic::AtomicCell;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path_node::PathNode;

pub struct TargetPathNode {
    pub path_node: PathNode,
    pub nearest_node_distance: AtomicCell<f32>,
    pub nearest_node: AtomicCell<Option<PathNode>>,
    pub reached: AtomicBool,
}

impl TargetPathNode {
    #[must_use]
    pub fn new(location: Vector3<f64>) -> Self {
        Self {
            path_node: PathNode::new(location),
            nearest_node_distance: AtomicCell::new(0.0),
            nearest_node: AtomicCell::new(None),
            reached: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn from_path_node(node: &PathNode) -> Self {
        Self::new(node.location)
    }

    pub fn update_nearest_node(&self, distance: f32, node: PathNode) {
        if distance < self.nearest_node_distance.load() {
            self.nearest_node_distance.store(distance);
            self.nearest_node.store(Some(node));
        }
    }

    pub fn mark_reached(&self) {
        self.reached.store(true, Relaxed);
    }
}