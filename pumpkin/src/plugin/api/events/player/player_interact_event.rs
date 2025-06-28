use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use super::PlayerEvent;

/// Event that is triggered when a player interacts with a block or air.
///
/// This event includes information about the player, the action performed,
/// the item in the player's hand, the block interacted with, and the position clicked (if any).
/// It can be cancelled to prevent the default interaction behavior.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractEvent {
    /// The player who performed the interaction.
    pub player: Arc<Player>,

    /// The type of action the player performed.
    pub action: InteractAction,

    /// The position of the block that was clicked, if any.
    pub clicked_pos: Option<BlockPos>,

    /// The item in the player's hand at the time of interaction.
    pub item: Arc<Mutex<ItemStack>>,

    /// The block that was interacted with.
    pub block: Block,
}

impl PlayerInteractEvent {
    /// Creates a new instance of `PlayerInteractEvent`.
    ///
    /// # Arguments
    ///
    /// - `player`: A reference-counted pointer to the player who triggered the event.
    /// - `action`: The type of interaction performed.
    /// - `item`: A reference-counted, mutex-protected item stack used during the interaction.
    /// - `block`: The block that was interacted with.
    /// - `clicked_pos`: The optional position of the block that was clicked.
    ///
    /// # Returns
    ///
    /// A new `PlayerInteractEvent` instance with the specified data.
    pub fn new(
        player: &Arc<Player>,
        action: InteractAction,
        item: &Arc<Mutex<ItemStack>>,
        block: Block,
        clicked_pos: Option<BlockPos>,
    ) -> Self {
        Self {
            player: Arc::clone(player),
            action,
            item: Arc::clone(item),
            block,
            clicked_pos,
            cancelled: false,
        }
    }
}

/// Enum representing possible player interaction actions.
#[derive(Clone, PartialEq, Eq)]
pub enum InteractAction {
    /// Left-clicking the air
    LeftClickBlock,

    /// Left-clicking a block
    LeftClickAir,

    /// Right-clicking the air
    RightClickAir,

    /// Right-clicking a block
    RightClickBlock,
}

impl InteractAction {
    /// Gets whether this action is a result of a left click.
    #[must_use]
    #[inline]
    pub fn is_left_click(&self) -> bool {
        Self::LeftClickAir.eq(self) || Self::LeftClickBlock.eq(self)
    }

    /// Gets whether this action is a result of a right click.
    #[must_use]
    #[inline]
    pub fn is_right_click(&self) -> bool {
        Self::RightClickAir.eq(self) || Self::RightClickBlock.eq(self)
    }
}

impl PlayerEvent for PlayerInteractEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
