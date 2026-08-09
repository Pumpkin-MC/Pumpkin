use pumpkin_data::packet::clientbound::PLAY_COOLDOWN;

use crate::codec::var_int::VarInt;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_COOLDOWN)]
pub struct CItemCooldown {
    pub group: String,
    pub cooldown: VarInt,
}

impl CItemCooldown {
    #[must_use]
    pub const fn new(group: String, cooldown: VarInt) -> Self {
        Self { group, cooldown }
    }
}

impl ClientPacket for CItemCooldown {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(&self.group)?;
        write.write_var_int(&self.cooldown)?;
        Ok(())
    }
}
