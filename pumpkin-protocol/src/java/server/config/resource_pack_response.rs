use pumpkin_data::packet::serverbound::CONFIG_RESOURCE_PACK;
use pumpkin_macros::packet;
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

#[derive(serde::Deserialize, Serialize)]
#[packet(CONFIG_RESOURCE_PACK)]
pub struct SConfigResourcePack {
    #[serde(with = "uuid::serde::compact")]
    pub uuid: uuid::Uuid,
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
