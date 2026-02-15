use std::sync::Arc;

use crate::{entity::player::Player, plugin::player::inventory::PlayerInventoryEvent};
use pumpkin_data::screen::WindowType;
use pumpkin_macros::{Event, cancellable};

#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryOpenEvent {
    pub player: Arc<Player>,
    pub window_type: Option<WindowType>, // Know if it's chest/furnace/etc
    pub sync_id: u16,
}

impl InventoryOpenEvent {
    pub const fn new(player: Arc<Player>, window_type: Option<WindowType>, sync_id: u16) -> Self {
        Self {
            player,
            window_type,
            sync_id,
            cancelled: false,
        }
    }
}

impl PlayerInventoryEvent for InventoryOpenEvent {
    fn get_player(&self) -> &std::sync::Arc<crate::entity::player::Player> {
        &self.player
    }

    fn get_window_type(&self) -> Option<pumpkin_data::screen::WindowType> {
        self.window_type
    }

    fn get_sync_id(&self) -> u16 {
        self.sync_id
    }
}
