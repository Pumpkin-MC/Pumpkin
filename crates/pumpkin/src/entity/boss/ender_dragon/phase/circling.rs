use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext, find_path};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct CirclingPhase;

impl super::Phase for CirclingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Circling
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let pos = dragon.mob_entity.living_entity.entity.pos.load();
        let mut target_location = dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let d0 = target_location.map_or(0.0, |loc| pos.distance_squared(loc));

        if target_location.is_none()
            || !(100.0..=22500.0).contains(&d0)
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
                let i = dragon.find_closest_node();
                let mut j = i as i32;

                let mut clockwise = dragon
                    .holding_pattern_clockwise
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if rand::random_range(0..8) == 0 {
                    *clockwise = !*clockwise;
                    j = i as i32 + 6;
                }
                if *clockwise {
                    j += 1;
                } else {
                    j -= 1;
                }
                drop(clockwise);

                let world = dragon.mob_entity.living_entity.entity.world.load();
                let crystals_alive = world.dragon_fight.as_ref().is_some_and(|fight| {
                    fight
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .alive_crystals()
                        > 0
                });

                let j = if crystals_alive {
                    j.rem_euclid(12) as usize
                } else {
                    (j - 12).rem_euclid(8) as usize + 12
                };

                let mut path_lock = dragon
                    .path
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let nodes = dragon
                    .nodes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *path_lock = find_path(&nodes, i, j, None);
                drop(nodes);
                path = path_lock;
            }

            if let Some(next_node_idx) = path.first().copied() {
                path.remove(0);
                let nodes = dragon
                    .nodes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Some(node) = nodes[next_node_idx] {
                    let mut y_target = node.y + rand::random_range(0.0..20.0);
                    while y_target < node.y {
                        y_target = node.y + rand::random_range(0.0..20.0);
                    }
                    *target_location = Some(Vector3::new(node.x, y_target, node.z));
                }
            }
        }
        drop(target_location);

        if rand::random_range(0..64) == 0 {
            if rand::random_bool(0.5) {
                dragon.set_phase(EnderDragonPhase::FlyToPortal);
            } else if let Some(player) = dragon.find_nearest_player() {
                *dragon
                    .target_player
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some(player.gameprofile.id);
                dragon.set_phase(EnderDragonPhase::Strafing);
            }
        }
    }
}
