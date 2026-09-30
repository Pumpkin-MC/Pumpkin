use pumpkin_data::packet::clientbound::play::MOVE_ENTITY_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(MOVE_ENTITY_POS)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CUpdateEntityPos {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub on_ground: bool,
}

impl CUpdateEntityPos {
    #[must_use]
    pub const fn new(entity_id: VarInt, delta: Vector3<i16>, on_ground: bool) -> Self {
        Self {
            entity_id,
            delta,
            on_ground,
        }
    }
}

impl ClientPacket for CUpdateEntityPos {
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

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateEntityPos {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;
        let delta = Vector3::new(
            bytebuf.get_i16_be()?,
            bytebuf.get_i16_be()?,
            bytebuf.get_i16_be()?,
        );
        let on_ground = bytebuf.get_bool()?;
        Ok(Self {
            entity_id,
            delta,
            on_ground,
        })
    }
}
