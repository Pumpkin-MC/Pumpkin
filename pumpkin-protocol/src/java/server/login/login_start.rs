use std::io::Read;

use pumpkin_data::packet::serverbound::LOGIN_HELLO;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;
use serde::Serialize;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(LOGIN_HELLO)]
#[derive(Serialize)]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ServerPacket for SLoginStart {
    fn read(mut read: impl Read, _version: &MinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            name: read.get_string_bounded(16)?,
            uuid: read.get_uuid()?,
        })
    }
}
