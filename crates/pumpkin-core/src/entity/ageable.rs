use pumpkin_data::tracked_data;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use crate::entity::mob::Mob;

pub const BABY_START_AGE: i32 = -24000;
pub const DEFAULT_BABY_SPAWN_CHANCE: f32 = 0.05;
pub const ABSTRACT_HORSE_BABY_SPAWN_CHANCE: f32 = 0.2;
pub const PANDA_BABY_SPAWN_CHANCE: f32 = 0.2;
pub const POLAR_BEAR_BABY_SPAWN_CHANCE: f32 = 1.0;
pub const STRIDER_BABY_SPAWN_CHANCE: f32 = 0.5;
pub const FORCED_AGE_PARTICLE_TICKS: i32 = 40;
pub const AGE_LOCK_COOLDOWN_TICKS: i32 = 40;

pub struct AgeableData {
    pub forced_age: AtomicI32,
    pub forced_age_timer: AtomicI32,
    pub age_locked: AtomicBool,
    pub age_lock_particle_timer: AtomicI32,
}

impl Default for AgeableData {
    fn default() -> Self {
        Self {
            forced_age: AtomicI32::new(0),
            forced_age_timer: AtomicI32::new(0),
            age_locked: AtomicBool::new(false),
            age_lock_particle_timer: AtomicI32::new(0),
        }
    }
}

pub trait AgeableMob: Mob {
    fn get_ageable_data(&self) -> &AgeableData;

    fn get_baby_start_age(&self) -> i32 {
        BABY_START_AGE
    }

    fn is_baby(&self) -> bool {
        self.can_be_a_baby() && self.get_mob_entity().living_entity.is_baby()
    }

    fn set_baby(&self, baby: bool) {
        if self.can_be_a_baby() {
            self.set_age(if baby { self.get_baby_start_age() } else { 0 });
        }
    }

    fn get_age(&self) -> i32 {
        self.get_mob_entity().living_entity.entity.age.load(Relaxed)
    }

    fn set_age(&self, new_age: i32) {
        let mob = self.get_mob_entity();
        let entity = &mob.living_entity.entity;
        let old_age = entity.age.swap(new_age, Relaxed);

        if (old_age < 0 && new_age >= 0) || (old_age >= 0 && new_age < 0) {
            let is_baby = new_age < 0;
            entity.set_synced_data(tracked_data::ageable_mob::DATA_BABY_ID, is_baby);
            self.age_boundary_reached();
            if let Some(meta) = self.mob_metadata() {
                entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
            }
        }
    }

    fn age_boundary_reached(&self) {}

    fn default_should_spawn_baby(&self) -> bool {
        true
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        DEFAULT_BABY_SPAWN_CHANCE
    }

    fn finalize_ageable_spawn(
        &self,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        use crate::entity::mob::spawn::SpawnGroupData;
        use rand::RngExt;

        let (group_size, should_spawn_baby, baby_spawn_chance) = match group_data {
            Some(SpawnGroupData::Ageable {
                group_size,
                should_spawn_baby,
                baby_spawn_chance,
            }) => (group_size, should_spawn_baby, baby_spawn_chance),
            _ => (
                0,
                self.default_should_spawn_baby(),
                self.default_baby_spawn_chance(),
            ),
        };
        if should_spawn_baby
            && group_size > 0
            && self.get_random().random::<f32>() <= baby_spawn_chance
        {
            self.set_baby(true);
        }
        self.get_mob_entity().finalize_spawn_base();
        Some(SpawnGroupData::Ageable {
            group_size: group_size + 1,
            should_spawn_baby,
            baby_spawn_chance,
        })
    }

    fn is_age_locked(&self) -> bool {
        self.get_ageable_data().age_locked.load(Relaxed)
    }

    fn set_age_locked(&self, locked: bool) {
        self.get_ageable_data().age_locked.store(locked, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::ageable_mob::AGE_LOCKED, locked);
    }

    fn ageable_interact(
        &self,
        player: &std::sync::Arc<crate::entity::player::Player>,
        stack: &mut pumpkin_data::item_stack::ItemStack,
    ) -> bool {
        use pumpkin_data::{
            item::Item,
            sound::{Sound, SoundCategory},
            tag::{self, Taggable},
        };
        let entity = self.get_entity();
        if stack.item != &Item::GOLDEN_DANDELION
            || !self.is_baby()
            || self
                .get_ageable_data()
                .age_lock_particle_timer
                .load(Relaxed)
                != 0
            || entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_CANNOT_BE_AGE_LOCKED)
        {
            return false;
        }
        self.set_age_locked(!self.is_age_locked());
        self.set_age(self.get_baby_start_age());
        self.get_ageable_data()
            .age_lock_particle_timer
            .store(AGE_LOCK_COOLDOWN_TICKS, Relaxed);
        stack.decrement_unless_creative(player.gamemode.load(), 1);
        if self.is_age_locked() {
            self.get_mob_entity()
                .persistence_required
                .store(true, Relaxed);
        }
        entity.world.load().play_sound(
            if self.is_age_locked() {
                Sound::ItemGoldenDandelionUse
            } else {
                Sound::ItemGoldenDandelionUnuse
            },
            SoundCategory::Players,
            &entity.block_pos.load().0.to_f64().add_raw(0.5, 0.5, 0.5),
        );
        true
    }

    fn tick_age_lock_particle_timer(&self) {
        let timer = &self.get_ageable_data().age_lock_particle_timer;
        let ticks = timer.load(Relaxed);
        if ticks > 0 {
            timer.store(ticks - 1, Relaxed);
        }
    }

    fn can_age_up(&self) -> bool {
        self.is_baby() && !self.is_age_locked()
    }

    fn age_up(&self, seconds: i32, forced: bool) {
        let mut age = self.get_age();
        let old_age = age;
        age += seconds * 20;
        if age > 0 {
            age = 0;
        }

        let delta = age - old_age;
        self.set_age(age);

        let data = self.get_ageable_data();
        if forced {
            data.forced_age.fetch_add(delta, Relaxed);
            if data.forced_age_timer.load(Relaxed) == 0 {
                data.forced_age_timer.store(40, Relaxed);
            }
        }

        if self.get_age() == 0 {
            self.set_age(data.forced_age.load(Relaxed));
        }
    }

    #[must_use]
    fn get_speed_up_seconds_when_feeding(ticks_until_adult: i32) -> i32
    where
        Self: Sized,
    {
        (ticks_until_adult as f32 / 20.0 * 0.1) as i32
    }

    fn write_ageable_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        if self.can_be_a_baby() {
            nbt.put_int("Age", self.get_age());
            nbt.put_int(
                "ForcedAge",
                self.get_ageable_data().forced_age.load(Relaxed),
            );
            nbt.put_bool("AgeLocked", self.is_age_locked());
        }
    }

    fn read_ageable_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if self.can_be_a_baby() {
            self.set_age(nbt.get_int("Age").unwrap_or(0));
            self.get_ageable_data()
                .forced_age
                .store(nbt.get_int("ForcedAge").unwrap_or(0), Relaxed);
            self.set_age_locked(nbt.get_bool("AgeLocked").unwrap_or(false));
        }
    }

    fn can_be_a_baby(&self) -> bool {
        true
    }

    fn ageable_ai_step(&self) {
        self.tick_age_lock_particle_timer();
        if self.can_age_up() {
            let age = self.get_age() + 1;
            self.set_age(age);
        } else if self.get_age() > 0 {
            let age = self.get_age() - 1;
            self.set_age(age);
        }
    }
}
