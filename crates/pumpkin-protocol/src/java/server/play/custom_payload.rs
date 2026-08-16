use pumpkin_data::packet::serverbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{ReadingError, ServerPacket, ser::NetworkReadSliceExt};

/// The maximum allowed size for a play custom payload (32 KiB).
const MAX_PAYLOAD_SIZE: usize = 32_767;

/// A packet used for custom communication between the client and server.
///
/// This allows mods, plugins, or proxy software to send proprietary data over the standard
/// Minecraft protocol.
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct SCustomPayload<'a> {
    /// The name of the channel used to distinguish different types of messages.
    /// Example: `minecraft:brand` or `voicechat:request_secret`.
    pub channel: &'a str,
    /// The payload sent by the client.
    pub data: &'a [u8],
}

impl<'a> ServerPacket<'a> for SCustomPayload<'a> {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            channel: read.get_str_borrowed()?,
            data: read.read_remaining_slice_borrowed(MAX_PAYLOAD_SIZE)?,
        })
    }
}
