use crate::entity::effect::{EffectFuture, MobEffect};
use crate::entity::living::LivingEntity;

pub struct SaturationMobEffect;

impl MobEffect for SaturationMobEffect {
    fn should_apply_effect_tick(&self, _duration: i32, _amplifier: u8) -> bool {
        true
    }

    fn apply_effect_tick<'a>(
        &'a self,
        living: &'a LivingEntity,
        amplifier: u8,
    ) -> EffectFuture<'a, ()> {
        Box::pin(async move {
            let world = living.entity.world.load();
            if let Some(entity) = world.get_entity_by_id(living.entity.entity_id)
                && let Some(player) = entity.get_player()
            {
                let hunger = amplifier + 1;
                player.hunger_manager.add_hunger(hunger);
                player
                    .hunger_manager
                    .add_saturation(f32::from(hunger) * 2.0);
            }
        })
    }
}
