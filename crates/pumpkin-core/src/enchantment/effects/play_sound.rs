use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

const CHANNELING_THUNDER_VOLUME: f32 = 5.0;

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
        entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        if let Some(snd) = Sound::from_name(self.sound) {
            let category = if entity.is_some_and(|e| e.entity_type == &EntityType::PLAYER) {
                SoundCategory::Players
            } else {
                SoundCategory::Neutral
            };
            let volume = if snd == Sound::ItemTridentThunder {
                CHANNELING_THUNDER_VOLUME
            } else {
                1.0
            };
            world.play_sound_fine(snd, category, &position, volume, 1.0);
        }
    }
}
