use pumpkin_data::packet::clientbound::play::SET_TIME;
use pumpkin_macros::java_packet;

use crate::{
    ClientPacket,
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkWriteExt, WritingError},
};

#[java_packet(SET_TIME)]
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

    #[must_use]
    pub fn new_clock(
        game_time: i64,
        clock_id: i32,
        total_ticks: i64,
        partial_tick: f32,
        rate: f32,
    ) -> Self {
        Self {
            game_time,
            clock_updates: vec![(clock_id, total_ticks, partial_tick, rate)],
        }
    }
}

impl ClientPacket for CUpdateTime {
    fn write_packet_data(&self, mut write: impl std::io::Write) -> Result<(), WritingError> {
        write.write_i64_be(self.game_time)?;

        write.write_var_int(&VarInt(self.clock_updates.len() as i32))?;
        for &(clock_id, total_ticks, partial_tick, rate) in &self.clock_updates {
            write.write_var_int(&VarInt(clock_id))?;
            write.write_var_long(&VarLong(total_ticks))?;
            write.write_f32_be(partial_tick)?;
            write.write_f32_be(rate)?;
        }
        Ok(())
    }
}
