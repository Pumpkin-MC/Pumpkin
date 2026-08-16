use pumpkin_data::packet::clientbound::CONFIG_CODE_OF_CONDUCT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_CODE_OF_CONDUCT)]
pub struct CCodeOfConduct<'a> {
    pub code_of_conduct: &'a str,
}

impl<'a> CCodeOfConduct<'a> {
    #[must_use]
    pub const fn new(code_of_conduct: &'a str) -> Self {
        Self { code_of_conduct }
    }
}

impl ClientPacket for CCodeOfConduct<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.code_of_conduct)?;
        Ok(())
    }
}
