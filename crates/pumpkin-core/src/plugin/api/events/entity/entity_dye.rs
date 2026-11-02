use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// The 16 colors available for dyeing entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

/// An event that occurs when an entity is dyed.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDyeEvent {
    /// The ID of the dyed entity.
    pub entity_id: i32,

    /// The color applied.
    pub color: DyeColor,

    /// The player dyeing the entity, if any.
    pub player: Option<Arc<Player>>,
}

impl EntityDyeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, color: DyeColor, player: Option<Arc<Player>>) -> Self {
        Self {
            entity_id,
            color,
            player,
            cancelled: false,
        }
    }
}
