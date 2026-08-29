use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct HungerMobEffect;

impl MobEffect for HungerMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, _amplifier: u8) -> bool {
        if duration <= 0 {
            return false;
        }
        (duration as u32).is_multiple_of(20)
    }

    fn apply_effect_tick(&self, living: &LivingEntity, amplifier: u8) {
        let world = living.entity.world.load();
        if let Some(entity) = world.get_entity_by_id(living.entity.entity_id)
            && let Some(player) = entity.get_player()
        {
            let exhaustion = 0.1 * (f32::from(amplifier) + 1.0);
            player.hunger_manager.add_exhaustion(exhaustion);
        }
    }
}
