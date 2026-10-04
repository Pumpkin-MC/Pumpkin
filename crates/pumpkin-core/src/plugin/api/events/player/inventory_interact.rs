use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_inventory::screen_handler::ClickType;
use pumpkin_macros::{Event, cancellable};

use super::PlayerEvent;

/// Event that is triggered when a player clicks in an inventory.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryClickEvent {
    /// The player who performed the interaction.
    pub player: Arc<Player>,

    /// The window type of the inventory being interacted with.
    pub window_type: Option<WindowType>,

    /// The type of click performed.
    pub click_type: ClickType,

    /// The slot index that was clicked.
    pub slot: i16,

    /// The raw slot number clicked, ready for passing to #getItem(int)
    /// This slot number is unique for the view.
    pub raw_slot: i16,

    /// The item stack that was in the slot being clicked.
    pub clicked_item: Option<ItemStack>,

    /// The current `ItemStack` on the cursor.
    pub cursor: Option<ItemStack>,

    /// If the `ClickType` is `NUMBER_KEY`, this field will return the index of the pressed key (0-8).
    pub hotbar_button: i32,
}

impl InventoryClickEvent {
    /// Creates a new instance of `InventoryClickEvent`.
    ///
    /// # Arguments
    ///
    /// - `player`: A reference-counted pointer to the player who triggered the event.
    /// - `window_type`: The window type of the inventory.
    /// - `click_type`: The type of click.
    /// - `slot`: The slot index that was clicked.
    /// - `raw_slot`: The raw slot index.
    /// - `clicked_item`: The item stack that was in the slot being clicked.
    /// - `cursor`: The item stack on the cursor.
    /// - `hotbar_button`: The hotbar button pressed (0-8).
    ///
    /// # Returns
    ///
    /// A new `InventoryClickEvent` instance with the specified data.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        player: &Arc<Player>,
        window_type: Option<WindowType>,
        click_type: ClickType,
        slot: i16,
        raw_slot: i16,
        clicked_item: Option<ItemStack>,
        cursor: Option<ItemStack>,
        hotbar_button: i32,
    ) -> Self {
        Self {
            player: Arc::clone(player),
            window_type,
            click_type,
            slot,
            raw_slot,
            clicked_item,
            cursor,
            hotbar_button,
            cancelled: false,
        }
    }
}

impl PlayerEvent for InventoryClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
