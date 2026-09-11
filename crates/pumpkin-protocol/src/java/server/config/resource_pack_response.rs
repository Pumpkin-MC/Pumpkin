use pumpkin_data::packet::serverbound::config::RESOURCE_PACK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

pub enum ResourcePackResponseResult {
    DownloadSuccess,
    DownloadFail,
    Downloaded,
    Accepted,
    Declined,
    InvalidUrl,
    ReloadFailed,
    Discarded,
    Unknown(i32),
}

/// Sent by the client to inform the server of the status of a requested resource pack.
///
/// This allows the server to know if the player is using the required textures
/// or if the download failed.
#[java_packet(RESOURCE_PACK)]
pub struct SConfigResourcePack {
    /// The unique identifier of the resource pack this response refers to.
    pub uuid: uuid::Uuid,
    /// The status code of the operation, mapped to [`ResourcePackResponseResult`].
    pub result: VarInt,
}

impl<'a> ServerPacket<'a> for SConfigResourcePack {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let uuid = if *version >= JavaMinecraftVersion::V_1_20_3 {
            bytebuf.get_uuid()?
        } else {
            uuid::Uuid::nil()
        };
        if *version < JavaMinecraftVersion::V_1_10 {
            let _hash = bytebuf.get_str_bounded_borrowed(40)?;
        }
        let result = bytebuf.get_var_int()?;
        Ok(Self { uuid, result })
    }
}

impl crate::ClientPacket for SConfigResourcePack {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_uuid(&self.uuid)?;
        write.write_var_int(&self.result)?;
        Ok(())
    }
}

impl SConfigResourcePack {
    #[must_use]
    pub const fn response_result(&self) -> ResourcePackResponseResult {
        match self.result.0 {
            0 => ResourcePackResponseResult::DownloadSuccess,
            1 => ResourcePackResponseResult::Declined,
            2 => ResourcePackResponseResult::DownloadFail,
            3 => ResourcePackResponseResult::Accepted,
            4 => ResourcePackResponseResult::Downloaded,
            5 => ResourcePackResponseResult::InvalidUrl,
            6 => ResourcePackResponseResult::ReloadFailed,
            7 => ResourcePackResponseResult::Discarded,
            x => ResourcePackResponseResult::Unknown(x),
        }
    }
}
