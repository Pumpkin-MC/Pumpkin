use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player changes their client language/locale setting.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLocaleChangeEvent {
    /// The player involved.
    pub player: Arc<Player>,

    /// The new locale code (e.g. "`en_us`").
    pub new_locale: String,
}

impl PlayerEvent for PlayerLocaleChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
