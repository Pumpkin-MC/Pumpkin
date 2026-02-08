use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_CUSTOM_PAYLOAD;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

/// The maximum allowed size for a plugin message payload (1 MiB).
const MAX_PAYLOAD_SIZE: usize = 1_048_576;

/// Serverbound plugin message (custom payload) sent during Play state.
///
/// Used by mods, plugins, or proxy software (e.g. Velocity, `BungeeCord`)
/// to send proprietary data over the standard Minecraft protocol.
/// Common channels: `minecraft:brand`, `velocity:player_info`.
// TODO: hook up handler in pumpkin/src/net/java/mod.rs
#[java_packet(PLAY_CUSTOM_PAYLOAD)]
pub struct SCustomPayload {
    /// The channel identifier (e.g. `minecraft:brand`).
    pub channel: ResourceLocation,
    /// Raw payload bytes (everything after the channel string).
    pub data: Vec<u8>,
}

impl ServerPacket for SCustomPayload {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let channel = read.get_string()?;

        let mut data = Vec::new();
        read.read_to_end(&mut data)
            .map_err(|e| ReadingError::Incomplete(e.to_string()))?;
        if data.len() > MAX_PAYLOAD_SIZE {
            return Err(ReadingError::TooLarge(format!(
                "Custom payload too large: {} bytes (max {})",
                data.len(),
                MAX_PAYLOAD_SIZE
            )));
        }

        Ok(Self { channel, data })
    }
}
