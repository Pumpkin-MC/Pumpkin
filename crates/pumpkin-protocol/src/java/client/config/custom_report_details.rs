use pumpkin_data::packet::clientbound::config::CUSTOM_REPORT_DETAILS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CUSTOM_REPORT_DETAILS)]
pub struct CConfigCustomReportDetails<'a> {
    pub details: &'a [(&'a str, &'a str)],
}

impl<'a> CConfigCustomReportDetails<'a> {
    #[must_use]
    pub const fn new(details: &'a [(&'a str, &'a str)]) -> Self {
        Self { details }
    }
}

impl ClientPacket for CConfigCustomReportDetails<'_> {
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
