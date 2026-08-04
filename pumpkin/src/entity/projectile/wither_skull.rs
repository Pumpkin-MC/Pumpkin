use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;

const EXPLOSION_POWER: f32 = 1.0;
const GRAVITY: f64 = 0.0;
const OWNER_DAMAGE: f32 = 8.0;
const NO_OWNER_DAMAGE: f32 = 5.0;
const OWNER_HEAL_ON_KILL: f32 = 5.0;
const WITHER_AMPLIFIER: u8 = 1;

/// `WitherSkull#onHitEntity` (WitherSkull.java): the Wither effect's duration is picked from
/// the world's `Difficulty`, not from the skull's `dangerous` flag. `dangerous` only changes
/// flight inertia and which blocks the resulting explosion can destroy (see `getInertia` /
/// `getBlockExplosionResistance`), neither of which affects this duration.
const fn wither_duration_ticks(difficulty: Difficulty) -> i32 {
    match difficulty {
        Difficulty::Normal => 20 * 10,
        Difficulty::Hard => 20 * 40,
        Difficulty::Peaceful | Difficulty::Easy => 0,
    }
}

pub struct WitherSkullEntity {
    pub thrown: ThrownItemEntity,
    /// Fired by charged Wither boss attacks. Data-only for now: the boss AI that sets this
    /// (and the inertia/block-resistance behavior it should drive) is out of scope here.
    pub dangerous: AtomicBool,
}

impl WitherSkullEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: GRAVITY,
        };

        Self {
            thrown,
            dangerous: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, dangerous: bool) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self {
            thrown,
            dangerous: AtomicBool::new(dangerous),
        }
    }

    pub fn is_dangerous(&self) -> bool {
        self.dangerous.load(Ordering::Relaxed)
    }

    pub fn set_dangerous(&self, value: bool) {
        self.dangerous.store(value, Ordering::Relaxed);
    }
}

impl NBTStorage for WitherSkullEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_bool("dangerous", self.is_dangerous());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.set_dangerous(nbt.get_bool("dangerous").unwrap_or(false));
        })
    }
}

impl EntityBase for WitherSkullEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.thrown.process_tick(caller, server).await })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();

            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let owner = self
                    .thrown
                    .owner_id
                    .and_then(|id| world.get_entity_by_id(id));
                let owner_living = owner.as_ref().and_then(|o| o.get_living_entity());

                let was_hurt = if let Some(owner_living) = owner_living {
                    let hurt = entity
                        .damage(self, OWNER_DAMAGE, DamageType::WITHER_SKULL)
                        .await;
                    if hurt && !entity.get_entity().is_alive() {
                        owner_living.heal(OWNER_HEAL_ON_KILL);
                    }
                    hurt
                } else {
                    entity
                        .damage(self, NO_OWNER_DAMAGE, DamageType::MAGIC)
                        .await
                };

                if was_hurt && let Some(living) = entity.get_living_entity() {
                    let difficulty = world.level_info.load().difficulty;
                    let duration = wither_duration_ticks(difficulty);
                    if duration > 0 {
                        let effect = Effect {
                            effect_type: &StatusEffect::WITHER,
                            duration,
                            amplifier: WITHER_AMPLIFIER,
                            ambient: false,
                            show_particles: true,
                            show_icon: true,
                            blend: true,
                        };
                        if let Some(player) = entity.get_player() {
                            player.send_effect(effect.clone()).await;
                        }
                        living.add_effect(effect).await;
                    }
                }
            }

            let hit_pos = hit.hit_pos();
            let mob_griefing = world.level_info.load().game_rules.mob_griefing;
            if mob_griefing {
                world.explode(hit_pos, EXPLOSION_POWER).await;
            } else {
                world.explode_without_blocks(hit_pos, EXPLOSION_POWER).await;
            }
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn wither_duration_scales_with_difficulty_not_dangerous() {
        assert_eq!(wither_duration_ticks(Difficulty::Peaceful), 0);
        assert_eq!(wither_duration_ticks(Difficulty::Easy), 0);
        assert_eq!(wither_duration_ticks(Difficulty::Normal), 200);
        assert_eq!(wither_duration_ticks(Difficulty::Hard), 800);
    }
}
