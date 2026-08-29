use pumpkin_data::packet::clientbound::play::DEBUG_SAMPLE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, VarInt};

#[java_packet(DEBUG_SAMPLE)]
pub struct CDebugSample<'a> {
    pub sample: &'a [i64],
    pub sample_type: VarInt,
}

impl<'a> CDebugSample<'a> {
    #[must_use]
    pub const fn new(sample: &'a [i64], sample_type: VarInt) -> Self {
        Self {
            sample,
            sample_type,
        }
    }
}

impl ClientPacket for CDebugSample<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.sample.len() as i32))?;
        for &val in self.sample {
            write.write_i64_be(val)?;
        }
        write.write_var_int(&self.sample_type)?;
        Ok(())
    }
}
