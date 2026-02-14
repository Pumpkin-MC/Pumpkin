use std::sync::Arc;

use pumpkin_data::screen::WindowType;
use pumpkin_macros::{Event};
use crate::{entity::player::Player, plugin::player::inventory::PlayerInventoryEvent};


#[derive(Event, Clone)]
pub struct InventoryCloseEvent {
    pub player: Arc<Player>,
    pub window_type: Option<WindowType>, // Know if it's chest/furnace/etc // Verify this is a clone
    pub sync_id: u16,
}

impl InventoryCloseEvent {

    pub const fn new(
        player: Arc<Player>,
        window_type: Option<WindowType>,
        sync_id: u16,
    ) -> Self {
        Self {
            player,
            window_type,
            sync_id,
        }
    }

}

impl PlayerInventoryEvent for InventoryCloseEvent {
    fn get_player(&self) -> &std::sync::Arc<crate::entity::player::Player> {
        &self.player
    }

    fn get_window_type(&self) -> Option<pumpkin_data::screen::WindowType> {
        self.window_type.clone()
    }

    fn get_sync_id(&self) -> u16 {
        self.sync_id
    }
}