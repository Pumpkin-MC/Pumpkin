use std::sync::Arc;

use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaySound {
    pub sound: &'static str,
}

impl PlaySound {
    #[must_use]
    pub const fn new(sound: &'static str) -> Self {
        Self { sound }
    }
}

impl EnchantmentEntityEffectExt for PlaySound {
    fn apply(
        &self,
        world: &Arc<World>,
        _enchantment_level: i32,
        _owner: Option<&Arc<Player>>,
        _entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        if let Some(snd) = Sound::from_name(self.sound) {
            world.play_sound_fine(snd, SoundCategory::Players, &position, 1.0, 1.0);
        }
    }
}
