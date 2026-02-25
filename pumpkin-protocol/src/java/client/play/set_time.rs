use pumpkin_data::packet::clientbound::PLAY_SET_TIME;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_SET_TIME)]
pub struct CUpdateTime {
    pub world_age: i64,
    pub time_of_day: i64,
    pub time_of_day_increasing: bool,
}

impl CUpdateTime {
    #[must_use]
    pub const fn new(world_age: i64, time_of_day: i64, time_of_day_increasing: bool) -> Self {
        Self {
            world_age,
            time_of_day,
            time_of_day_increasing,
        }
    }
}

impl ClientPacket for CUpdateTime {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i64_be(self.world_age)?;
        write.write_i64_be(self.time_of_day)?;
        if version >= &MinecraftVersion::V_1_21_2 {
            write.write_bool(self.time_of_day_increasing)?;
        }
        Ok(())
    }
}
