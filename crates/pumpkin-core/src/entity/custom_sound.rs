use crate::entity::mob::Mob;
use pumpkin_data::sound::{Sound, SoundCategory};

pub trait CustomSound: Mob {
    fn death_sound(&self) -> Option<Sound>;
    fn hurt_sound(&self) -> Option<Sound>;

    fn sound_volume(&self) -> f32 {
        1.0
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Players
    }

    fn on_hurt_sound(&self) {}

    fn voice_pitch(&self) -> f32 {
        self.get_mob_entity().living_entity.get_pitch()
    }
}
