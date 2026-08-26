use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;

pub struct RegenerationMobEffect;

impl MobEffect for RegenerationMobEffect {
    fn should_apply_effect_tick(&self, duration: i32, amplifier: u8) -> bool {
        if duration <= 0 {
            return false;
        }
        let tick_rate = 50 >> amplifier.min(4);
        if tick_rate > 0 {
            (duration as u32).is_multiple_of(tick_rate as u32)
        } else {
            true
        }
    }

    fn apply_effect_tick(&self, living: &LivingEntity, _amplifier: u8) {
        let current_health = living.health.load();
        let max_health = living.get_max_health();
        if current_health < max_health && current_health > 0.0 {
            living.heal(1.0);
        }
    }
}
