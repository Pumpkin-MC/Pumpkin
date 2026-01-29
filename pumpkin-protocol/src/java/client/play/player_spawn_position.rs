use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_DEFAULT_SPAWN_POSITION;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::MinecraftVersion};

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};

/// Sent by the server to set the client's default spawn point and compass target.
///
/// This packet updates where the player will respawn upon death (if no bed or anchor is set)
/// and dictates the coordinates that a compass will point toward.
#[java_packet(PLAY_SET_DEFAULT_SPAWN_POSITION)]
pub struct CPlayerSpawnPosition {
    /// The namespaced ID of the dimension (e.g., "minecraft:overworld").
    /// Required for the client to determine if the spawn point is in their current world.
    pub dimension_name: String,
    /// The X, Y, and Z coordinates of the spawn location.
    pub location: BlockPos,
    /// The horizontal rotation (0-360 degrees) the player's camera should face upon respawning.
    pub yaw: f32,
    /// The vertical rotation (-90 to 90 degrees) the player's camera should face upon respawning.
    pub pitch: f32,
}

impl CPlayerSpawnPosition {
    #[must_use]
    pub const fn new(location: BlockPos, yaw: f32, pitch: f32, dimension_name: String) -> Self {
        Self {
            dimension_name,
            location,
            yaw,
            pitch,
        }
    }
}

impl ClientPacket for CPlayerSpawnPosition {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        if *version >= MinecraftVersion::V_1_21_9 {
            write.write_string(&self.dimension_name)?;
        }
        write.write_block_pos(&self.location)?;
        write.write_f32_be(self.yaw)?;

        if *version >= MinecraftVersion::V_1_21_9 {
            write.write_f32_be(self.pitch)?;
        }

        Ok(())
    }
}
