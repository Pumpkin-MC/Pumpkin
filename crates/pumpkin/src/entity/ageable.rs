use pumpkin_data::tracked_data;
use pumpkin_protocol::java::client::play::Metadata;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use crate::entity::mob::Mob;

pub const BABY_START_AGE: i32 = -24000;
pub const FORCED_AGE_PARTICLE_TICKS: i32 = 40;

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
        self.get_mob_entity().living_entity.entity.age.load(Relaxed) < 0
    }

    fn set_baby(&self, baby: bool) {
        self.set_age(if baby { self.get_baby_start_age() } else { 0 });
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
            entity.send_meta_data(
                &[Metadata::new(
                    tracked_data::ageable_mob::DATA_BABY_ID,
                    is_baby,
                )],
                None,
            );
        }
    }

    fn is_age_locked(&self) -> bool {
        self.get_ageable_data().age_locked.load(Relaxed)
    }

    fn set_age_locked(&self, locked: bool) {
        self.get_ageable_data().age_locked.store(locked, Relaxed);
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
        if self.can_age_up() {
            let age = self.get_age() + 1;
            self.set_age(age);
        } else if self.get_age() > 0 {
            let age = self.get_age() - 1;
            self.set_age(age);
        }
    }
}
