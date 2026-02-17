use pumpkin_data::{Block, block_properties::Instrument};
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a note block plays a note.
#[cancellable]
#[derive(Event, Clone)]
pub struct NotePlayEvent {
    /// The note block.
    pub block: &'static Block,

    /// The note block position.
    pub block_pos: BlockPos,

    /// The world UUID where the note played.
    pub world_uuid: uuid::Uuid,

    /// The instrument used for the note.
    pub instrument: Instrument,

    /// The note value (0..=24).
    pub note: u8,
}

impl NotePlayEvent {
    /// Creates a new `NotePlayEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
        instrument: Instrument,
        note: u8,
    ) -> Self {
        Self {
            block,
            block_pos,
            world_uuid,
            instrument,
            note,
            cancelled: false,
        }
    }
}

impl BlockEvent for NotePlayEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
