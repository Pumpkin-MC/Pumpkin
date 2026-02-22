use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's item is repaired via mending.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemMendEvent {
    /// The player whose item is being mended.
    pub player: Arc<Player>,

    /// The item stack being repaired.
    pub item_stack: ItemStack,

    /// The equipment slot that is being repaired.
    pub slot: EquipmentSlot,

    /// The repair amount to apply.
    pub repair_amount: i32,

    /// The UUID of the experience orb that triggered the mend, if known.
    pub orb_uuid: Option<uuid::Uuid>,
}

impl PlayerItemMendEvent {
    /// Creates a new instance of `PlayerItemMendEvent`.
    pub const fn new(
        player: Arc<Player>,
        item_stack: ItemStack,
        slot: EquipmentSlot,
        repair_amount: i32,
        orb_uuid: Option<uuid::Uuid>,
    ) -> Self {
        Self {
            player,
            item_stack,
            slot,
            repair_amount,
            orb_uuid,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerItemMendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
