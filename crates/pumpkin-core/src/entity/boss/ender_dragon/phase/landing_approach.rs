use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{DragonNode, EnderDragonEntity, Vector3Ext, find_path};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct LandingApproachPhase;

impl super::Phase for LandingApproachPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::LandingApproach
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        dragon
            .path
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let pos = dragon.mob_entity.living_entity.entity.pos.load();
        let mut target_location = dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let dist_to_target = target_location.map_or(0.0, |loc| pos.distance_squared(loc));
        if target_location.is_none()
            || !(100.0..=22500.0).contains(&dist_to_target)
            || dragon
                .mob_entity
                .living_entity
                .entity
                .horizontal_collision
                .load(Ordering::Relaxed)
        {
            let mut path = dragon
                .path
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            if path.is_empty() {
                drop(path);
                let current_node = dragon.find_closest_node();
                let fight_origin = *dragon
                    .fight_origin
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let egg = fight_origin.0;

                let nearest_player = dragon.find_nearest_player();
                let target_node_idx = if let Some(player) = nearest_player {
                    let p_pos = player.living_entity.entity.pos.load();
                    let mut aim = Vector3::new(p_pos.x, 0.0, p_pos.z);
                    if aim.length_squared() > 1e-6 {
                        aim = aim.normalize();
                    }
                    dragon.find_closest_node_to(Vector3::new(-aim.x * 40.0, 105.0, -aim.z * 40.0))
                } else {
                    dragon.find_closest_node_to(Vector3::new(40.0, egg.y as f64, 0.0))
                };

                let final_node = DragonNode::new(egg.x as f64, egg.y as f64, egg.z as f64);
                let nodes = dragon
                    .nodes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut path_lock = dragon
                    .path
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *path_lock = find_path(&nodes, current_node, target_node_idx, Some(final_node));
                drop(nodes);
                path = path_lock;
            }

            if let Some(next_node) = path.first().copied() {
                path.remove(0);
                let nodes = dragon
                    .nodes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(node) = nodes[next_node] {
                    let y_target = node.y + rand::random_range(0.0..20.0);
                    *target_location = Some(Vector3::new(node.x, y_target, node.z));
                }
            }

            if path.is_empty() {
                drop(path);
                drop(target_location);
                dragon.set_phase(EnderDragonPhase::Landing);
                return;
            }
        }
    }
}
