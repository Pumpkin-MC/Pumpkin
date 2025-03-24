use pumpkin_data::{packet::clientbound::PLAY_SOUND_ENTITY, sound::SoundCategory};
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::{IdOr, SoundEvent, VarInt};

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
#[packet(PLAY_SOUND_ENTITY)]
pub struct CEntitySoundEffect<'a> {
    #[serde(borrow)]
    sound_event: IdOr<SoundEvent<'a>>,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl<'a> CEntitySoundEffect<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sound_event: IdOr<SoundEvent<'a>>,
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
