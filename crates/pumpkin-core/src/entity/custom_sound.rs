use crate::entity::mob::Mob;
use pumpkin_data::sound::Sound;

pub trait CustomSound: Mob {
    fn death_sound(&self) -> Option<Sound>;
    fn hurt_sound(&self) -> Option<Sound>;
}
