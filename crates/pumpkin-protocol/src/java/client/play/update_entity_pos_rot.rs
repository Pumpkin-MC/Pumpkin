use pumpkin_data::packet::clientbound::play::MOVE_ENTITY_POS_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

use super::update_entity_pos::read_on_ground_and_linear_delta;

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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;
        // Since 26.3 the on ground flag and the delta step count are packed into a var int in
        // front of the delta, 0 steps being a single linear delta.
        write.write_var_int(&VarInt(i32::from(self.on_ground)))?;
        write.write_i16_be(self.delta.x)?;
        write.write_i16_be(self.delta.y)?;
        write.write_i16_be(self.delta.z)?;
        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateEntityPosRot {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;
        let (on_ground, delta) = read_on_ground_and_linear_delta(bytebuf)?;
        let yaw = bytebuf.get_u8()?;
        let pitch = bytebuf.get_u8()?;
        Ok(Self {
            entity_id,
            delta,
            yaw,
            pitch,
            on_ground,
        })
    }
}
