use pumpkin_data::packet::clientbound::play::GAME_RULE_VALUES;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(GAME_RULE_VALUES)]
pub struct CGameRuleValues<'a> {
    pub rules: &'a [(&'a str, &'a str)],
}

impl<'a> CGameRuleValues<'a> {
    #[must_use]
    pub const fn new(rules: &'a [(&'a str, &'a str)]) -> Self {
        Self { rules }
    }
}

impl ClientPacket for CGameRuleValues<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.rules.len() as i32))?;
        for (key, value) in self.rules {
            write.write_string(key)?;
            write.write_string(value)?;
        }
        Ok(())
    }
}
