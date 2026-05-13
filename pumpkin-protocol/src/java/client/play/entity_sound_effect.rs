use std::io::Write;

use pumpkin_data::{
    packet::clientbound::PLAY_SOUND_ENTITY, sound::SoundCategory,
    sound_id_remap::remap_sound_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{ClientPacket, IdOr, SoundEvent, VarInt, WritingError, ser::NetworkWriteExt};

/// Plays a sound effect that originates from a specific entity.
///
/// Unlike global sounds, this sound will follow the entity as it moves
/// through the world. The client handles the panning and attenuation
/// (volume drop-off) based on the distance between the player and the entity.
#[java_packet(PLAY_SOUND_ENTITY)]
pub struct CEntitySoundEffect {
    /// The sound to play. Can be a hardcoded ID or a custom `SoundEvent`
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
    #[must_use]
    pub const fn new(
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

impl ClientPacket for CEntitySoundEffect {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let sound_event = match &self.sound_event {
            IdOr::Id(id) => IdOr::Id(remap_sound_id_for_version(*id, *version)),
            IdOr::Value(value) => IdOr::Value(value.clone()),
        };

        write.write_serialize(&sound_event)?;
        write.write_serialize(&self.sound_category)?;
        write.write_serialize(&self.entity_id)?;
        write.write_serialize(&self.volume)?;
        write.write_serialize(&self.pitch)?;
        write.write_serialize(&self.seed)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pumpkin_data::sound::SoundCategory;
    use pumpkin_data::sound_id_remap::remap_sound_id_for_version;
    use pumpkin_util::version::MinecraftVersion;

    use crate::{ClientPacket, IdOr, SoundEvent, VarInt};

    use super::CEntitySoundEffect;

    fn first_remapped_sound_id(version: MinecraftVersion) -> u16 {
        (0..=u16::MAX)
            .find(|id| remap_sound_id_for_version(*id, version) != *id)
            .expect("sound remap table should contain at least one changed id")
    }

    fn first_var_int(bytes: Vec<u8>) -> VarInt {
        VarInt::decode(&mut Cursor::new(bytes)).unwrap()
    }

    #[test]
    fn numeric_sound_id_remaps_for_1_21_11() {
        let sound_id = first_remapped_sound_id(MinecraftVersion::V_1_21_11);
        let packet = CEntitySoundEffect::new(
            IdOr::Id(sound_id),
            SoundCategory::Players,
            VarInt(123),
            1.0,
            1.0,
            42,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &MinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(
            first_var_int(bytes),
            VarInt::from(remap_sound_id_for_version(sound_id, MinecraftVersion::V_1_21_11) + 1)
        );
    }

    #[test]
    fn numeric_sound_id_stays_latest_for_26_1() {
        let sound_id = first_remapped_sound_id(MinecraftVersion::V_1_21_11);
        let packet = CEntitySoundEffect::new(
            IdOr::Id(sound_id),
            SoundCategory::Players,
            VarInt(123),
            1.0,
            1.0,
            42,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &MinecraftVersion::V_26_1)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(sound_id + 1));
    }

    #[test]
    fn direct_sound_event_keeps_direct_holder_encoding() {
        let packet = CEntitySoundEffect::new(
            IdOr::Value(SoundEvent {
                sound_name: "minecraft:test.sound".to_string(),
                range: None,
            }),
            SoundCategory::Players,
            VarInt(123),
            1.0,
            1.0,
            42,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &MinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(0));
    }
}
