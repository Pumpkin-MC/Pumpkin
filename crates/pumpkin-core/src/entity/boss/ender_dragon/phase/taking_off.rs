use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, NODE_Y, Vector3Ext, find_path};
use pumpkin_util::math::vector3::Vector3;

pub struct TakingOffPhase;

impl super::Phase for TakingOffPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Takeoff
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        let current_node = dragon.find_closest_node();
        let target_node = (current_node + 6) % 24;

        let nodes = dragon
            .nodes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut path = dragon
            .path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *path = find_path(&nodes, current_node, target_node, None);

        let origin = dragon
            .fight_origin
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .0;
        let climb_target = Vector3::new(origin.x as f64, NODE_Y as f64, origin.z as f64);

        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(climb_target);
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let fight_origin = dragon
            .fight_origin
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .0;
        let podium_pos = Vector3::new(fight_origin.x as f64, fight_origin.y as f64, fight_origin.z as f64);
        let pos = dragon.mob_entity.living_entity.entity.pos.load();

        if pos.distance_squared(podium_pos) > 100.0 {
            dragon.set_phase(EnderDragonPhase::HoldingPattern);
        }
    }
}
