use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{EnderDragonEntity, Vector3Ext};
use pumpkin_util::math::vector3::Vector3;

pub struct LandingApproachPhase;

impl super::Phase for LandingApproachPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::LandingApproach
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let origin = {
            let guard = dragon
                .fight_origin
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.0
        };
        let target = Vector3::new(origin.x as f64, origin.y as f64, origin.z as f64);
        let pos = dragon.mob_entity.living_entity.entity.pos.load();

        if pos.distance_squared(target) < 1.0 {
            dragon.set_phase(EnderDragonPhase::Landing);
            return;
        }

        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(target);
    }
}
