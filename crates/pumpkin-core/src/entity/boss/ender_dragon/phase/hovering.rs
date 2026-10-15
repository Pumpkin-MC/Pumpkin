use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;

pub struct HoveringPhase;

impl super::Phase for HoveringPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Hovering
    }

    fn is_sitting(&self) -> bool {
        true
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        let pos = dragon.mob_entity.living_entity.entity.pos.load();
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(pos);
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        if rand::random_bool(0.01) {
            dragon.set_phase(EnderDragonPhase::Takeoff);
            return;
        }

        let mut target_lock = dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if target_lock.is_none() {
            let pos = dragon.mob_entity.living_entity.entity.pos.load();
            *target_lock = Some(pos);
        }
    }

    fn get_fly_speed(&self) -> f32 {
        1.0
    }
}
