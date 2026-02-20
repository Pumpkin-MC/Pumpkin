use pumpkin_data::data_component_impl::EquipmentSlot;
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

    /// The player's hand slot used for the interaction (`MAIN_HAND` or `OFF_HAND`).
    pub player_slot: EquipmentSlot,

    /// The armor stand equipment slot targeted by the interaction.
    pub armor_stand_slot: EquipmentSlot,
}

impl PlayerArmorStandManipulateEvent {
    #[must_use]
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(
        player: Arc<Player>,
        armor_stand_uuid: Uuid,
        item_key: String,
        armor_stand_item_key: String,
        player_slot: EquipmentSlot,
        armor_stand_slot: EquipmentSlot,
    ) -> Self {
        Self {
            player,
            armor_stand_uuid,
            item_key,
            armor_stand_item_key,
            player_slot,
            armor_stand_slot,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerArmorStandManipulateEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
