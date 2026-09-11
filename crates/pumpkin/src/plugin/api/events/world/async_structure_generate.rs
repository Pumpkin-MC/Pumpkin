use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a structure is asynchronously generated.
#[cancellable]
#[derive(Event, Clone)]
pub struct AsyncStructureGenerateEvent {
    pub world_name: String,
    pub structure_name: String,
    pub pos: BlockPos,
}

impl AsyncStructureGenerateEvent {
    #[must_use]
    pub const fn new(world_name: String, structure_name: String, pos: BlockPos) -> Self {
        Self {
            world_name,
            structure_name,
            pos,
            cancelled: false,
        }
    }
}
