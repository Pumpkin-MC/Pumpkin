use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;
use tokio::sync::{Mutex, OwnedMutexGuard};

// Inventory.java
#[async_trait]
pub trait Inventory: Send + Sync + Debug {
    fn size(&self) -> usize;

    async fn is_empty(&self) -> bool;

    fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>>;

    async fn remove_stack(&mut self, slot: usize) -> ItemStack;

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack;

    fn get_max_count_per_stack(&self) -> u8 {
        99
    }

    async fn set_stack(&mut self, slot: usize, stack: ItemStack);

    fn mark_dirty(&mut self);

    /*
    boolean canPlayerUse(PlayerEntity player);

    default void onOpen(PlayerEntity player) {
    }

    default void onClose(PlayerEntity player) {
    }
    */

    /// isValid is source
    fn is_valid_slot_for(&self, _slot: usize, _stack: &ItemStack) -> bool {
        true
    }

    fn can_transfer_to(
        &self,
        _hopper_inventory: &dyn Inventory,
        _slot: usize,
        _stack: &ItemStack,
    ) -> bool {
        true
    }

    async fn count(&self, item: &Item) -> u8 {
        let mut count = 0;

        for i in 0..self.size() {
            let slot = self.get_stack(i);
            let stack = slot.lock().await;
            if stack.get_item().id == item.id {
                count += stack.item_count;
            }
        }

        count
    }

    async fn contains_any_predicate(
        &self,
        predicate: &(dyn Fn(OwnedMutexGuard<ItemStack>) -> bool + Sync),
    ) -> bool {
        for i in 0..self.size() {
            let slot = self.get_stack(i);
            let stack = slot.lock_owned().await;
            if predicate(stack) {
                return true;
            }
        }

        false
    }

    async fn contains_any(&self, items: &[Item]) -> bool {
        self.contains_any_predicate(&|stack| !stack.is_empty() && items.contains(stack.get_item()))
            .await
    }

    // TODO: canPlayerUse
}

pub trait Clearable {
    fn clear(&mut self);
}
