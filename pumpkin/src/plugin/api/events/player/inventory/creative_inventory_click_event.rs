use crate::entity::player::Player;
use crate::plugin::player::inventory::PlayerInventoryEvent;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

/// An event that occurs when a player in creative mode clicks on a slot in their own inventory.
///
/// This event is triggered when a player in creative mode interacts with a slot in their player inventory.
/// Creative mode slot interactions differ from regular inventory clicks as they don't use traditional
/// slot action types. The event can be cancelled to prevent the action from completing.
#[cancellable]
#[derive(Event, Clone)]
pub struct CreativeInventoryClickEvent {
    /// The player performing the click action.
    pub player: Arc<Player>,

    /// The index of the slot that was clicked.
    pub slot_index: i32,

    /// The item stack currently held by the cursor.
    pub cursor_stack: ItemStack,

    /// The item stack in the slot that was clicked.
    pub clicked_slot_stack: ItemStack,

    /// The synchronization ID used to track this inventory instance.
    pub sync_id: u8,
}

impl CreativeInventoryClickEvent {
    /// Creates a new instance of `CreativeInventoryClickEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player performing the click.
    /// - `slot_index`: The index of the slot that was clicked.
    /// - `cursor_stack`: The item stack currently held by the cursor.
    /// - `clicked_slot_stack`: The item stack in the slot that was clicked.
    /// - `sync_id`: The synchronization ID for this inventory instance.
    ///
    /// # Returns
    /// A new instance of `CreativeInventoryClickEvent`.
    pub const fn new(
        player: Arc<Player>,
        slot_index: i32,
        cursor_stack: ItemStack,
        clicked_slot_stack: ItemStack,
        sync_id: u8,
    ) -> Self {
        Self {
            player,
            slot_index,
            cursor_stack,
            clicked_slot_stack,
            sync_id,
            cancelled: false,
        }
    }
}

impl PlayerInventoryEvent for CreativeInventoryClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }

    fn get_sync_id(&self) -> u8 {
        self.sync_id
    }
}
