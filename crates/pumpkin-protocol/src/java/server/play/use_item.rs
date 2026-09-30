use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::USE_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[java_packet(USE_ITEM)]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}

impl<'a> ServerPacket<'a> for SUseItem {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let hand = bytebuf.get_var_int()?;
        let sequence = bytebuf.get_var_int()?;
        let (yaw, pitch) = (bytebuf.get_f32_be()?, bytebuf.get_f32_be()?);

        Ok(Self {
            hand,
            sequence,
            yaw,
            pitch,
        })
    }
}

impl crate::ClientPacket for SUseItem {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&self.hand)?;
        write.write_var_int(&self.sequence)?;
        write.write_f32_be(self.yaw)?;
        write.write_f32_be(self.pitch)?;
        Ok(())
    }
}
