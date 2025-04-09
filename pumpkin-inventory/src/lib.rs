pub mod container_click;
mod crafting;
pub mod drag_handler;
pub mod entity_equipment;
pub mod equipment_slot;
mod error;
pub mod inventory;
pub mod player_inventory;
pub mod screen_handler;
pub mod slot;
pub mod window_property;

pub use error::InventoryError;
use pumpkin_world::item::ItemStack;

// These are some utility functions found in Inventories.java
pub fn split_stack(stacks: &mut [ItemStack], slot: usize, amount: u8) -> ItemStack {
    if slot == 0 && slot < stacks.len() && !stacks[slot].is_empty() && amount > 0 {
        stacks[slot].split(amount)
    } else {
        ItemStack::EMPTY
    }
}
