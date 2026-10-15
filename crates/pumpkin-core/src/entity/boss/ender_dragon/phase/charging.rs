use std::sync::Mutex;
use std::sync::atomic::Ordering;

use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};

pub struct ChargingPhase {
    time_since_charge: Mutex<i32>,
}

impl Default for ChargingPhase {
    fn default() -> Self {
        Self::new()
    }
}

impl ChargingPhase {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            time_since_charge: Mutex::new(0),
        }
    }
}

impl super::Phase for ChargingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::ChargingPlayer
    }

    fn begin(&self, _dragon: &EnderDragonEntity) {
        *self
            .time_since_charge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut time_lock = self
            .time_since_charge
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let target_location = *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(target) = target_location else {
            drop(time_lock);
            dragon.set_phase(EnderDragonPhase::HoldingPattern);
            return;
        };

        if *time_lock > 0 {
            *time_lock += 1;
            if *time_lock >= 10 {
                drop(time_lock);
                dragon.set_phase(EnderDragonPhase::HoldingPattern);
                return;
            }
        } else {
            let pos = dragon.mob_entity.living_entity.entity.pos.load();
            let dist_sq = pos.distance_squared(target);
            if dist_sq < 100.0
                || dist_sq > 22500.0
                || dragon
                    .mob_entity
                    .living_entity
                    .entity
                    .horizontal_collision
                    .load(Ordering::Relaxed)
            {
                *time_lock += 1;
            }
        }
    }

    fn get_fly_speed(&self) -> f32 {
        3.0
    }
}
