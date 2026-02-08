use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

/// An event that occurs when a player requests tab completion.
///
/// If the event is cancelled, no completions will be sent.
///
/// Matches Bukkit's `TabCompleteEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct TabCompleteEvent {
    /// The player requesting tab completion.
    pub player: Arc<Player>,

    /// The partial command/chat being completed.
    pub buffer: String,

    /// The list of completions to send.
    pub completions: Vec<String>,
}

impl TabCompleteEvent {
    #[must_use]
    pub fn new(player: Arc<Player>, buffer: String, completions: Vec<String>) -> Self {
        Self {
            player,
            buffer,
            completions,
            cancelled: false,
        }
    }
}
