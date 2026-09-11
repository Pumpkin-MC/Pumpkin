use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player names an entity using a name tag.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerNameEntityEvent {
    /// The player naming the entity.
    pub player: Arc<Player>,

    /// The ID of the named entity.
    pub entity_id: i32,

    /// The custom name applied.
    pub name: TextComponent,
}

impl PlayerEvent for PlayerNameEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
