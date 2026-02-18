use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player edits a book.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerEditBookEvent {
    /// The player editing the book.
    pub player: Arc<Player>,

    /// The inventory slot containing the book.
    pub slot: i32,

    /// The pages being written.
    pub pages: Vec<String>,

    /// The title for the book, if signing.
    pub title: Option<String>,

    /// Whether the player is signing the book.
    pub is_signing: bool,
}

impl PlayerEditBookEvent {
    /// Creates a new instance of `PlayerEditBookEvent`.
    pub const fn new(
        player: Arc<Player>,
        slot: i32,
        pages: Vec<String>,
        title: Option<String>,
        is_signing: bool,
    ) -> Self {
        Self {
            player,
            slot,
            pages,
            title,
            is_signing,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerEditBookEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
