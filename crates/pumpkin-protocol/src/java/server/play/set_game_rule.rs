use pumpkin_data::packet::serverbound::play::SET_GAME_RULE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    codec::var_int::VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

pub struct GameRuleEntry<'a> {
    pub game_rule_key: &'a str,
    pub value: &'a str,
}

#[java_packet(SET_GAME_RULE)]
pub struct SSetGameRule<'a> {
    pub entries: Vec<GameRuleEntry<'a>>,
}

impl<'a> ServerPacket<'a> for SSetGameRule<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let count = bytebuf.get_var_int()?.0 as usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let game_rule_key = bytebuf.get_str_borrowed()?;
            let value = bytebuf.get_str_borrowed()?;
            entries.push(GameRuleEntry {
                game_rule_key,
                value,
            });
        }
        Ok(Self { entries })
    }
}

impl crate::ClientPacket for SSetGameRule<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&VarInt(self.entries.len() as i32))?;
        for entry in &self.entries {
            write.write_string(entry.game_rule_key)?;
            write.write_string(entry.value)?;
        }
        Ok(())
    }
}
