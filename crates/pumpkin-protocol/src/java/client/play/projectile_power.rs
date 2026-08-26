use pumpkin_data::packet::clientbound::play::PROJECTILE_POWER;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PROJECTILE_POWER)]
pub struct CProjectilePower {
    pub entity_id: VarInt,
    pub x_power: f64,
    pub y_power: f64,
    pub z_power: f64,
}

impl CProjectilePower {
    #[must_use]
    pub const fn new(entity_id: VarInt, x_power: f64, y_power: f64, z_power: f64) -> Self {
        Self {
            entity_id,
            x_power,
            y_power,
            z_power,
        }
    }
}

impl ClientPacket for CProjectilePower {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_f64_be(self.x_power)?;
        write.write_f64_be(self.y_power)?;
        write.write_f64_be(self.z_power)?;
        Ok(())
    }
}
