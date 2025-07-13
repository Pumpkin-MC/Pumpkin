use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64};
use crossbeam::atomic::AtomicCell;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path::Path;

const RECALCULATE_COOLDOWN: i32 = 20;

pub struct EntityNavigation {
    pub current_path: Option<Path>,
    pub speed: AtomicCell<f64>,
    pub tick_count: AtomicI32,
    pub path_start_time: AtomicI32,
    pub path_start_pos: AtomicCell<Vector3<f64>>,
    pub last_node_position: AtomicCell<Vector3<f64>>,
    pub current_node_ms: AtomicI64,
    pub current_node_timeout: AtomicCell<f64>,
    pub node_reach_proximity: AtomicCell<f32>, // Default 0.5
    pub is_recalculation_cooldown: AtomicBool,
    pub last_recalculation_time: AtomicI64,
    
}

pub trait EntityNavigationTrait {

}