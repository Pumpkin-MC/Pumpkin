use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::SWING;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}

impl<'a> ServerPacket<'a> for SSwingArm {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let hand = if version >= &JavaMinecraftVersion::V_1_9 {
            bytebuf.get_var_int()?
        } else if version >= &JavaMinecraftVersion::V_1_8 {
            VarInt(0)
        } else {
            let _entity_id = bytebuf.get_i32_be()?;
            let _animation = bytebuf.get_u8()?;
            VarInt(0)
        };
        Ok(Self { hand })
    }
}

impl crate::ClientPacket for SSwingArm {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if version >= &JavaMinecraftVersion::V_1_9 {
            write.write_var_int(&self.hand)?;
        } else if version <= &JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(0)?;
            write.write_u8(1)?;
        }
        Ok(())
    }
}
