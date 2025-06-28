use pumpkin_data::{packet::clientbound::PLAY_SOUND_ENTITY, sound::SoundCategory};
use pumpkin_macros::packet;
use serde::Deserialize;

use crate::{IdOr, SoundEvent, VarInt};

#[allow(dead_code)]
#[derive(Deserialize)]
#[packet(PLAY_SOUND_ENTITY)]
pub struct CEntitySoundEffect {
    sound_event: IdOr<SoundEvent>,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CEntitySoundEffect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sound_event: IdOr<SoundEvent>,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event,
            sound_category: VarInt(sound_category as i32),
            entity_id,
            volume,
            pitch,
            seed,
        }
    }
}
