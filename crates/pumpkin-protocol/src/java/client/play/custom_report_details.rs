use pumpkin_data::packet::clientbound::PLAY_CUSTOM_REPORT_DETAILS;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CUSTOM_REPORT_DETAILS)]
pub struct CPlayCustomReportDetails<'a> {
    pub details: &'a [(&'a str, &'a str)],
}

impl<'a> CPlayCustomReportDetails<'a> {
    #[must_use]
    pub const fn new(details: &'a [(&'a str, &'a str)]) -> Self {
        Self { details }
    }
}

impl ClientPacket for CPlayCustomReportDetails<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.details.len() as i32))?;
        for (key, value) in self.details {
            write.write_string(key)?;
            write.write_string(value)?;
        }
        Ok(())
    }
}
