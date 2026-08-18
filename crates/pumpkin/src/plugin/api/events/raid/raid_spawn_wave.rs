use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a wave of a raid spawns.
#[cancellable]
#[derive(Event, Clone)]
pub struct RaidSpawnWaveEvent {
    /// Wave number.
    pub wave: u32,
    /// Spawn position.
    pub pos: BlockPos,
}

impl RaidSpawnWaveEvent {
    #[must_use]
    pub const fn new(wave: u32, pos: BlockPos) -> Self {
        Self {
            wave,
            pos,
            cancelled: false,
        }
    }
}
