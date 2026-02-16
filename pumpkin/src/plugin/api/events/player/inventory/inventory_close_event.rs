use std::sync::Arc;

use crate::{entity::player::Player, plugin::player::inventory::PlayerInventoryEvent};
use pumpkin_data::screen::WindowType;
use pumpkin_macros::Event;

#[derive(Event, Clone)]
pub struct InventoryCloseEvent {
    pub player: Arc<Player>,
    pub identifier: String,
    pub window_type: Option<WindowType>, // Know if it's chest/furnace/etc
    pub sync_id: u8,
}

impl InventoryCloseEvent {
    pub const fn new(player: Arc<Player>,identifier: String, window_type: Option<WindowType>, sync_id: u8) -> Self {
        Self {
            player,
            identifier,
            window_type,
            sync_id,
        }
    }

    #[allow(unused)]
    const fn get_window_type(&self) -> Option<pumpkin_data::screen::WindowType> {
        self.window_type
    }

    #[allow(unused)]
    fn get_identifier(&self) -> &str {
        &self.identifier
    }
}

impl PlayerInventoryEvent for InventoryCloseEvent {
    fn get_player(&self) -> &std::sync::Arc<crate::entity::player::Player> {
        &self.player
    }


    fn get_sync_id(&self) -> u8 {
        self.sync_id
    }

}
