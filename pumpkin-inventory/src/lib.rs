pub mod container_click;
pub mod crafting;
pub mod drag_handler;
pub mod entity_equipment;
pub mod equipment_slot;
mod error;
pub mod inventory;
pub mod player;
pub mod screen_handler;
pub mod slot;
pub mod sync_handler;
pub mod window_property;

use std::sync::Arc;

pub use error::InventoryError;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

// These are some utility functions found in Inventories.java
pub async fn split_stack(stacks: &[Arc<Mutex<ItemStack>>], slot: usize, amount: u8) -> ItemStack {
    let mut stack = stacks[slot].lock().await;
    if slot < stacks.len() && !stack.is_empty() && amount > 0 {
        stack.split(amount)
    } else {
        ItemStack::EMPTY
    }
}
