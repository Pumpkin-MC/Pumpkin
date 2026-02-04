use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;
use uuid::Uuid;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player manipulates an armor stand.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerArmorStandManipulateEvent {
    /// The player who interacted with the armor stand.
    pub player: Arc<Player>,

    /// The armor stand's UUID.
    pub armor_stand_uuid: Uuid,

    /// The item key in the player's hand (e.g., "minecraft:stone").
    pub item_key: String,

    /// The item key currently in the armor stand slot (if known).
    pub armor_stand_item_key: String,

    /// The equipment slot being manipulated (e.g., "HAND", "OFF_HAND").
    pub slot: String,
}

impl PlayerArmorStandManipulateEvent {
    #[must_use]
    pub fn new(
        player: Arc<Player>,
        armor_stand_uuid: Uuid,
        item_key: String,
        armor_stand_item_key: String,
        slot: String,
    ) -> Self {
        Self {
            player,
            armor_stand_uuid,
            item_key,
            armor_stand_item_key,
            slot,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerArmorStandManipulateEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
