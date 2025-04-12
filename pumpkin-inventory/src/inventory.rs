use std::fmt::Debug;

use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;

// Inventory.java
#[async_trait]
pub trait Inventory: Send + Sync + Debug {
    fn size(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get_stack(&mut self, slot: usize) -> &mut ItemStack;

    fn get_stack_ref(&self, slot: usize) -> &ItemStack;

    fn remove_stack_specific(&mut self, slot: usize, amount: u8) -> ItemStack;

    fn get_max_count_per_stack(&self) -> u8 {
        99
    }

    fn remove_stack(&mut self, slot: usize) -> ItemStack;

    fn set_stack(&mut self, slot: usize, stack: ItemStack);

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

    fn count(&self, item: &Item) -> u8 {
        let mut count = 0;

        for i in 0..self.size() {
            let stack = self.get_stack_ref(i);
            if stack.get_item().id == item.id {
                count += stack.item_count;
            }
        }

        count
    }

    fn contains_any_predicate(&self, predicate: &dyn Fn(&ItemStack) -> bool) -> bool {
        for i in 0..self.size() {
            let stack = self.get_stack_ref(i);
            if predicate(stack) {
                return true;
            }
        }

        false
    }

    fn contains_any(&self, items: &[Item]) -> bool {
        self.contains_any_predicate(&|stack| !stack.is_empty() && items.contains(stack.get_item()))
    }

    fn clone_box(&self) -> Box<dyn Inventory>;

    // TODO: canPlayerUse
}

impl Clone for Box<dyn Inventory> {
    fn clone(&self) -> Box<dyn Inventory> {
        self.clone_box()
    }
}

pub trait Clearable {
    fn clear(&mut self);
}
