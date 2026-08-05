use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// The type of portal created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalType {
    Nether,
    End,
    Custom,
}

/// An event that occurs when a nether or end portal frame/block is created.
#[cancellable]
#[derive(Event, Clone)]
pub struct PortalCreateEvent {
    /// The block position of the portal creation.
    pub block_pos: BlockPos,

    /// The type of portal created.
    pub portal_type: PortalType,
}

impl PortalCreateEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, portal_type: PortalType) -> Self {
        Self {
            block_pos,
            portal_type,
            cancelled: false,
        }
    }
}
