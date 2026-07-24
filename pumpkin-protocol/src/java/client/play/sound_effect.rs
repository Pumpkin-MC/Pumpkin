use std::io::Write;

use pumpkin_data::{
    packet::clientbound::PLAY_SOUND, sound::SoundCategory,
    sound_id_remap::remap_sound_id_for_version,
};
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{ClientPacket, IdOr, SoundEvent, VarInt, WritingError, ser::NetworkWriteExt};

/// Clientbound sound effect — matches vanilla `PlaySoundS2CPacket` /
/// `MCProtocolLib` `ClientboundSoundPacket`.
///
/// Wire format (after sound holder + category):
/// - x/y/z as `i32` fixed-point world coords (`world * 8`, client divides by 8)
/// - volume `f32`, pitch `f32`, seed `i64`
///
/// Distance attenuation is done **on the client** from these world coordinates.
/// If coords are wrong (e.g. double `* 8`), the source is misplaced and falloff
/// feels broken (silent nearby, or always loud if the source collapses onto the player).
#[java_packet(PLAY_SOUND)]
pub struct CSoundEffect {
    pub sound_event: IdOr<SoundEvent>,
    pub sound_category: VarInt,
    /// World-space position (blocks). Scaled by 8 only when writing the packet.
    pub position: Vector3<f64>,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

impl CSoundEffect {
    #[must_use]
    pub const fn new(
        sound_event: IdOr<SoundEvent>,
        sound_category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self {
            sound_event,
            sound_category: VarInt(sound_category as i32),
            position: *position,
            volume,
            pitch,
            seed,
        }
    }

    /// Vanilla fixed-point encoding: `(int)(world * 8)`.
    #[must_use]
    pub fn fixed_position(&self) -> Vector3<i32> {
        Vector3::new(
            (self.position.x * 8.0) as i32,
            (self.position.y * 8.0) as i32,
            (self.position.z * 8.0) as i32,
        )
    }
}

impl ClientPacket for CSoundEffect {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let sound_event = match &self.sound_event {
            IdOr::Id(id) => IdOr::Id(remap_sound_id_for_version(*id, *version)),
            IdOr::Value(value) => IdOr::Value(value.clone()),
        };

        crate::IdOr::<crate::SoundEvent>::write(&sound_event, &mut write, |w, e| {
            w.write_string(&e.sound_name)?;
            w.write_option(&e.range, |w2, r| w2.write_f32_be(*r))
        })?;
        write.write_var_int(&self.sound_category)?;

        // Vanilla PlaySoundS2CPacket / MCProtocolLib: write (int)(x * 8) once.
        let fixed = self.fixed_position();
        write.write_i32_be(fixed.x)?;
        write.write_i32_be(fixed.y)?;
        write.write_i32_be(fixed.z)?;
        write.write_f32_be(self.volume)?;
        write.write_f32_be(self.pitch)?;
        write.write_i64_be(self.seed)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use pumpkin_data::sound::SoundCategory;
    use pumpkin_data::sound_id_remap::remap_sound_id_for_version;
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    use crate::{ClientPacket, IdOr, SoundEvent, VarInt};

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
    fn fixed_point_position_matches_vanilla() {
        // Vanilla: fixedX = (int)(x * 8); client recovers x/8.
        let packet = CSoundEffect::new(
            IdOr::Id(0),
            SoundCategory::Players,
            &Vector3::new(100.5, 64.0, -20.25),
            1.0,
            1.0,
            42,
        );
        let fixed = packet.fixed_position();
        assert_eq!(fixed.x, 804); // 100.5 * 8
        assert_eq!(fixed.y, 512); // 64 * 8
        assert_eq!(fixed.z, -162); // -20.25 * 8
    }

    #[test]
    fn write_encodes_fixed_point_once() {
        let packet = CSoundEffect::new(
            IdOr::Id(0),
            SoundCategory::Blocks,
            &Vector3::new(1.0, 2.0, 3.0),
            0.5,
            1.25,
            99,
        );
        let mut bytes = Vec::new();
        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        // sound id varint (1) + category varint (4 = blocks) + 3*i32 + 2*f32 + i64
        // After first varint (sound = 0+1 = 1):
        let mut cur = Cursor::new(&bytes);
        let sound = VarInt::decode(&mut cur).unwrap();
        assert_eq!(sound.0, 1);
        let cat = VarInt::decode(&mut cur).unwrap();
        assert_eq!(cat.0, SoundCategory::Blocks as i32);
        // Read three BE i32s
        let mut x_buf = [0u8; 4];
        let mut y_buf = [0u8; 4];
        let mut z_buf = [0u8; 4];
        cur.read_exact(&mut x_buf).unwrap();
        cur.read_exact(&mut y_buf).unwrap();
        cur.read_exact(&mut z_buf).unwrap();
        assert_eq!(i32::from_be_bytes(x_buf), 8); // 1.0 * 8
        assert_eq!(i32::from_be_bytes(y_buf), 16); // 2.0 * 8
        assert_eq!(i32::from_be_bytes(z_buf), 24); // 3.0 * 8
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
            42,
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
            42,
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
            42,
        );
        let mut bytes = Vec::new();

        packet
            .write_packet_data(&mut bytes, &JavaMinecraftVersion::V_1_21_11)
            .unwrap();

        assert_eq!(first_var_int(bytes), VarInt::from(0));
    }
}
