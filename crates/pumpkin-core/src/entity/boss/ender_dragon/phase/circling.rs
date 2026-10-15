use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext, find_path};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct CirclingPhase;

impl super::Phase for CirclingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::HoldingPattern
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
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

            let world = dragon.mob_entity.living_entity.entity.world.load();
            let alive_crystals = world
                .dragon_fight
                .as_ref()
                .map(|f| f.lock().unwrap_or_else(std::sync::PoisonError::into_inner).alive_crystals)
                .unwrap_or(0);

            // Check if current path completed to transition to landing or strafing
            if path.is_empty() {
                // 1. Landing approach probability: 1 in (crystals + 3)
                if rand::random_range(0..(alive_crystals + 3).max(1)) == 0 {
                    drop(path);
                    drop(target_location);
                    dragon.set_phase(EnderDragonPhase::LandingApproach);
                    return;
                }

                // 2. Strafe player probability
                if let Some(player) = dragon.find_nearest_player() {
                    let fight_origin = dragon
                        .fight_origin
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .0;
                    let egg_pos = Vector3::new(fight_origin.x as f64, fight_origin.y as f64, fight_origin.z as f64);
                    let p_pos = player.living_entity.entity.pos.load();
                    let dist_sq = egg_pos.distance_squared(p_pos);
                    let dist_chance = ((dist_sq / 512.0) as i32 + 2).max(1);

                    if rand::random_range(0..dist_chance) == 0
                        || rand::random_range(0..(alive_crystals + 2).max(1)) == 0
                    {
                        *dragon
                            .target_player
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) =
                            Some(player.gameprofile.id);
                        drop(path);
                        drop(target_location);
                        dragon.set_phase(EnderDragonPhase::StrafePlayer);
                        return;
                    }
                }
            }

            // Path generation for holding pattern
            if path.is_empty() {
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

                let j = if alive_crystals > 0 {
                    j.rem_euclid(12) as usize
                } else {
                    (j - 12).rem_euclid(8) as usize + 12
                };

                let nodes = dragon
                    .nodes
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *path = find_path(&nodes, i, j, None);
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
    }
}
