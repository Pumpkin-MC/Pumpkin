use pumpkin_data::packet::clientbound::PLAY_SET_TIME;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ClientPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(PLAY_SET_TIME)]
pub struct CUpdateTime {
    pub game_time: i64,
    /// (`clock_registry_id`, `total_ticks`, `partial_tick`, rate)
    pub clock_updates: Vec<(i32, i64, f32, f32)>,
}

impl CUpdateTime {
    #[must_use]
    pub fn new(game_time: i64, day_time: i64, increasing: bool) -> Self {
        let overworld_id = 0;
        let partial_tick = 0.0f32;
        let rate = if increasing { 1.0f32 } else { 0.0f32 }; // Normal speed
        Self {
            game_time,
            clock_updates: vec![(overworld_id, day_time, partial_tick, rate)],
        }
    }
}

impl ClientPacket for CUpdateTime {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i64_be(self.game_time)?;

        if version >= &MinecraftVersion::V_26_1 {
            write.write_var_int(&VarInt(self.clock_updates.len() as i32))?;
            for &(clock_id, total_ticks, partial_tick, rate) in &self.clock_updates {
                write.write_var_int(&VarInt(clock_id))?;
                write.write_var_long(&VarLong(total_ticks))?;
                write.write_f32_be(partial_tick)?;
                write.write_f32_be(rate)?;
            }
        } else {
            // not super efficient ig
            let (day_time, rate) = self
                .clock_updates
                .first()
                .map_or((0, 1.0), |&(_, total_ticks, _, rate)| (total_ticks, rate)); // Fallback defaults

            write.write_i64_be(day_time)?;

            if version >= &MinecraftVersion::V_1_21_2 {
                let is_increasing = rate > 0.0;
                write.write_bool(is_increasing)?;
            }
        }
        Ok(())
    }
}
