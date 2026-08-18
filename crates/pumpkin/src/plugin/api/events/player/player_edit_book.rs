use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player edits or signs a book.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerEditBookEvent {
    /// The player editing the book.
    pub player: Arc<Player>,

    /// The inventory slot of the book.
    pub slot: u32,

    /// The book pages.
    pub pages: Vec<String>,

    /// The book title, if signing.
    pub title: Option<String>,

    /// Whether the book is being signed.
    pub signing: bool,
}

impl PlayerEvent for PlayerEditBookEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
