use std::io::Write;

use pumpkin_data::{
    packet::clientbound::play::SOUND, sound::SoundCategory,
    sound_id_remap::remap_sound_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{ClientPacket, IdOr, SoundEvent, VarInt, WritingError, ser::NetworkWriteExt};

#[java_packet(SOUND)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19_3 {
            let sound_event = match &self.sound_event {
                IdOr::Id(id) => IdOr::Id(remap_sound_id_for_version(*id, *version)),
                IdOr::Value(value) => IdOr::Value(value.clone()),
            };

            crate::IdOr::<crate::SoundEvent>::write(&sound_event, &mut write, |w, e| {
                w.write_string(&e.sound_name)?;
                w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
            })?;
        } else if *version >= JavaMinecraftVersion::V_1_9 {
            let sound_id = match &self.sound_event {
                IdOr::Id(id) => remap_sound_id_for_version(*id, *version),
                IdOr::Value(_) => 0,
            };
            write.write_var_int(&VarInt(i32::from(sound_id)))?;
        } else {
            let sound_name: &str = match &self.sound_event {
                IdOr::Id(id) => {
                    let remapped = remap_sound_id_for_version(*id, *version);
                    pumpkin_data::sound::Sound::NAMES
                        .get(usize::from(remapped))
                        .copied()
                        .unwrap_or("ambient.cave")
                }
                IdOr::Value(event) => &event.sound_name,
            };
            write.write_string(sound_name)?;
        }

        if *version >= JavaMinecraftVersion::V_1_9 {
            write.write_var_int(&self.sound_category)?;
        }

        write.write_i32_be(self.position.x)?;
        write.write_i32_be(self.position.y)?;
        write.write_i32_be(self.position.z)?;
        write.write_f32_be(self.volume)?;

        if *version >= JavaMinecraftVersion::V_1_10 {
            write.write_f32_be(self.pitch)?;
        } else {
            let pitch_byte = (self.pitch * 63.0).round().clamp(0.0, 255.0) as u8;
            write.write_u8(pitch_byte)?;
        }

        if *version >= JavaMinecraftVersion::V_1_19 {
            write.write_i64_be(self.seed as i64)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use pumpkin_data::sound::SoundCategory;
    use pumpkin_data::sound_id_remap::remap_sound_id_for_version;
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    use crate::{ClientPacket, IdOr, SoundEvent, VarInt, ser::NetworkReadExt};

    use super::CSoundEffect;

    fn first_remapped_sound_id(version: JavaMinecraftVersion) -> u16 {
        (0..=u16::MAX)
            .find(|id| remap_sound_id_for_version(*id, version) != *id)
            .expect("sound remap table should contain at least one changed id")
    }

    fn first_var_int(bytes: Vec<u8>) -> VarInt {
        VarInt::decode(&mut Cursor::new(bytes)).unwrap()
    }

    #[test]
    fn numeric_sound_id_remaps_for_1_21_11() {
        let sound_id = first_remapped_sound_id(JavaMinecraftVersion::V_1_21_11);
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
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(
            first_var_int(bytes),
            VarInt::from(remap_sound_id_for_version(sound_id, JavaMinecraftVersion::V_1_21_11) + 1)
        );
    }

    #[test]
    fn numeric_sound_id_stays_latest_for_26_2() {
        let sound_id = first_remapped_sound_id(JavaMinecraftVersion::V_1_21_11);
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
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_26_2)
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
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(0));
    }

    #[test]
    fn position_scaling_is_applied_only_once() {
        let packet = CSoundEffect::new(
            IdOr::Id(0),
            SoundCategory::Players,
            &Vector3::new(2.5, 3.5, 4.5),
            1.0,
            1.0,
            42.0,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        let mut cursor = Cursor::new(bytes);
        // skip sound ID varint
        VarInt::decode(&mut cursor).unwrap();
        // skip sound category varint
        VarInt::decode(&mut cursor).unwrap();
        // read position
        let x = cursor.get_i32_be().unwrap();
        let y = cursor.get_i32_be().unwrap();
        let z = cursor.get_i32_be().unwrap();

        // position should be floor(input * 8), applied exactly once
        // (2.5 * 8 = 20, 3.5 * 8 = 28, 4.5 * 8 = 36)
        assert_eq!(x, 20);
        assert_eq!(y, 28);
        assert_eq!(z, 36);
    }
}
