use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::EnderDragonEntity;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct SitAttackingPhase;

impl super::Phase for SitAttackingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::SittingAttacking
    }

    fn is_sitting(&self) -> bool {
        true
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *dragon
            .sit_attack_timer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut timer = dragon
            .sit_attack_timer
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *timer += 1;
        let t = *timer;
        drop(timer);

        if t == 1 {
            let entity = &dragon.mob_entity.living_entity.entity;
            let pos = entity.pos.load();
            let world = entity.world.load();
            world.play_sound(Sound::EntityEnderDragonGrowl, SoundCategory::Hostile, &pos);
        }

        if t >= 40 {
            dragon.set_phase(EnderDragonPhase::SittingFlaming);
        }
    }
}
