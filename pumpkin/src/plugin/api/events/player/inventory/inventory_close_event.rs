use std::sync::Arc;

use crate::{entity::player::Player, plugin::player::inventory::PlayerInventoryEvent};
use pumpkin_data::screen::WindowType;
use pumpkin_macros::Event;

/// An event that occurs when a player closes a container inventory.
///
/// This event is triggered when a player closes a container such as
/// chests, furnaces, hoppers, and other inventories.
#[derive(Event, Clone)]
pub struct InventoryCloseEvent {
    /// The player closing the container.
    pub player: Arc<Player>,

    /// The identifier for the inventory being closed.
    pub identifier: String,

    /// The type of container window being closed (chest, furnace, etc.).
    pub window_type: Option<WindowType>,

    /// The synchronization ID used to track this inventory instance.
    pub sync_id: u8,
}

impl InventoryCloseEvent {
    /// Creates a new instance of `InventoryCloseEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player closing the container.
    /// - `identifier`: The identifier for the inventory being closed.
    /// - `window_type`: The type of container window being closed.
    /// - `sync_id`: The synchronization ID for this inventory instance.
    ///
    /// # Returns
    /// A new instance of `InventoryCloseEvent`.
    pub const fn new(
        player: Arc<Player>,
        identifier: String,
        window_type: Option<WindowType>,
        sync_id: u8,
    ) -> Self {
        Self {
            player,
            identifier,
            window_type,
            sync_id,
        }
    }

    /// Gets the window type of the container being closed.
    ///
    /// # Returns
    /// The container window type if available, or `None` if not applicable.
    #[allow(unused)]
    const fn get_window_type(&self) -> Option<pumpkin_data::screen::WindowType> {
        self.window_type
    }

    /// Gets the identifier of the inventory being closed.
    ///
    /// # Returns
    /// A string slice containing the identifier.
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
