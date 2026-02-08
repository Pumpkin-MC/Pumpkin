use std::io::Write;

use crate::{
    ClientPacket,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::packet::clientbound::PLAY_LEVEL_EVENT;
use pumpkin_data::world::WorldEvent;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::MinecraftVersion;

/// Triggers a specific sound or particle effect at a world location.
///
/// This packet handles a wide variety of "world-level" events, such as
/// block breaking particles, firework explosions, or ambient sounds
/// like doors opening and portals humming.
#[java_packet(PLAY_LEVEL_EVENT)]
pub struct CLevelEvent {
    /// The ID of the event to trigger.
    /// Event IDs are generally divided into Sound Events (1000s) and
    /// Particle/Visual Events (2000s).
    pub event: i32,
    /// The world coordinates where the event occurs.
    pub location: BlockPos,
    /// Event-specific data (e.g., the block ID for break particles
    /// or the direction of a smoke puff).
    pub data: i32,
    /// If true, the sound is played at a constant volume regardless of
    /// the player's distance from the `location`.
    pub disable_relative_volume: bool,
}

impl CLevelEvent {
    #[must_use]
    pub const fn new(
        event: i32,
        location: BlockPos,
        data: i32,
        disable_relative_volume: bool,
    ) -> Self {
        Self {
            event,
            location,
            data,
            disable_relative_volume,
        }
    }
}

impl ClientPacket for CLevelEvent {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_i32_be(self.event)?;
        write.write_block_pos(&self.location)?;

        let data = if self.event == WorldEvent::BlockBroken as i32 {
            u16::try_from(self.data).map_or(self.data, |state_id| {
                i32::from(remap_block_state_for_version(state_id, *version))
            })
        } else {
            self.data
        };
        write.write_i32_be(data)?;
        write.write_bool(self.disable_relative_volume)?;

        Ok(())
    }
}
