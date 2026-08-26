use pumpkin_data::packet::clientbound::play::AWARD_STATS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(AWARD_STATS)]
pub struct CAwardStats<'a> {
    pub stats: &'a [Statistic],
}

pub struct Statistic {
    pub category_id: VarInt,
    pub statistic_id: VarInt,
    pub value: VarInt,
}

use pumpkin_data::custom_stat_id_remap::remap_custom_stat_id_for_version;
use pumpkin_data::entity_id_remap::remap_entity_id_for_version;
use pumpkin_data::item_id_remap::remap_item_id_for_version;

impl Statistic {
    pub fn write(&self, write: impl std::io::Write) -> Result<(), crate::ser::WritingError> {
        self.write_with_version(write, &JavaMinecraftVersion::V_26_2)
    }

    pub fn write_with_version(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        let remapped_stat_id = match self.category_id.0 {
            // Category 8 is Custom stat
            8 => remap_custom_stat_id_for_version(self.statistic_id.0 as u32, *version) as i32,
            // Categories 1..=5 are item stats (Crafted, Used, Broken, PickedUp, Dropped)
            1..=5 => remap_item_id_for_version(self.statistic_id.0 as u16, *version) as i32,
            // Categories 6..=7 are entity stats (Killed, KilledBy)
            6..=7 => remap_entity_id_for_version(self.statistic_id.0 as u16, *version) as i32,
            _ => self.statistic_id.0,
        };
        write.write_var_int(&self.category_id)?;
        write.write_var_int(&VarInt(remapped_stat_id))?;
        write.write_var_int(&self.value)?;
        Ok(())
    }
}

impl ClientPacket for CAwardStats<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.stats.len() as i32))?;
        for stat in self.stats {
            stat.write_with_version(&mut write, version)?;
        }
        Ok(())
    }
}
