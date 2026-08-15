use pumpkin_data::packet::clientbound::CONFIG_RESOURCE_PACK_POP;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_RESOURCE_PACK_POP)]
pub struct CConfigRemoveResourcePack<'a> {
    pub uuid: Option<&'a uuid::Uuid>,
}

impl<'a> CConfigRemoveResourcePack<'a> {
    #[must_use]
    pub const fn new(uuid: Option<&'a uuid::Uuid>) -> Self {
        Self { uuid }
    }
}

impl ClientPacket for CConfigRemoveResourcePack<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if let Some(uuid) = self.uuid {
            write.write_bool(true)?;
            write.write_uuid(uuid)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
