use pumpkin_data::item_stack::ItemStack;
use std::sync::RwLock;

#[expect(clippy::module_inception)]
mod inventory;
mod simple_inventory;

pub use inventory::*;
pub use simple_inventory::*;

/// [`Inventory::update_stack`] for slots behind one lock: the write lock is held from read to
/// write, so a parallel write can't land in between and get overwritten.
pub fn update_locked_stack<S: AsMut<[ItemStack]> + ?Sized>(
    stacks: &RwLock<S>,
    slot: usize,
    update: &mut dyn FnMut(&mut ItemStack) -> bool,
) -> bool {
    stacks
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_mut()
        .get_mut(slot)
        .is_some_and(update)
}

// Utility functions for split_stack
pub fn split_stack_slice(stacks: &mut [ItemStack], slot: usize, amount: u8) -> ItemStack {
    if slot < stacks.len() && !stacks[slot].is_empty() && amount > 0 {
        stacks[slot].split(amount)
    } else {
        ItemStack::EMPTY.clone()
    }
}
