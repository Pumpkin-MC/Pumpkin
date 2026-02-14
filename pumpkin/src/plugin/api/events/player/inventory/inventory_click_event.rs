use std::sync::Arc;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::container_click::ClickType;
use pumpkin_macros::{cancellable, Event};
use pumpkin_world::item::ItemStack;
use crate::entity::player::Player;
use crate::plugin::player::inventory::PlayerInventoryEvent;

#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInventoryClickEvent {
    pub player: Arc<Player>,
    pub slot_index: i32,
    pub click_button: i32,
    pub click_type: ClickType,
    pub cursor_stack: ItemStack,        // Verify this is a clone
    pub clicked_slot_stack: ItemStack,  // Verify this is a clone
    pub window_type: Option<WindowType>, // Know if it's chest/furnace/etc // Verify this is a clone
    pub sync_id: u16,
    pub is_player_inventory_click: bool,
}

impl PlayerInventoryClickEvent {

    pub const fn new(
        player: Arc<Player>,
        slot_index: i32,
        click_button: i32,
        click_type: ClickType,
        cursor_stack: ItemStack,
        clicked_slot_stack: ItemStack,
        window_type: Option<WindowType>,
        sync_id: u16,
        is_player_inventory_click: bool,
    ) -> Self {
        Self {
            player,
            slot_index,
            click_button,
            click_type,
            cursor_stack,
            clicked_slot_stack,
            window_type,
            sync_id,
            is_player_inventory_click,
            cancelled: false
        }
    }

}

impl PlayerInventoryEvent for PlayerInventoryClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }

    fn get_window_type(&self) -> Option<WindowType> {
        self.window_type
    }

    fn get_sync_id(&self) -> u16 {
        self.sync_id.clone()
    }
}