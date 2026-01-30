use std::io::Read;

use pumpkin_data::packet::serverbound::CONFIG_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

/// The maximum allowed size for a plugin message payload (1 MiB).
const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// A packet used for custom communication between the client and server.
///
/// This allows mods, plugins, or proxy
/// software to send proprietary data over the standard Minecraft protocol.
#[java_packet(CONFIG_CUSTOM_PAYLOAD)]
pub struct SPluginMessage {
    /// The name of the channel used to distinguish different types of messages.
    /// Example: `minecraft:brand` or `velocity:main`.
    pub channel: ResourceLocation,
    /// The payload sent by the client.
    pub data: Box<[u8]>,
}

impl ServerPacket for SPluginMessage {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        Ok(Self {
            channel: read.get_resource_location()?,
            data: read.read_remaining_to_boxed_slice(MAX_PAYLOAD_SIZE)?,
        })
    }
}
