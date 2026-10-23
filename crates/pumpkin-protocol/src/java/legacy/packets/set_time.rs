//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/set_time.rs`.

use crate::java::client::play::CUpdateTime;
use crate::java::legacy::LegacyWrite;
use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

impl LegacyWrite for CUpdateTime {
    fn write_legacy(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i64_be(self.game_time)?;

        if version >= &JavaMinecraftVersion::V_26_1 {
            write.write_var_int(&VarInt(self.clock_updates.len() as i32))?;
            for &(clock_id, total_ticks, partial_tick, rate) in &self.clock_updates {
                write.write_var_int(&VarInt(clock_id))?;
                write.write_var_long(&VarLong(total_ticks))?;
                write.write_f32_be(partial_tick)?;
                write.write_f32_be(rate)?;
            }
        } else {
            let (day_time, rate) = self
                .clock_updates
                .first()
                .map_or((0, 1.0), |&(_, total_ticks, _, rate)| (total_ticks, rate));

            let is_paused = rate == 0.0;
            let day_time = if version < &JavaMinecraftVersion::V_1_21_2 {
                if is_paused {
                    match day_time.cmp(&0) {
                        std::cmp::Ordering::Greater => -day_time,
                        std::cmp::Ordering::Equal => -1,
                        std::cmp::Ordering::Less => day_time,
                    }
                } else {
                    day_time.abs()
                }
            } else {
                day_time
            };

            write.write_i64_be(day_time)?;

            if version >= &JavaMinecraftVersion::V_1_21_2 {
                let is_increasing = rate > 0.0;
                write.write_bool(is_increasing)?;
            }
        }
        Ok(())
    }
}
