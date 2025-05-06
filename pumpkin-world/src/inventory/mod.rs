use std::sync::Arc;

use tokio::sync::Mutex;

use crate::item::ItemStack;

#[allow(clippy::module_inception)]
mod inventory;

pub use inventory::*;

// These are some utility functions found in Inventories.java
pub async fn split_stack(stacks: &[Arc<Mutex<ItemStack>>], slot: usize, amount: u8) -> ItemStack {
    let mut stack = stacks[slot].lock().await;
    if slot < stacks.len() && !stack.is_empty() && amount > 0 {
        stack.split(amount)
    } else {
        ItemStack::EMPTY
    }
}
