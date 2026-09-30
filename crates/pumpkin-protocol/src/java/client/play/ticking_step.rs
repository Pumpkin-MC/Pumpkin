use pumpkin_data::packet::clientbound::play::TICKING_STEP;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(TICKING_STEP)]
pub struct CTickingStep {
    pub tick_steps: VarInt,
}

impl CTickingStep {
    #[must_use]
    pub const fn new(tick_steps: VarInt) -> Self {
        Self { tick_steps }
    }
}

impl ClientPacket for CTickingStep {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.tick_steps)?;
        Ok(())
    }
}
