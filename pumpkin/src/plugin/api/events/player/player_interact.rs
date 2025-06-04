use super::PlayerEvent;
use crate::entity::player::Player;
use crate::world::GetBlockError;
use pumpkin_data::block::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, InvalidBlockFace};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

/// An event that occurs when a `Player` interacts with a `Block` using their hand.
/// This event does not consider interactions through block movement, e.g. pressure plates, tripwire hooks, sculk sensors etc.
///
/// If the event is cancelled, the interaction  will not happen.
///
/// This event contains information about the player, the type of interaction (including whether the player is sneaking or not), the `Block` they are interacting with,
/// the `ItemStack` they are interacting using, the block face (`BlockDirection`) they are interacting with, and the `BlockPos` of the interaction.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractEvent {
    /// The player who attempted to interact.
    pub player: Arc<Player>,

    /// The type of interaction performed.
    pub action: InteractAction,

    /// The block the player is interacting with.
    pub block: Result<Block, GetBlockError>,

    /// The face of the block the player is interacting with
    pub block_direction: Result<BlockDirection, InvalidBlockFace>,

    /// The ItemStack the player is interacting using
    pub item_stack: Arc<Option<ItemStack>>,

    /// The position of the block being interacted with
    pub position: BlockPos,

    /// Is the player sneaking?
    pub sneaking: bool,
}

impl PlayerEvent for PlayerInteractEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}

/// Represents the type of interaction performed by the player.
#[derive(Clone, Debug)]
pub enum InteractAction {
    LeftClick,
    RightClick,
    // TODO: Add and implement a middle-click interaction type.
}
