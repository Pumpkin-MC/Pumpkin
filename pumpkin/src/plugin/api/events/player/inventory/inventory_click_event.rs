use crate::entity::player::Player;
use crate::plugin::player::inventory::PlayerInventoryEvent;
use pumpkin_data::screen::WindowType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

/// An event that occurs when a player clicks on a slot in an inventory.
///
/// This event is triggered when a player interacts with any slot in an inventory,
/// including picking up items, placing items, swapping items, or performing other
/// inventory actions. The event can be cancelled to prevent the action from completing.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInventoryClickEvent {
    /// The player performing the click action.
    pub player: Arc<Player>,

    /// The identifier for the inventory being clicked.
    pub identifier: String,

    /// The index of the slot that was clicked.
    pub slot_index: i32,

    /// The mouse button used for the click (0 for left, 1 for right, etc.).
    pub click_button: i32,

    /// The type of action being performed (pickup, swap, drop, etc.).
    pub action_type: SlotActionType,

    /// The item stack currently held by the cursor.
    pub cursor_stack: ItemStack,

    /// The item stack in the slot that was clicked.
    pub clicked_slot_stack: ItemStack,

    /// The type of window being interacted with (chest, furnace, etc.), if applicable.
    pub window_type: Option<WindowType>,

    /// The synchronization ID used to track this inventory instance.
    pub sync_id: u8,

    /// Whether this click occurred in the player's own inventory.
    pub is_player_inventory_click: bool,
}

#[allow(clippy::too_many_arguments)]
impl PlayerInventoryClickEvent {
    /// Creates a new instance of `PlayerInventoryClickEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player performing the click.
    /// - `identifier`: The identifier for the inventory being clicked.
    /// - `slot_index`: The index of the slot that was clicked.
    /// - `click_button`: The mouse button used for the click.
    /// - `action_type`: The type of action being performed.
    /// - `cursor_stack`: The item stack currently held by the cursor.
    /// - `clicked_slot_stack`: The item stack in the slot that was clicked.
    /// - `window_type`: The type of window being interacted with, if applicable.
    /// - `sync_id`: The synchronization ID for this inventory instance.
    /// - `is_player_inventory_click`: Whether this click occurred in the player's own inventory.
    ///
    /// # Returns
    /// A new instance of `PlayerInventoryClickEvent`.
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

    /// Gets the window type of the inventory being clicked.
    ///
    /// # Returns
    /// The window type if available, or `None` if not applicable.
    #[allow(unused)]
    const fn get_window_type(&self) -> Option<WindowType> {
        self.window_type
    }

    /// Gets the identifier of the inventory being clicked.
    ///
    /// # Returns
    /// A string slice containing the identifier.
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
