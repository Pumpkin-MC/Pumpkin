use pumpkin_data::packet::serverbound::PLAY_RESOURCE_PACK;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};

pub use crate::java::server::config::ResourcePackResponseResult;

#[java_packet(PLAY_RESOURCE_PACK)]
pub struct SPlayResourcePack {
    pub uuid: uuid::Uuid,
    pub result: VarInt,
}

impl<'a> ServerPacket<'a> for SPlayResourcePack {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            uuid: bytebuf.get_uuid()?,
            result: bytebuf.get_var_int()?,
        })
    }
}

impl SPlayResourcePack {
    #[must_use]
    pub const fn response_result(&self) -> ResourcePackResponseResult {
        ResourcePackResponseResult::from_id(self.result.0)
    }
}
