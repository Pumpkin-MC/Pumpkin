use std::sync::Mutex;

use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};

pub struct SittingScanningPhase {
    scanning_time: Mutex<i32>,
}

impl Default for SittingScanningPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl SittingScanningPhase {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scanning_time: Mutex::new(0),
        }
    }
}

impl super::Phase for SittingScanningPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingScanning
    }

    fn is_sitting(&self) -> bool {
        true
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .scanning_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut time_lock = self
            .scanning_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *time_lock += 1;
        let time = *time_lock;
        drop(time_lock);

        let entity = &dragon.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let dragon_pos = entity.pos.load();

        // 1. Scan for nearby players within 20 blocks horizontally and 10 blocks vertically
        let mut nearby_target = None;
        let mut min_dist_sq = 400.0; // 20 blocks

        for player in world.players.load().iter() {
            let p_pos = player.living_entity.entity.pos.load();
            let dx = p_pos.x - dragon_pos.x;
            let dy = (p_pos.y - dragon_pos.y).abs();
            let dz = p_pos.z - dragon_pos.z;
            let dist_horiz_sq = dx * dx + dz * dz;

            if dy <= 10.0 && dist_horiz_sq < min_dist_sq {
                min_dist_sq = dist_horiz_sq;
                nearby_target = Some(player.clone());
            }
        }

        if let Some(target) = nearby_target {
            if time > 25 {
                dragon.set_phase(EnderDragonPhase::SittingAttacking);
            } else {
                let target_pos = target.living_entity.entity.pos.load();
                let head_pos = dragon.parts[0].entity.pos.load();
                let x_attack_dist = target_pos.x - head_pos.x;
                let z_attack_dist = target_pos.z - head_pos.z;

                let yaw = entity.yaw.load();
                let mut y_rot_delta = (180.0
                    - (x_attack_dist.atan2(z_attack_dist).to_degrees() as f32)
                    - yaw)
                    .rem_euclid(360.0);
                if y_rot_delta > 180.0 {
                    y_rot_delta -= 360.0;
                }
                let y_rot_delta = y_rot_delta.clamp(-100.0, 100.0);

                let dist = (x_attack_dist.hypot(z_attack_dist) as f32 + 1.0).min(40.0);
                let rot_speed = dist;

                let mut yaw_accel = dragon
                    .yaw_rot_accel
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *yaw_accel *= 0.8;
                *yaw_accel += y_rot_delta * (0.7 / dist / rot_speed);
                entity.yaw.store(yaw + *yaw_accel);
            }
        } else if time >= 100 {
            // Check for players within 150 blocks to charge
            let mut charge_target = None;
            let mut min_charge_dist_sq = 22500.0; // 150 blocks

            for player in world.players.load().iter() {
                let p_pos = player.living_entity.entity.pos.load();
                let dist_sq = dragon_pos.distance_squared(p_pos);
                if dist_sq < min_charge_dist_sq {
                    min_charge_dist_sq = dist_sq;
                    charge_target = Some(p_pos);
                }
            }

            dragon.set_phase(EnderDragonPhase::Takeoff);
            if let Some(target_pos) = charge_target {
                dragon.set_phase(EnderDragonPhase::ChargingPlayer);
                *dragon
                    .target_location
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(target_pos);
            }
        }
    }
}
