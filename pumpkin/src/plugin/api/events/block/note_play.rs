use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a note block plays.
#[cancellable]
#[derive(Event, Clone)]
pub struct NotePlayEvent {
    pub block_pos: BlockPos,
    pub instrument: String,
    pub note: u8,
}

impl NotePlayEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, instrument: String, note: u8) -> Self {
        Self {
            block_pos,
            instrument,
            note,
            cancelled: false,
        }
    }
}
