use pumpkin_data::packet::serverbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

/// This packet signals to the server that the client is ready to transition
/// from the `Configuration` state to the `Play` state.
#[java_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct SAcknowledgeFinishConfig;

impl<'a> ServerPacket<'a> for SAcknowledgeFinishConfig {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}
