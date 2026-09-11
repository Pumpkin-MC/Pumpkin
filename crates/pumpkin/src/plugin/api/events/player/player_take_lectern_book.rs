use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player takes a book from a lectern.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerTakeLecternBookEvent {
    /// The player taking the book.
    pub player: Arc<Player>,

    /// The position of the lectern.
    pub block_pos: BlockPos,

    /// The book item stack.
    pub book: ItemStack,
}

impl PlayerEvent for PlayerTakeLecternBookEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
