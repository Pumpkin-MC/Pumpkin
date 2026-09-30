use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::SET_SIMULATION_DISTANCE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_SIMULATION_DISTANCE)]
pub struct CSetSimulationDistance {
    pub simulation_distance: VarInt,
}

impl CSetSimulationDistance {
    #[must_use]
    pub const fn new(simulation_distance: VarInt) -> Self {
        Self {
            simulation_distance,
        }
    }
}

impl ClientPacket for CSetSimulationDistance {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.simulation_distance)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetSimulationDistance {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            simulation_distance: bytebuf.get_var_int()?,
        })
    }
}
