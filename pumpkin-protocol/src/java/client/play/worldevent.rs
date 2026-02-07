use pumpkin_data::packet::clientbound::PLAY_LEVEL_EVENT;
use pumpkin_util::math::position::BlockPos;

use pumpkin_macros::java_packet;
use serde::Serialize;

/// Sent by the server to trigger a specific sound or particle effect at a world location.
///
/// This is used for a wide variety of effects, from breaking blocks and firework
/// explosions to splashing water or record playing.
#[derive(Serialize)]
#[java_packet(PLAY_LEVEL_EVENT)]
pub struct CWorldEvent {
    /// The ID of the event to trigger (e.g., 1000 for a bow shoot, 2001 for block break).
    /// Refer to the latest protocol registry for the full list of sound/particle IDs.
    pub event: i32,
    /// The world coordinates where the effect should originate.
    pub location: BlockPos,
    /// Additional metadata associated with the event.
    ///
    /// For example, if breaking a block, this contains the block ID.
    /// For firework particles, it may contain the color or type.
    pub data: i32,
    /// If true, the sound will be played at a constant volume regardless of the
    /// player's distance from the `location`.
    pub disable_relative_volume: bool,
}

impl CWorldEvent {
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
