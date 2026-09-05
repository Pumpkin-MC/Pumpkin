use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::serverbound::play::CLIENT_COMMAND;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CLIENT_COMMAND)]
pub struct SClientCommand {
    pub action_id: VarInt,
}

impl SClientCommand {
    pub const PERFORM_RESPAWN: i32 = 0;
    pub const REQUEST_STATS: i32 = 1;
    /// 1.7.10 - 1.15.2: Open Inventory Achievement
    pub const OPEN_INVENTORY_ACHIEVEMENT: i32 = 2;
    /// 26.1+: Request `GameRule` Values
    pub const REQUEST_GAMERULE_VALUES: i32 = 2;

    #[must_use]
    pub const fn new(action_id: VarInt) -> Self {
        Self { action_id }
    }
}

impl<'a> ServerPacket<'a> for SClientCommand {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let action_id = if *version >= JavaMinecraftVersion::V_1_8 {
            bytebuf.get_var_int()?
        } else {
            VarInt(i32::from(bytebuf.get_u8()?))
        };
        Ok(Self { action_id })
    }
}

impl ClientPacket for SClientCommand {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_8 {
            write.write_var_int(&self.action_id)?;
        } else {
            write.write_u8(self.action_id.0 as u8)?;
        }
        Ok(())
    }
}
