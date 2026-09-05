use pumpkin_data::packet::clientbound::play::{PLAYER_COMBAT_END, PLAYER_COMBAT_ENTER};
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAYER_COMBAT_ENTER)]
pub struct CCombatEnter;

impl ClientPacket for CCombatEnter {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}

#[java_packet(PLAYER_COMBAT_END)]
pub struct CCombatEnd {
    pub duration_ticks: VarInt,
}

impl CCombatEnd {
    #[must_use]
    pub const fn new(duration_ticks: VarInt) -> Self {
        Self { duration_ticks }
    }
}

impl ClientPacket for CCombatEnd {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.duration_ticks)?;
        Ok(())
    }
}
