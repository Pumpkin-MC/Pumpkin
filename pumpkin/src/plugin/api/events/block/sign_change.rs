use crate::entity::player::Player;
use crate::plugin::player::PlayerEvent;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

#[cancellable]
#[derive(Event, Clone)]
/// Event fired when a player changes the text on a sign.
///
/// This event can be cancelled to prevent the sign update.
pub struct SignChangeEvent {
    /// The player who interacts with the sign.
    pub player: Arc<Player>,

    /// The new text content of the sign as a vector of strings (lines).
    pub content: Vec<String>,

    /// The side of the sign that is being edited (front or back).
    pub side: Side,
}

impl SignChangeEvent {
    /// Creates a new `SignChangeEvent`.
    ///
    /// # Parameters
    /// * `player`: The player who is editing the sign.
    /// * `content`: The new text content for the sign lines.
    /// * `side`: The side of the sign being edited.
    ///
    /// # Returns
    /// A new instance of `SignChangeEvent`.
    pub fn new(player: Arc<Player>, content: Vec<String>, side: Side) -> Self {
        Self {
            player,
            content,
            side,
            cancelled: false,
        }
    }

    /// Returns a cloned vector of the sign's new text lines.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        self.content.clone()
    }

    /// Returns the content of a specific line by index.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    #[must_use]
    pub fn line(&self, index: usize) -> String {
        self.content.get(index).unwrap().clone()
    }

    /// Sets or replaces the content of a specific line by index.
    ///
    /// # Parameters
    /// * `index`: The line index to set.
    /// * `line`: The new string content for the line.
    // TODO: clamp this?
    pub fn set_line(&mut self, index: usize, line: String) {
        self.content.insert(index, line);
    }

    /// Returns the side of the sign being edited.
    #[must_use]
    pub fn get_side(&self) -> Side {
        self.side.clone()
    }
}

impl PlayerEvent for SignChangeEvent {
    /// Returns a reference to the player who caused this event.
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}

/// Enum representing which side of a sign is being interacted with.
#[derive(Clone)]
pub enum Side {
    /// The front side of the sign.
    Front,
    /// The back side of the sign.
    Back,
}
