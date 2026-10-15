use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};
use pumpkin_util::math::vector3::Vector3;

pub struct LandingPhase;

impl super::Phase for LandingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Landing
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        let fight_origin = *dragon
            .fight_origin
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let egg = fight_origin.0;
        let target = Vector3::new(egg.x as f64 + 0.5, egg.y as f64, egg.z as f64 + 0.5);

        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(target);
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let pos = dragon.mob_entity.living_entity.entity.pos.load();
        let target_location = *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(target) = target_location {
            if pos.distance_squared(target) < 1.0 {
                dragon.reset_flame_count();
                dragon.set_phase(EnderDragonPhase::SittingScanning);
            }
        }
    }

    fn get_fly_speed(&self) -> f32 {
        1.5
    }
}
