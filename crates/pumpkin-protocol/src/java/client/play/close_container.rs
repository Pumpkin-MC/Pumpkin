use crate::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};
use pumpkin_data::packet::clientbound::play::CONTAINER_CLOSE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Notifies the client that a container (inventory, chest, etc.) has been closed.
///
/// This is used by the server to force the player's UI to shut, for example,
/// if the player moves too far away from a chest or if an NPC's trade window
/// is invalidated.
#[java_packet(CONTAINER_CLOSE)]
pub struct CCloseContainer {
    /// The ID of the container window to close.
    ///
    /// A value of 0 usually refers to the player's own inventory, while higher
    /// values refer to active windows opened via previous packets.
    pub sync_id: VarInt,
}
impl CCloseContainer {
    #[must_use]
    pub const fn new(window_id: VarInt) -> Self {
        Self { sync_id: window_id }
    }
}

impl ClientPacket for CCloseContainer {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_container_id(&self.sync_id, version)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CCloseContainer {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let sync_id = bytebuf.get_container_id(version)?;
        Ok(Self { sync_id })
    }
}
