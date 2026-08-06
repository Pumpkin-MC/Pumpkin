use pumpkin_data::packet::clientbound::PLAY_AWARD_STATS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_AWARD_STATS)]
pub struct CAwardStats<'a> {
    pub stats: &'a [Statistic],
}

pub struct Statistic {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
}

impl Statistic {
    pub fn write(&self, mut write: impl std::io::Write) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.category_id)?;
        write.write_var_int(&self.statistic_id)?;
        write.write_var_int(&self.value)?;
        Ok(())
    }
}

impl ClientPacket for CAwardStats<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.stats.len() as i32))?;
        for stat in self.stats {
            stat.write(&mut write)?;
        }
        Ok(())
    }
}
