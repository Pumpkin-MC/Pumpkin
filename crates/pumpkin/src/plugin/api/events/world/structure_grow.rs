use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// The type of structure growing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    Oak,
    Spruce,
    Birch,
    Jungle,
    Acacia,
    DarkOak,
    Mangrove,
    Cherry,
    Azalea,
    BrownMushroom,
    RedMushroom,
    Custom,
}

/// An event that occurs when a structure (like a tree or mushroom) grows.
#[cancellable]
#[derive(Event, Clone)]
pub struct StructureGrowEvent {
    /// The origin block position where growth started.
    pub block_pos: BlockPos,

    /// The type of structure growing.
    pub species: TreeType,

    /// Whether bone meal was used.
    pub bone_meal: bool,
}

impl StructureGrowEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, species: TreeType, bone_meal: bool) -> Self {
        Self {
            block_pos,
            species,
            bone_meal,
            cancelled: false,
        }
    }
}
