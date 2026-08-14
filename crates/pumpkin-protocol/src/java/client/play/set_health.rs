use pumpkin_data::packet::clientbound::PLAY_SET_HEALTH;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_HEALTH)]
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32(self.health)?;
        write.write_var_int(&self.food)?;
        write.write_f32(self.food_saturation)?;
        Ok(())
    }
}
