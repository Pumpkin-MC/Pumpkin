use pumpkin_data::damage::DamageType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::effect::{EffectFuture, MobEffect};
use crate::entity::living::LivingEntity;
use crate::entity::projectile::wind_charge::BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR;
use crate::world::explosion::ExplosionInteraction;

pub struct WindChargedMobEffect;

impl MobEffect for WindChargedMobEffect {
    fn on_mob_death<'a>(
        &'a self,
        living: &'a LivingEntity,
        _amplifier: u8,
        _damage_type: &'a DamageType,
    ) -> EffectFuture<'a, ()> {
        Box::pin(async move {
            let world = living.entity.world.load();
            let pos = living.entity.pos.load();
            let height = living.entity.height();
            let center = Vector3::new(pos.x, pos.y + f64::from(height) / 2.0, pos.z);

            // gustStrength = 3.0 + random * 2.0
            let gust_strength = 3.0 + rand::random::<f32>() * 2.0;

            world
                .explode_with_calculator(
                    center,
                    gust_strength,
                    ExplosionInteraction::Trigger,
                    Some(BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone()),
                )
                .await;

            world.play_sound(
                Sound::EntityBreezeWindBurst,
                SoundCategory::Neutral,
                &center,
            );
        })
    }
}
