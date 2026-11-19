use crate::entity::projectile::owner::ProjectileOwner;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag;
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
    #[must_use]
    pub const fn new_normal(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Normal {
                deflect_cooldown: AtomicU8::new(DEFAULT_DEFLECT_COOLDOWN),
            },
            thrown_item_entity: HurtingProjectile::new(thrown_item_entity, 0.0),
        }
    }

    #[must_use]
    pub const fn new_breeze(thrown_item_entity: ThrownItemEntity) -> Self {
        Self {
            kind: WindChargeKind::Breeze,
            thrown_item_entity: HurtingProjectile::new(thrown_item_entity, 0.0),
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

    fn on_deflection(&self, by_attack: bool) {
        self.thrown_item_entity.on_deflection(by_attack);
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
