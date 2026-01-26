use pumpkin_data::packet::serverbound::CONFIG_RESOURCE_PACK;
use pumpkin_macros::java_packet;
use serde::Serialize;

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
#[derive(serde::Deserialize, Serialize)]
#[java_packet(CONFIG_RESOURCE_PACK)]
pub struct SConfigResourcePack {
    /// The unique identifier of the resource pack this response refers to.
    #[serde(with = "uuid::serde::compact")]
    pub uuid: uuid::Uuid,
    /// The status code of the operation, mapped to [ResourcePackResponseResult].
    result: VarInt,
}

impl SConfigResourcePack {
    pub fn response_result(&self) -> ResourcePackResponseResult {
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
