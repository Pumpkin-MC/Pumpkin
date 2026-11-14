use pumpkin_data::damage::DamageType;
use pumpkin_data::enchantment::LevelBasedValue;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::vector3::Vector3;

use crate::world::ExplosionInteraction;

/// Enchantment entity effect that produces an explosion at an offset position.
#[derive(Clone, Debug, PartialEq)]
pub struct ExplodeEffect {
    pub attribute_to_user: bool,
    pub damage_type: Option<&'static DamageType>,
    pub knockback_multiplier: Option<LevelBasedValue>,
    pub immune_blocks: Option<&'static str>,
    pub offset: Vector3<f64>,
    pub radius: LevelBasedValue,
    pub create_fire: bool,
    pub block_interaction: ExplosionInteraction,
    pub small_particle: Option<Particle>,
    pub large_particle: Option<Particle>,
    pub sound: Option<Sound>,
}

impl ExplodeEffect {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        attribute_to_user: bool,
        damage_type: Option<&'static DamageType>,
        knockback_multiplier: Option<LevelBasedValue>,
        immune_blocks: Option<&'static str>,
        offset: Vector3<f64>,
        radius: LevelBasedValue,
        create_fire: bool,
        block_interaction: ExplosionInteraction,
        small_particle: Option<Particle>,
        large_particle: Option<Particle>,
        sound: Option<Sound>,
    ) -> Self {
        Self {
            attribute_to_user,
            damage_type,
            knockback_multiplier,
            immune_blocks,
            offset,
            radius,
            create_fire,
            block_interaction,
            small_particle,
            large_particle,
            sound,
        }
    }

    #[must_use]
    pub fn calculate_radius(&self, level: i32) -> f32 {
        self.radius.calculate(level).max(0.0)
    }

    #[must_use]
    pub fn calculate_knockback(&self, level: i32) -> Option<f32> {
        self.knockback_multiplier
            .as_ref()
            .map(|kb| kb.calculate(level))
    }
}

impl super::EnchantmentEntityEffectExt for ExplodeEffect {
    fn apply(
        &self,
        world: &std::sync::Arc<crate::world::World>,
        enchantment_level: i32,
        owner: Option<&std::sync::Arc<crate::entity::player::Player>>,
        entity: Option<&crate::entity::Entity>,
        position: Vector3<f64>,
    ) {
        let r = self.calculate_radius(enchantment_level);
        let center = position + self.offset;
        let immune_blocks = self.immune_blocks.and_then(|tag| {
            pumpkin_data::tag::get_latest_map(pumpkin_data::tag::RegistryKey::Block)
                .get(tag.strip_prefix('#').unwrap_or(tag))
                .copied()
        });
        let calculator = std::sync::Arc::new(crate::world::SimpleExplosionDamageCalculator::new(
            self.block_interaction != ExplosionInteraction::None,
            self.damage_type.is_some(),
            self.calculate_knockback(enchantment_level),
            immune_blocks,
        ));
        let source_entity_id = if self.attribute_to_user {
            entity
                .map(|e| e.entity_id)
                .or_else(|| owner.map(|o| o.entity_id()))
        } else {
            None
        };
        let mut explosion = crate::world::Explosion::new(
            r,
            center,
            world.get_block_interaction(self.block_interaction),
        )
        .with_damage_calculator(calculator)
        .with_particles_and_sound(
            self.small_particle.unwrap_or(Particle::Explosion),
            self.large_particle.unwrap_or(Particle::ExplosionEmitter),
            self.sound.unwrap_or(Sound::EntityGenericExplode),
        );
        if let Some(damage_type) = self.damage_type {
            explosion = explosion.with_damage_type(*damage_type);
        }
        if let Some(source_id) = source_entity_id {
            explosion = explosion.with_source_entity_id(source_id);
        }
        world.run_explosion(&explosion, center, r);
    }
}
