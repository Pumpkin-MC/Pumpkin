use pumpkin_data::packet::clientbound::play::SET_HEALTH;
use pumpkin_macros::java_packet;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_HEALTH)]
#[derive(Clone, Debug, PartialEq)]
pub struct CSetHealth {
    pub health: f32,
    pub food: VarInt,
    pub food_saturation: f32,
}

impl CSetHealth {
    #[must_use]
    pub const fn new(health: f32, food: VarInt, food_saturation: f32) -> Self {
        Self {
            health,
            food,
            food_saturation,
        }
    }
}

impl ClientPacket for CSetHealth {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_f32_be(self.health)?;
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.food.0 as i16)?;
        } else {
            write.write_var_int(&self.food)?;
        }
        write.write_f32_be(self.food_saturation)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetHealth {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let health = bytebuf.get_f32_be()?;
        let food = if *version <= JavaMinecraftVersion::V_1_7_6 {
            VarInt(i32::from(bytebuf.get_i16_be()?))
        } else {
            bytebuf.get_var_int()?
        };
        let food_saturation = bytebuf.get_f32_be()?;
        Ok(Self {
            health,
            food,
            food_saturation,
        })
    }
}
