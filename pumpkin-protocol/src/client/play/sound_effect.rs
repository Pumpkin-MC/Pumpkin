use pumpkin_data::{packet::clientbound::PLAY_SOUND, sound::SoundCategory};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{IdOr, SoundEvent, VarInt};

#[derive(Serialize)]
#[packet(PLAY_SOUND)]
pub struct CSoundEffect {
    sound_event: IdOr<SoundEvent>,
    sound_category: VarInt,
    position: Vector3<i32>,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CSoundEffect {
    pub fn new(
        sound_event: IdOr<SoundEvent>,
        sound_category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event,
            sound_category: VarInt(sound_category as i32),
            position: Vector3::new(
                (position.x * 8.0) as i32,
                (position.y * 8.0) as i32,
                (position.z * 8.0) as i32,
            ),
            volume,
            pitch,
            seed,
        }
    }
}
