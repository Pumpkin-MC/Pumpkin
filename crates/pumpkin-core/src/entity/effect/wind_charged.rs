use pumpkin_data::damage::DamageType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::effect::MobEffect;
use crate::entity::living::LivingEntity;
use crate::entity::projectile::wind_charge::BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR;
use crate::world::explosion::{Explosion, ExplosionInteraction};

pub struct WindChargedMobEffect;

impl MobEffect for WindChargedMobEffect {
    fn on_mob_death(&self, living: &LivingEntity, _amplifier: u8, _damage_type: &DamageType) {
        let world = living.entity.world.load();
        let pos = living.entity.pos.load();
        let height = living.entity.height();
        let center = Vector3::new(pos.x, pos.y + f64::from(height) / 2.0, pos.z);

        // gustStrength = 3.0 + random * 2.0
        let gust_strength = 3.0 + rand::random::<f32>() * 2.0;

        let explosion = Explosion::new(
            gust_strength,
            center,
            world.get_block_interaction(ExplosionInteraction::Trigger),
        )
        .with_source(living)
        .with_damage_calculator(BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone())
        .with_particles_and_sound(
            Particle::GustEmitterSmall,
            Particle::GustEmitterLarge,
            Sound::EntityBreezeWindBurst,
        );
        world.run_explosion(&explosion);
    }
}
