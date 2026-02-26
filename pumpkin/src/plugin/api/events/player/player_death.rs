use pumpkin_data::damage::DamageType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player dies.
///
/// Cancelling this event prevents the death (restores health, skips loot drop and death screen).
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDeathEvent {
    pub player: Arc<Player>,
    pub damage_type: DamageType,
    pub death_message: TextComponent,
}

impl PlayerDeathEvent {
    pub fn new(
        player: Arc<Player>,
        damage_type: DamageType,
        death_message: TextComponent,
    ) -> Self {
        Self {
            player,
            damage_type,
            death_message,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerDeathEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
