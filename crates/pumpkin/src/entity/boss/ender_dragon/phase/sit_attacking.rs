use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;

pub struct SitAttackingPhase;

impl super::Phase for SitAttackingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SitAttacking
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut timer = dragon
            .sit_attack_timer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *timer += 1;

        if *timer > 40 {
            *timer = 0;
            let should_breathe = rand::random_bool(0.5);
            let should_take_off = *dragon
                .ticks_sitting
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                > 200;
            drop(timer);

            if should_breathe {
                dragon.set_phase(EnderDragonPhase::SitBreathing);
                *dragon
                    .breathing_timer
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
            } else if should_take_off {
                dragon.set_phase(EnderDragonPhase::TakingOff);
            }
        } else {
            drop(timer);
        }

        let mut dmg = dragon
            .sitting_damage_received
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *dmg > 150.0 {
            *dmg = 0.0;
            drop(dmg);
            dragon.set_phase(EnderDragonPhase::TakingOff);
        }
    }
}
