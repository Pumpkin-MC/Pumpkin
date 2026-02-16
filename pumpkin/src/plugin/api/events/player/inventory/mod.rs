pub mod inventory_click_event;
pub mod inventory_close_event;
pub mod inventory_open_event;

use crate::entity::player::Player;
use std::sync::Arc;

pub trait PlayerInventoryEvent: Send + Sync {
    /// Retrieves a reference to the player associated with the event.
    ///
    /// # Returns
    /// A reference to the `Arc<Player>` involved in the event.
    fn get_player(&self) -> &Arc<Player>;

    /// Retrieves the synchronization ID associated with the inventory interaction.
    ///
    /// # Returns
    /// A `u16` representing the synchronization ID for the inventory interaction.
    fn get_sync_id(&self) -> u8;
}
