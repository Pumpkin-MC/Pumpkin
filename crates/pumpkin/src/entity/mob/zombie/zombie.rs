use std::sync::Arc;
use std::sync::atomic::{AtomicI8, AtomicI32, Ordering};

use pumpkin_data::entity::EntityType;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::ai::goal::break_door::{self, BreakDoorGoal};
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::zombie::{ZombieEntityBase, drowned::DrownedEntity};
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    mob::{Mob, MobEntity},
};

/// `Zombie::inWaterTime` threshold (`Zombie.java` `tick`, `if (this.inWaterTime >= 600)`):
/// ticks the zombie must have `isEyeInFluid(FluidTags.WATER)` before it starts converting.
/// Same value as `Husk`'s own underwater-conversion timer (`husk.rs`'s
/// `WATER_TICKS_TO_START_CONVERSION`) -- both are driven by the same base `Zombie::tick`.
const WATER_TICKS_TO_START_CONVERSION: i32 = 600;
/// `Zombie::startUnderWaterConversion(300)` (`Zombie.java` `tick`).
const CONVERSION_TICKS: i32 = 300;

/// `can_break_doors` sentinel: no `CanBreakDoors` NBT loaded yet and no spawn roll made yet.
/// Distinguishes a freshly spawned zombie (roll on first tick) from one restored from a chunk
/// (`read_nbt_non_mut` already settled it to `0`/`1` before `mob_init_data_tracker` runs).
const CAN_BREAK_DOORS_UNDECIDED: i8 = -1;
/// `Zombie.BREAK_DOOR_CHANCE` (`Zombie.java:93`), rolled against `finalizeSpawn`'s
/// `difficultyModifier` at `Zombie.java:493`: `random.nextFloat() < difficultyModifier * 0.1F`.
const BREAK_DOOR_CHANCE: f32 = 0.1;

pub struct ZombieEntity {
    entity: Arc<ZombieEntityBase>,
    /// Vanilla `Zombie::inWaterTime`. See `HuskEntity::in_water_time` for the same
    /// `touching_water`-vs-`isEyeInFluid` caveat.
    in_water_time: AtomicI32,
    /// Vanilla `Zombie::conversionTime`. `-1` while not converting.
    conversion_time: AtomicI32,
    /// Vanilla `Zombie::canBreakDoors`. `-1` (`CAN_BREAK_DOORS_UNDECIDED`) until a spawn roll or
    /// an NBT load settles it to `0`/`1`; mirrors whether `Zombie::breakDoorGoal` is currently in
    /// the goal selector (vanilla adds/removes it dynamically from `setCanBreakDoors`).
    can_break_doors: AtomicI8,
}

impl ZombieEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::new(entity);
        let zombie = Self {
            entity,
            in_water_time: AtomicI32::new(0),
            conversion_time: AtomicI32::new(-1),
            can_break_doors: AtomicI8::new(CAN_BREAK_DOORS_UNDECIDED),
        };
        Arc::new(zombie)
    }

    /// `Zombie::setCanBreakDoors` (`Zombie.java:156-170`), minus the `navigation.canNavigateGround`
    /// guard (Pumpkin's `BreakDoorGoal`/`InteractWithDoorGoal` have no equivalent gate either).
    async fn set_can_break_doors(&self, can_break_doors: bool) {
        let new_value = i8::from(can_break_doors);
        let previous = self.can_break_doors.swap(new_value, Ordering::Relaxed);
        if previous == new_value || (previous == CAN_BREAK_DOORS_UNDECIDED && !can_break_doors) {
            // Undecided -> false is the common case on every chunk load (most zombies rolled
            // `false`): the selector never had `BreakDoorGoal` to begin with, so skip the
            // take/remove/put-back round trip below.
            return;
        }
        self.entity
            .mob_entity
            .navigator
            .lock()
            .unwrap()
            .set_can_open_doors(can_break_doors);
        if can_break_doors {
            let mut goal_selector = self.entity.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(1, Box::new(BreakDoorGoal::new(break_door::hard_only)));
        } else {
            let mut goal_selector = {
                let mut guard = self.entity.mob_entity.goals_selector.lock().unwrap();
                std::mem::take(&mut *guard)
            };
            goal_selector.remove_goal::<BreakDoorGoal>(self).await;
            *self.entity.mob_entity.goals_selector.lock().unwrap() = goal_selector;
        }
    }

    /// `Zombie::doUnderWaterConversion` (`Zombie.java:240-245`): replaces this zombie with a
    /// `Drowned` at the same position. Mirrors `HuskEntity::finish_conversion`'s simplified
    /// copy set (position/velocity/rotation/age/custom name/active effects) -- there is no
    /// generic `Mob::convertTo` here (equipment/leash/passenger transfer), so those are not
    /// carried over.
    async fn finish_conversion(&self) {
        let old_entity = self.get_entity();
        let world = old_entity.world.load().clone();
        let pos = old_entity.pos.load();

        let new_entity = Entity::new(world.clone(), pos, &EntityType::DROWNED);
        let drowned = DrownedEntity::new(new_entity);

        let new_entity = drowned.get_entity();
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
        let new_living = &drowned.get_mob_entity().living_entity;
        for effect in effects {
            new_living.add_effect(effect).await;
        }

        world.spawn_entity(drowned).await;
        // Zombie.java:242 gates this on `!isSilent()`, which has no equivalent field here.
        world.sync_world_event(
            WorldEvent::SoundZombieToDrowned,
            old_entity.block_pos.load(),
            0,
        );

        old_entity.remove().await;
    }
}

impl NBTStorage for ZombieEntity {
    /// `Zombie::addAdditionalSaveData` (`Zombie.java:399-405`):
    /// `putInt("InWaterTime", isInWater() ? inWaterTime : -1)` and
    /// `putInt("DrownedConversionTime", isUnderWaterConverting() ? conversionTime : -1)`.
    /// `conversion_time` is already `-1` except while actively converting, so writing it
    /// unconditionally already matches the gated vanilla value.
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.entity.mob_entity.living_entity.write_nbt(nbt).await;
            let in_water_time = if self
                .entity
                .mob_entity
                .living_entity
                .entity
                .touching_water
                .load(Ordering::Relaxed)
            {
                self.in_water_time.load(Ordering::Relaxed)
            } else {
                -1
            };
            nbt.put_int("InWaterTime", in_water_time);
            nbt.put_int(
                "DrownedConversionTime",
                self.conversion_time.load(Ordering::Relaxed),
            );
            // `Zombie::addAdditionalSaveData` (`Zombie.java:402`): `putBoolean("CanBreakDoors",
            // this.canBreakDoors())`. Still-undecided (never ticked) zombies save as `false`,
            // same as `canBreakDoors()`'s own `false` initial value.
            nbt.put_bool(
                "CanBreakDoors",
                self.can_break_doors.load(Ordering::Relaxed) == 1,
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
            self.in_water_time
                .store(nbt.get_int("InWaterTime").unwrap_or(0), Ordering::Relaxed);
            let time = nbt.get_int("DrownedConversionTime").unwrap_or(-1);
            self.conversion_time.store(time, Ordering::Relaxed);
            // `Zombie::readAdditionalSaveData` (`Zombie.java:411`):
            // `setCanBreakDoors(input.getBooleanOr("CanBreakDoors", false))`. Routed through
            // `set_can_break_doors` (rather than storing the flag directly) so a loaded zombie
            // that can break doors actually gets `BreakDoorGoal` back in its goal selector, and so
            // this settles the `CAN_BREAK_DOORS_UNDECIDED` sentinel before `mob_init_data_tracker`
            // can roll a fresh chance for it.
            self.set_can_break_doors(nbt.get_bool("CanBreakDoors").unwrap_or(false))
                .await;
        })
    }
}

impl Mob for ZombieEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    /// Delegates to `ZombieEntityBase`'s default (baby-metadata) behavior, then -- on a fresh
    /// spawn only -- performs `Zombie::finalizeSpawn`'s door-breaking roll (`Zombie.java:493`):
    /// `setCanBreakDoors(random.nextFloat() < difficultyModifier * 0.1F)`. A zombie restored from
    /// a chunk already had `can_break_doors` settled by `read_nbt_non_mut` (which runs first), so
    /// `CAN_BREAK_DOORS_UNDECIDED` here means this is a genuine new spawn, matching vanilla's
    /// `finalizeSpawn` never running for loaded entities.
    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.entity.mob_init_data_tracker().await;
            if self.can_break_doors.load(Ordering::Relaxed) == CAN_BREAK_DOORS_UNDECIDED {
                let entity = self.get_entity();
                let world = entity.world.load_full();
                let difficulty = RegionalDifficulty::at(&world, entity.pos.load());
                let roll = BREAK_DOOR_CHANCE * difficulty.special_multiplier;
                self.set_can_break_doors(rand::random::<f32>() < roll).await;
            }
        })
    }

    /// `Zombie::tick` (`Zombie.java:212-233`): the base-Zombie half of the underwater
    /// conversion timer that `Husk` also drives (see `HuskEntity::mob_tick`), this time
    /// converting into `Drowned` instead of plain `Zombie`.
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

#[cfg(test)]
mod tests {
    use super::{CONVERSION_TICKS, WATER_TICKS_TO_START_CONVERSION};

    #[test]
    fn matches_vanilla_zombie_to_drowned_thresholds() {
        // `Zombie.java:223-224`: `if (this.inWaterTime >= 600) { startUnderWaterConversion(300); }`
        assert_eq!(WATER_TICKS_TO_START_CONVERSION, 600);
        assert_eq!(CONVERSION_TICKS, 300);
    }
}
