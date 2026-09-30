use pumpkin_data::packet::clientbound::play::MOVE_ENTITY_ROT;
use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(MOVE_ENTITY_ROT)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CUpdateEntityRot {
    pub entity_id: VarInt,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

impl CUpdateEntityRot {
    #[must_use]
    pub const fn new(entity_id: VarInt, yaw: u8, pitch: u8, on_ground: bool) -> Self {
        Self {
            entity_id,
            yaw,
            pitch,
            on_ground,
        }
    }
}

impl ClientPacket for CUpdateEntityRot {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        // The on ground flag moved in front of the rotation in 26.3
        write.write_bool(self.on_ground)?;
        write.write_u8(self.yaw)?;
        write.write_u8(self.pitch)?;

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateEntityRot {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;
        let on_ground = bytebuf.get_bool()?;
        let yaw = bytebuf.get_u8()?;
        let pitch = bytebuf.get_u8()?;
        Ok(Self {
            entity_id,
            yaw,
            pitch,
            on_ground,
        })
    }
}
