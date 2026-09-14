use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::client::dialog::Dialog;
use std::sync::Arc;

use crate::entity::player::Player;

use super::super::player::PlayerEvent;

/// An event that occurs when a dialog is shown to a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct DialogShowEvent {
    /// The player receiving the dialog.
    pub player: Arc<Player>,
    /// The dialog being shown.
    pub dialog: Dialog,
}

impl DialogShowEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, dialog: Dialog) -> Self {
        Self {
            player,
            dialog,
            cancelled: false,
        }
    }
}

impl PlayerEvent for DialogShowEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
