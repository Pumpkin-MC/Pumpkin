use pumpkin_data::packet::clientbound::PLAY_SET_PASSENGERS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_PASSENGERS)]
pub struct CSetPassengers<'a> {
    pub entity_id: VarInt,
    pub passengers: &'a [VarInt],
}

impl<'a> CSetPassengers<'a> {
    #[must_use]
    pub const fn new(entity_id: VarInt, passengers: &'a [VarInt]) -> Self {
        Self {
            entity_id,
            passengers,
        }
    }
}

impl ClientPacket for CSetPassengers<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&crate::VarInt(self.passengers.len() as i32))?;
        for passenger in self.passengers {
            write.write_var_int(passenger)?;
        }
        Ok(())
    }
}
