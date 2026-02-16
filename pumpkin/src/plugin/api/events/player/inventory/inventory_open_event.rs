use std::sync::Arc;

use crate::{entity::player::Player, plugin::player::inventory::PlayerInventoryEvent};
use pumpkin_data::screen::WindowType;
use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a player opens a container inventory.
///
/// This event is triggered when a player attempts to open a container such as
/// chests, furnaces, hoppers, and other block-based inventories. This event does
/// not trigger when the player opens their own personal inventory.
/// The event can be cancelled to prevent the container from opening.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryOpenEvent {
    /// The player opening the container.
    pub player: Arc<Player>,

    /// The type of container window being opened (chest, furnace, etc.).
    pub window_type: Option<WindowType>,

    /// The synchronization ID used to track this inventory instance.
    pub sync_id: u8,
}

impl InventoryOpenEvent {
    /// Creates a new instance of `InventoryOpenEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player opening the container.
    /// - `window_type`: The type of container window being opened.
    /// - `sync_id`: The synchronization ID for this inventory instance.
    ///
    /// # Returns
    /// A new instance of `InventoryOpenEvent`.
    pub const fn new(player: Arc<Player>, window_type: Option<WindowType>, sync_id: u8) -> Self {
        Self {
            player,
            window_type,
            sync_id,
            cancelled: false,
        }
    }

    /// Gets the window type of the container being opened.
    ///
    /// # Returns
    /// The container window type if available, or `None` if not applicable.
    #[must_use]
    pub const fn get_window_type(&self) -> Option<WindowType> {
        self.window_type
    }
}

impl PlayerInventoryEvent for InventoryOpenEvent {
    fn get_player(&self) -> &std::sync::Arc<crate::entity::player::Player> {
        &self.player
    }
    fn get_sync_id(&self) -> u8 {
        self.sync_id
    }
}
