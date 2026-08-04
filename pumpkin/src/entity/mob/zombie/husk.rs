use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::zombie::{ZombieEntityBase, zombie::ZombieEntity};
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    mob::{Mob, MobEntity},
};

/// `Zombie::inWaterTime` threshold (`Zombie.java` `tick`): ticks the eyes must be submerged
/// before underwater conversion starts.
const WATER_TICKS_TO_START_CONVERSION: i32 = 600;
/// `Zombie::startUnderWaterConversion`'s fixed countdown (`Zombie.java` `tick`,
/// `startUnderWaterConversion(300)`).
const CONVERSION_TICKS: i32 = 300;

pub struct HuskEntity {
    entity: Arc<ZombieEntityBase>,
    /// Vanilla `Zombie::inWaterTime`. `Entity::touching_water` stands in for
    /// `isEyeInFluid(FluidTags.WATER)` -- Pumpkin has no per-fluid eye-submersion check, so this
    /// starts counting slightly earlier than vanilla (as soon as any part of the husk is in
    /// water, not just its eyes).
    in_water_time: AtomicI32,
    /// Vanilla `Zombie::conversionTime`. `-1` while not converting, matching the
    /// `DrownedConversionTime` NBT sentinel `Zombie::readAdditionalSaveData` uses.
    conversion_time: AtomicI32,
}

impl HuskEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let husk = Self {
            entity,
            in_water_time: AtomicI32::new(0),
            conversion_time: AtomicI32::new(-1),
        };
        Arc::new(husk)
    }

    /// Vanilla `Zombie::doUnderWaterConversion` + `Husk::doUnderWaterConversion`: replaces this
    /// husk with a plain zombie at the same position. Mirrors
    /// `ZombieVillagerEntity::finish_conversion`'s simplified copy set (position/velocity/
    /// rotation/age/custom name/active effects) -- Pumpkin has no generic `Mob::convertTo`
    /// (equipment/leash/passenger transfer), so those are not carried over.
    async fn finish_conversion(&self) {
        let old_entity = self.get_entity();
        let world = old_entity.world.load().clone();
        let pos = old_entity.pos.load();

        let new_entity = Entity::new(world.clone(), pos, &EntityType::ZOMBIE);
        let zombie = ZombieEntity::new(new_entity);

        let new_entity = zombie.get_entity();
        new_entity.velocity.store(old_entity.velocity.load());
        new_entity.yaw.store(old_entity.yaw.load());
        new_entity.pitch.store(old_entity.pitch.load());
        new_entity.on_ground.store(
            old_entity.on_ground.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        new_entity
            .age
            .store(old_entity.age.load(Ordering::Relaxed), Ordering::Relaxed);
        new_entity.invulnerable.store(
            old_entity.invulnerable.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        if let Some(name) = &**old_entity.custom_name.load() {
            new_entity.set_custom_name(name.clone());
            new_entity.custom_name_visible.store(
                old_entity.custom_name_visible.load(Ordering::Relaxed),
                Ordering::Relaxed,
            );
        }

        let effects: Vec<_> = self
            .entity
            .mob_entity
            .living_entity
            .active_effects
            .lock()
            .await
            .values()
            .cloned()
            .collect();
        let new_living = &zombie.get_mob_entity().living_entity;
        for effect in effects {
            new_living.add_effect(effect).await;
        }

        world.spawn_entity(zombie).await;
        // Zombie.java:243 gates this on `!isSilent()`, which has no equivalent field here.
        world.sync_world_event(
            WorldEvent::SoundHuskToZombie,
            old_entity.block_pos.load(),
            0,
        );

        old_entity.remove().await;
    }
}

impl NBTStorage for HuskEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_int(
                "DrownedConversionTime",
                self.conversion_time.load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity
                .mob_entity
                .living_entity
                .read_nbt_non_mut(nbt)
                .await;
            let time = nbt.get_int("DrownedConversionTime").unwrap_or(-1);
            self.conversion_time.store(time, Ordering::Relaxed);
        })
    }
}

impl Mob for HuskEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    /// Vanilla `Husk::doHurtTarget`: an unarmed husk hit applies Hunger for
    /// `140 * getEffectiveDifficulty()` ticks.
    fn on_successful_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let held_item = self.entity.mob_entity.living_entity.held_item(entity).await;
            let main_hand_empty = held_item.lock().await.is_empty();
            if !main_hand_empty {
                return;
            }
            let Some(target_living) = target.get_living_entity() else {
                return;
            };

            let difficulty = RegionalDifficulty::at(&entity.world.load(), entity.pos.load());
            let duration = (140.0 * difficulty.effective_difficulty) as i32;
            target_living
                .add_effect(Effect {
                    effect_type: &StatusEffect::HUNGER,
                    duration,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                })
                .await;
        })
    }

    /// Vanilla `Zombie::tick`'s underwater-conversion timer (`convertsInWater` is `true` for the
    /// base `Zombie`, and `Husk` doesn't override it, so husks convert to zombies just like any
    /// other zombie submerged for long enough).
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self
                .entity
                .mob_entity
                .living_entity
                .dead
                .load(Ordering::Relaxed)
            {
                return;
            }

            let converting_time = self.conversion_time.load(Ordering::Relaxed);
            if converting_time >= 0 {
                let new_time = converting_time - 1;
                self.conversion_time.store(new_time, Ordering::Relaxed);
                if new_time < 0 {
                    self.finish_conversion().await;
                }
            } else if self
                .entity
                .mob_entity
                .living_entity
                .entity
                .touching_water
                .load(Ordering::Relaxed)
            {
                let new_time = self.in_water_time.fetch_add(1, Ordering::Relaxed) + 1;
                if new_time >= WATER_TICKS_TO_START_CONVERSION {
                    self.in_water_time.store(0, Ordering::Relaxed);
                    self.conversion_time
                        .store(CONVERSION_TICKS, Ordering::Relaxed);
                }
            } else {
                self.in_water_time.store(-1, Ordering::Relaxed);
            }
        })
    }
}
