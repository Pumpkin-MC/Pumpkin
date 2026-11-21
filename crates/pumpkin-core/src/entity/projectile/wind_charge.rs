use crate::entity::projectile::owner::ProjectileOwner;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{VarInt, java::client::play::CProjectilePower};
use pumpkin_util::math::vector3::Vector3;
use std::sync::LazyLock;
use std::{
    f64,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use crate::{
    entity::{
        Entity, EntityBase,
        living::LivingEntity,
        projectile::{HurtingProjectile, ProjectileHit, ThrownItemEntity},
    },
    server::Server,
    world::SimpleExplosionDamageCalculator,
};

const DEFAULT_DEFLECT_COOLDOWN: u8 = 5;
const INITIAL_ACCELERATION_POWER: f64 = 0.0;
pub const WIND_CHARGE_GRAVITY: f64 = 0.0;

enum WindChargeKind {
    Normal { deflect_cooldown: AtomicU8 },
    Breeze,
}

pub struct WindChargeEntity {
    kind: WindChargeKind,
    thrown_item_entity: HurtingProjectile,
}

/// Wind charges keep their speed, in air and in water.
const INERTIA: f64 = 1.0;
/// How far above the build limit a wind charge bursts on its own.
const MAX_HEIGHT_ABOVE_WORLD: i32 = 30;

pub static WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR: LazyLock<Arc<SimpleExplosionDamageCalculator>> =
    LazyLock::new(|| {
        Arc::new(SimpleExplosionDamageCalculator::new(
            true,
            false,
            Some(1.22),
            Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
        ))
    });

pub static BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR: LazyLock<
    Arc<SimpleExplosionDamageCalculator>,
> = LazyLock::new(|| {
    Arc::new(SimpleExplosionDamageCalculator::new(
        true,
        false,
        None,
        Some(&tag::Block::MINECRAFT_BLOCKS_WIND_CHARGE_EXPLOSIONS),
    ))
});

impl WindChargeEntity {
    pub(crate) fn acceleration_power(&self) -> f64 {
        self.thrown_item_entity.acceleration_power()
    }

    #[must_use]
    pub const fn new_normal(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Normal {
                deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            },
            thrown_item_entity: HurtingProjectile::new(
                thrown_item_entity,
                INITIAL_ACCELERATION_POWER,
            ),
        }
    }

    #[must_use]
    pub const fn new_breeze(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Breeze,
            thrown_item_entity: HurtingProjectile::new(
                thrown_item_entity,
                INITIAL_ACCELERATION_POWER,
            ),
        }
    }

    pub const fn deflect_cooldown(&self) -> Option<&AtomicU8> {
        if let WindChargeKind::Normal {
            deflect_cooldown, ..
        } = &self.kind
        {
            Some(deflect_cooldown)
        } else {
            None
        }
    }

    pub fn create_explosion(&self, position: Vector3<f64>) {
        let world = self.get_entity().world.load();
        let (power, calculator, sound, is_normal) = match self.kind {
            WindChargeKind::Normal { .. } => (
                1.2,
                WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone(),
                pumpkin_data::sound::Sound::EntityWindChargeWindBurst,
                true,
            ),
            WindChargeKind::Breeze => (
                3.0,
                BREEZE_WIND_CHARGE_EXPLOSION_DAMAGE_CALCULATOR.clone(),
                pumpkin_data::sound::Sound::EntityBreezeWindBurst,
                false,
            ),
        };
        let mut explosion = crate::world::Explosion::new(
            power,
            position,
            world.get_block_interaction(crate::world::ExplosionInteraction::Trigger),
        )
        .with_source(self)
        .with_damage_calculator(calculator)
        .with_particles_and_sound(
            pumpkin_data::particle::Particle::GustEmitterSmall,
            pumpkin_data::particle::Particle::GustEmitterLarge,
            sound,
        );
        if is_normal {
            explosion = explosion.from_wind_charge();
        }
        world.run_explosion(&explosion);
    }
}

impl EntityBase for WindChargeEntity {
    fn push_from_explosion(&self, _knockback: Vector3<f64>) {
        // AbstractWindCharge ignores the push but still takes part in owner redirection.
    }

    fn clear_motion_on_chunk_load(&self) {
        // Charges resume flight with their saved motion.
    }

    fn can_hit(&self) -> bool {
        super::is_redirectable(self.get_entity())
    }

    fn can_be_deflected(&self) -> bool {
        self.deflect_cooldown()
            .is_none_or(|cooldown| cooldown.load(Ordering::Relaxed) == 0)
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown_item_entity.owner)
    }

    fn java_spawn_data(&self) -> VarInt {
        VarInt(
            self.get_owner_entity()
                .map_or(0, |owner| owner.get_entity().entity_id),
        )
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.thrown_item_entity.write_nbt(nbt);
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        // Older Pumpkin saves omitted the field and loaded charges with zero acceleration.
        self.thrown_item_entity
            .read_nbt_with_default(nbt, INITIAL_ACCELERATION_POWER);
    }

    fn on_deflection(&self, by_attack: bool) {
        self.thrown_item_entity.on_deflection(by_attack);
        let entity = self.get_entity();
        let power = self.thrown_item_entity.acceleration_power();
        entity.world.load().send_to_tracking_players(
            entity,
            &CProjectilePower::new(VarInt(entity.entity_id), power, 0.0, 0.0),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let entity = self.get_entity();
        if entity.block_pos.load().0.y > entity.world.load().get_top_y() + MAX_HEIGHT_ABOVE_WORLD {
            self.create_explosion(entity.pos.load());
            entity.remove();
            return;
        }
        self.thrown_item_entity
            .process_tick(caller, server, INERTIA, INERTIA, false, |other| {
                let other_type = other.get_entity().entity_type;
                other_type != &EntityType::WIND_CHARGE
                    && other_type != &EntityType::BREEZE_WIND_CHARGE
                    && other_type != &EntityType::END_CRYSTAL
            });

        if let Some(cooldown) = self.deflect_cooldown() {
            let cooldown_ticks = cooldown.load(Ordering::Relaxed);
            if cooldown_ticks > 0 {
                cooldown.store(cooldown_ticks - 1, Ordering::Relaxed);
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.thrown_item_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        match hit {
            ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } => {
                let world = self.get_entity().world.load();
                let owner = self
                    .thrown_item_entity
                    .owner
                    .get(&world)
                    .filter(|owner| owner.get_living_entity().is_some());
                if let Some(owner) = owner.as_deref().and_then(EntityBase::get_living_entity) {
                    owner.set_last_hurt_mob(entity.as_ref());
                }
                entity.damage_with_context(
                    entity.as_ref(),
                    1.0,
                    DamageType::WIND_CHARGE,
                    Some(hit_pos),
                    Some(self),
                    owner.as_deref(),
                );
                self.create_explosion(self.get_entity().pos.load());
            }
            ProjectileHit::Block { face, hit_pos, .. } => {
                // Bursts a little in front of the face it hit.
                let normal = face.to_offset().to_f64() * 0.25;
                self.create_explosion(hit_pos + normal);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{ai::test_support, r#type::load_entity};

    #[tokio::test]
    async fn old_wind_charges_load_without_acceleration_and_keep_explicit_values()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let position = Vector3::new(0.0, 64.0, 0.0);

        for (entity_type, default_power) in [
            (&EntityType::WIND_CHARGE, 0.0),
            (&EntityType::BREEZE_WIND_CHARGE, 0.0),
            (&EntityType::FIREBALL, 0.1),
        ] {
            for saved_power in [None, Some(0.0), Some(0.05), Some(0.1)] {
                let mut nbt = NbtCompound::new();
                if let Some(power) = saved_power {
                    nbt.put_double("acceleration_power", power);
                }
                let entity = load_entity(entity_type, position, &nbt, &world);
                let mut output = NbtCompound::new();
                entity.write_custom_nbt(&mut output);
                assert_eq!(
                    output.get_double("acceleration_power").map(f64::to_bits),
                    Some(saved_power.unwrap_or(default_power).to_bits()),
                    "{} with saved acceleration {saved_power:?}",
                    entity_type.resource_name,
                );
            }
        }

        world.level.shutdown().await;
        Ok(())
    }
}
