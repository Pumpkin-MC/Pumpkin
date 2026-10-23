use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use pumpkin_data::{sound::Sound, tracked_data};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use super::water_animal::{AMBIENT_SOUND_INTERVAL, WaterAnimal};

#[derive(Default)]
pub struct FishData {
    from_bucket: AtomicBool,
    ambient_sound_time: AtomicI32,
}

pub trait AbstractFish: WaterAnimal {
    fn fish_data(&self) -> &FishData;

    fn ambient_sound(&self) -> Sound;

    fn is_from_bucket(&self) -> bool {
        self.fish_data().from_bucket.load(Relaxed)
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.fish_data().from_bucket.store(from_bucket, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::abstract_fish::FROM_BUCKET, from_bucket);
    }

    fn fish_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("FromBucket", self.is_from_bucket());
    }

    fn fish_read_nbt(&self, nbt: &NbtCompound) {
        self.set_from_bucket(nbt.get_bool("FromBucket").unwrap_or(false));
    }

    fn play_ambient_sound(&self) {
        let entity = self.get_entity();
        if !entity.is_silent() {
            entity.world.load().play_sound_raw(
                self.ambient_sound() as u16,
                pumpkin_data::sound::SoundCategory::Neutral,
                &entity.pos.load(),
                1.0,
                self.get_mob_entity().living_entity.get_pitch(),
            );
        }
    }

    fn fish_base_tick(&self, caller: &dyn crate::entity::EntityBase) {
        if !self.get_mob_entity().living_entity.is_dead_or_dying()
            && self.get_random().random_range(0..1000)
                < self.fish_data().ambient_sound_time.fetch_add(1, Relaxed)
        {
            self.reset_ambient_sound_time();
            self.play_ambient_sound();
        }
        self.handle_air_supply(caller);
    }

    fn reset_ambient_sound_time(&self) {
        self.fish_data()
            .ambient_sound_time
            .store(-AMBIENT_SOUND_INTERVAL, Relaxed);
    }
}
