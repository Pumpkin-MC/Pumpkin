use pumpkin_data::packet::clientbound::play::MOVE_ENTITY_POS_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

#[java_packet(MOVE_ENTITY_POS_ROT)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CUpdateEntityPosRot {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

impl CUpdateEntityPosRot {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        delta: Vector3<i16>,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}

impl ClientPacket for CUpdateEntityPosRot {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }
        // Since 26.3 the on ground flag and the delta step count are packed into a var int in
        // front of the delta, 0 steps being a single linear delta.
        if *version >= JavaMinecraftVersion::V_26_3 {
            write.write_var_int(&VarInt(i32::from(self.on_ground)))?;
        }
        if *version >= JavaMinecraftVersion::V_1_9 {
            write.write_i16_be(self.delta.x)?;
            write.write_i16_be(self.delta.y)?;
            write.write_i16_be(self.delta.z)?;
        } else {
            write.write_i8((self.delta.x / 128) as i8)?;
            write.write_i8((self.delta.y / 128) as i8)?;
            write.write_i8((self.delta.z / 128) as i8)?;
        }
        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;
        if *version >= JavaMinecraftVersion::V_1_8 && *version < JavaMinecraftVersion::V_26_3 {
            write.write_bool(self.on_ground)?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateEntityPosRot {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = if *version <= JavaMinecraftVersion::V_1_7_6 {
            VarInt(bytebuf.get_i32_be()?)
        } else {
            bytebuf.get_var_int()?
        };
        // Mirrors the packed on ground flag and delta step count written since 26.3; only a
        // single linear delta (0 steps) is supported, since the struct has no stepped variant.
        let packed_on_ground = if *version >= JavaMinecraftVersion::V_26_3 {
            let properties = bytebuf.get_var_int()?.0;
            if properties >> 1 != 0 {
                return Err(ReadingError::Message(
                    "stepped entity position deltas are not supported".to_string(),
                ));
            }
            Some(properties & 1 != 0)
        } else {
            None
        };
        let delta = if *version >= JavaMinecraftVersion::V_1_9 {
            Vector3::new(
                bytebuf.get_i16_be()?,
                bytebuf.get_i16_be()?,
                bytebuf.get_i16_be()?,
            )
        } else {
            Vector3::new(
                i16::from(bytebuf.get_i8()?) * 128,
                i16::from(bytebuf.get_i8()?) * 128,
                i16::from(bytebuf.get_i8()?) * 128,
            )
        };
        let yaw = bytebuf.get_u8()?;
        let pitch = bytebuf.get_u8()?;
        let on_ground = if let Some(on_ground) = packed_on_ground {
            on_ground
        } else if *version >= JavaMinecraftVersion::V_1_8 {
            bytebuf.get_bool()?
        } else {
            false
        };
        Ok(Self {
            entity_id,
            delta,
            yaw,
            pitch,
            on_ground,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(version: JavaMinecraftVersion) {
        let packet =
            CUpdateEntityPosRot::new(VarInt(9), Vector3::new(4096, -128, 256), 64, 200, true);
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf, &version).unwrap();
        let read = CUpdateEntityPosRot::read(&mut buf.as_slice(), &version).unwrap();
        assert_eq!(read, packet);
    }

    #[test]
    fn round_trips_at_26_3() {
        round_trip(JavaMinecraftVersion::V_26_3);
    }

    #[test]
    fn round_trips_before_26_3() {
        round_trip(JavaMinecraftVersion::V_1_21_11);
    }

    #[test]
    fn round_trips_before_1_9() {
        round_trip(JavaMinecraftVersion::V_1_8);
    }

    #[test]
    fn stepped_delta_is_rejected() {
        let mut buf = Vec::new();
        buf.write_var_int(&VarInt(9)).unwrap();
        // properties: on_ground = false, step count = 1
        buf.write_var_int(&VarInt(2)).unwrap();
        buf.write_i16_be(1).unwrap();
        buf.write_i16_be(2).unwrap();
        buf.write_i16_be(3).unwrap();
        buf.write_u8(64).unwrap();
        buf.write_u8(200).unwrap();
        assert!(
            CUpdateEntityPosRot::read(&mut buf.as_slice(), &JavaMinecraftVersion::V_26_3).is_err()
        );
    }
}
