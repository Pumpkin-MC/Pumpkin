use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};

/// A packet used for custom communication between the server and client.
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct CCustomPayload<'a> {
    pub channel: &'a str,
    pub data: &'a [u8],
}

impl<'a> CCustomPayload<'a> {
    #[must_use]
    pub const fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}

impl ClientPacket for CCustomPayload<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(self.channel)?;

        write.write_all(self.data).map_err(WritingError::IoError)
    }
}
