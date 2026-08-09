use pumpkin_data::packet::serverbound::CONFIG_RESOURCE_PACK;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

use crate::VarInt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
#[java_packet(CONFIG_RESOURCE_PACK)]
pub struct SConfigResourcePack {
    /// The unique identifier of the resource pack this response refers to.
    pub uuid: uuid::Uuid,
    /// The status code of the operation, mapped to [`ResourcePackResponseResult`].
    pub result: VarInt,
}

impl<'a> ServerPacket<'a> for SConfigResourcePack {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            uuid: bytebuf.get_uuid()?,
            result: bytebuf.get_var_int()?,
        })
    }
}

impl SConfigResourcePack {
    #[must_use]
    pub const fn response_result(&self) -> ResourcePackResponseResult {
        ResourcePackResponseResult::from_id(self.result.0)
    }
}

impl ResourcePackResponseResult {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            0 => Self::DownloadSuccess,
            1 => Self::Declined,
            2 => Self::DownloadFail,
            3 => Self::Accepted,
            4 => Self::Downloaded,
            5 => Self::InvalidUrl,
            6 => Self::ReloadFailed,
            7 => Self::Discarded,
            x => Self::Unknown(x),
        }
    }
}
