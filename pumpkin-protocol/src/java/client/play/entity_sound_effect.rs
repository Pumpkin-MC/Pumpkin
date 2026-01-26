use pumpkin_data::{packet::clientbound::PLAY_SOUND_ENTITY, sound::SoundCategory};
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::{IdOr, SoundEvent, VarInt};

/// Plays a sound effect that originates from a specific entity.
///
/// Unlike global sounds, this sound will follow the entity as it moves
/// through the world. The client handles the panning and attenuation
/// (volume drop-off) based on the distance between the player and the entity.
#[derive(Serialize)]
#[java_packet(PLAY_SOUND_ENTITY)]
pub struct CEntitySoundEffect {
    /// The sound to play. Can be a hardcoded ID or a custom SoundEvent
    /// (Resource Location).
    pub sound_event: IdOr<SoundEvent>,
    /// The category of the sound (e.g., Master, Music, Weather, Players).
    /// Used by the client to apply volume sliders from settings.
    pub sound_category: VarInt,
    /// The Entity ID that the sound is "attached" to.
    pub entity_id: VarInt,
    /// The loudness of the sound (usually 1.0).
    pub volume: f32,
    /// The playback speed/pitch (0.5 to 2.0).
    pub pitch: f32,
    /// A random seed used for sound variations (like different pitch shifts
    /// for the same sound).
    pub seed: i64,
}

impl CEntitySoundEffect {
    pub fn new(
        sound_event: IdOr<SoundEvent>,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: i64,
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
