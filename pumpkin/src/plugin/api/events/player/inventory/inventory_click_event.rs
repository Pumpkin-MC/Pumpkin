use crate::entity::player::Player;
use crate::plugin::player::inventory::PlayerInventoryEvent;
use pumpkin_data::screen::WindowType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInventoryClickEvent {
    pub player: Arc<Player>,
    pub identifier: String,
    pub slot_index: i32,
    pub click_button: i32,
    pub action_type: SlotActionType,
    pub cursor_stack: ItemStack,
    pub clicked_slot_stack: ItemStack,
    pub window_type: Option<WindowType>,
    pub sync_id: u8,
    pub is_player_inventory_click: bool,
}

#[allow(clippy::too_many_arguments)]
impl PlayerInventoryClickEvent {
    pub const fn new(
        player: Arc<Player>,
        identifier: String,
        slot_index: i32,
        click_button: i32,
        action_type: SlotActionType,
        cursor_stack: ItemStack,
        clicked_slot_stack: ItemStack,
        window_type: Option<WindowType>,
        sync_id: u8,
        is_player_inventory_click: bool,
    ) -> Self {
        Self {
            player,
            identifier,
            slot_index,
            click_button,
            action_type,
            cursor_stack,
            clicked_slot_stack,
            window_type,
            sync_id,
            is_player_inventory_click,
            cancelled: false,
        }
    }

    #[allow(unused)]
    const fn get_window_type(&self) -> Option<WindowType> {
        self.window_type
    }

    #[allow(unused)]
    fn get_identifier(&self) -> &str {
        &self.identifier
    }
}

impl PlayerInventoryEvent for PlayerInventoryClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }



    fn get_sync_id(&self) -> u8 {
        self.sync_id
    }
}
