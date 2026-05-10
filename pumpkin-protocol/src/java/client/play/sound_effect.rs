use std::io::Write;

use pumpkin_data::{
    packet::clientbound::PLAY_SOUND, sound::SoundCategory,
    sound_id_remap::remap_sound_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

use crate::{ClientPacket, IdOr, SoundEvent, VarInt, WritingError, ser::NetworkWriteExt};

#[java_packet(PLAY_SOUND)]
pub struct CSoundEffect {
    pub sound_event: IdOr<SoundEvent>,
    pub sound_category: VarInt,
    pub position: Vector3<i32>,
    pub volume: f32,
    pub pitch: f32,
    pub seed: f64,
}

impl CSoundEffect {
    #[must_use]
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

impl ClientPacket for CSoundEffect {
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
        write.write_serialize(&self.position)?;
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
    use pumpkin_util::{math::vector3::Vector3, version::MinecraftVersion};

    use crate::{ClientPacket, IdOr, SoundEvent, VarInt};

    use super::CSoundEffect;

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
        let packet = CSoundEffect::new(
            IdOr::Id(sound_id),
            SoundCategory::Players,
            &Vector3::new(1.0, 2.0, 3.0),
            1.0,
            1.0,
            42.0,
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
        let packet = CSoundEffect::new(
            IdOr::Id(sound_id),
            SoundCategory::Players,
            &Vector3::new(1.0, 2.0, 3.0),
            1.0,
            1.0,
            42.0,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &MinecraftVersion::V_26_1)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(sound_id + 1));
    }

    #[test]
    fn direct_sound_event_keeps_direct_holder_encoding() {
        let packet = CSoundEffect::new(
            IdOr::Value(SoundEvent {
                sound_name: "minecraft:test.sound".to_string(),
                range: None,
            }),
            SoundCategory::Players,
            &Vector3::new(1.0, 2.0, 3.0),
            1.0,
            1.0,
            42.0,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &MinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(0));
    }
}
