use std::io::Write;

use pumpkin_data::packet::clientbound::LOGIN_CUSTOM_QUERY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Sent by the server to initiate a custom plugin messaging exchange during login.
///
/// This is used by server software (like proxies or anti-cheats) to request
/// information from a client-side mod before the player officially joins.
#[java_packet(LOGIN_CUSTOM_QUERY)]
pub struct CLoginPluginRequest<'a> {
    /// A unique ID for this request. The client must include this same ID
    /// in its response so the server can match them up.
    pub message_id: VarInt,
    /// The name of the custom channel (e.g., "velocity:main").
    pub channel: &'a str,
    /// The raw payload data. Unlike standard plugin messages, this data
    /// is often serialized without a length prefix at the end of the packet.
    pub data: &'a [u8],
}

impl<'a> CLoginPluginRequest<'a> {
    #[must_use]
    pub const fn new(message_id: VarInt, channel: &'a str, data: &'a [u8]) -> Self {
        Self {
            message_id,
            channel,
            data,
        }
    }
}

impl ClientPacket for CLoginPluginRequest<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.message_id)?;

        write.write_string(self.channel)?;

        write.write_all(self.data).map_err(WritingError::IoError)
    }
}
