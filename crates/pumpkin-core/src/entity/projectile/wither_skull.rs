use crate::entity::projectile::owner::ProjectileOwner;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::entity::boss::wither::WitherEntity;
use pumpkin_data::{Block, damage::DamageType, effect::StatusEffect, potion::Effect, tracked_data};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{Difficulty, math::vector3::Vector3};

use crate::{
    entity::{
        Entity, EntityBase,
        projectile::{HurtingProjectile, ProjectileHit, ThrownItemEntity},
    },
    server::Server,
    world::ExplosionInteraction,
};

/// A dangerous skull keeps less of its speed each tick.
const DANGEROUS_INERTIA: f64 = 0.73;

pub struct WitherSkullEntity {
    pub thrown: HurtingProjectile,
    pub dangerous: AtomicBool,
}

impl WitherSkullEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner: ProjectileOwner::none(),
            gravity: 0.0,
        };

        Self {
            thrown: HurtingProjectile::new(thrown, HurtingProjectile::DEFAULT_ACCELERATION_POWER),
            dangerous: AtomicBool::new(false),
        }
    }

    /// Keeps the position of `entity`, the wither places its skulls at the shooting head.
    #[must_use]
    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        dangerous: bool,
        direction: Vector3<f64>,
    ) -> Self {
        let thrown = HurtingProjectile::new(
            ThrownItemEntity::owned(entity, shooter, 0.0),
            HurtingProjectile::DEFAULT_ACCELERATION_POWER,
        );
        thrown.assign_directional_movement(direction);
        Self {
            thrown,
            dangerous: AtomicBool::new(dangerous),
        }
    }

    #[must_use]
    pub fn is_dangerous(&self) -> bool {
        self.dangerous.load(Ordering::Relaxed)
    }

    pub fn set_dangerous(&self, dangerous: bool) {
        self.dangerous.store(dangerous, Ordering::Relaxed);
        self.thrown
            .entity
            .set_synced_data(tracked_data::wither_skull::DATA_DANGEROUS, dangerous);
    }
}

impl EntityBase for WitherSkullEntity {
    fn get_block_explosion_resistance(&self, block: &Block, resistance: f32) -> f32 {
        if self.is_dangerous() && WitherEntity::can_destroy(block) {
            resistance.min(0.8)
        } else {
            resistance
        }
    }

    fn get_projectile_owner(&self) -> Option<&ProjectileOwner> {
        Some(&self.thrown.owner)
    }

    fn on_deflection(&self, by_attack: bool) {
        self.thrown.on_deflection(by_attack);
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        self.thrown.write_nbt(nbt);
        nbt.put_bool("dangerous", self.is_dangerous());
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        self.thrown.read_nbt(nbt);
        if let Some(dangerous) = nbt.get_bool("dangerous") {
            self.dangerous.store(dangerous, Ordering::Relaxed);
        }
    }

    fn init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::wither_skull::DATA_DANGEROUS,
            self.is_dangerous(),
        );
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let inertia = if self.is_dangerous() {
            DANGEROUS_INERTIA
        } else {
            HurtingProjectile::INERTIA
        };
        self.thrown.process_tick(
            caller,
            server,
            inertia,
            HurtingProjectile::LIQUID_INERTIA,
            false,
            |_| true,
        );
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) {
        let world = self.get_entity().world.load();

        if let ProjectileHit::Entity {
            ref entity,
            hit_pos,
            ..
        } = hit
        {
            let owner = self.thrown.owner.get(&world);
            let hurt = if let Some(owner) = owner.as_ref()
                && let Some(living_owner) = owner.get_living_entity()
            {
                let hurt = entity.damage_with_context(
                    entity.as_ref(),
                    8.0,
                    DamageType::WITHER_SKULL,
                    Some(hit_pos),
                    Some(self),
                    Some(owner.as_ref()),
                );
                if hurt && !entity.get_entity().is_alive() {
                    living_owner.heal(5.0);
                }
                hurt
            } else {
                entity.damage(entity.as_ref(), 5.0, DamageType::MAGIC)
            };

            let duration_seconds = match world.level_info.load().difficulty {
                Difficulty::Normal => 10,
                Difficulty::Hard => 40,
                Difficulty::Easy | Difficulty::Peaceful => 0,
            };
            if hurt
                && duration_seconds > 0
                && let Some(living) = entity.get_living_entity()
            {
                let effect = Effect {
                    effect_type: &StatusEffect::WITHER,
                    duration: 20 * duration_seconds,
                    amplifier: 1,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: true,
                };
                let effect_source: &dyn EntityBase = owner.as_deref().unwrap_or(self);
                if let Some(player) = entity.get_player() {
                    player.add_effect_with_source(effect, effect_source);
                } else {
                    living.add_effect_with_source(effect, Some(effect_source));
                }
            }
        }

        world.explode_from(
            self,
            self.get_entity().pos.load(),
            1.0,
            ExplosionInteraction::Mob,
        );
    }
}
