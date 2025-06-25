use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::item::ItemStack;
use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use tokio::sync::{Mutex, OwnedMutexGuard};

// Inventory.java
#[async_trait]
pub trait Inventory: Send + Sync + Debug + Clearable {
    fn size(&self) -> usize;

    async fn is_empty(&self) -> bool;

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>>;

    async fn remove_stack(&self, slot: usize) -> ItemStack;

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack;

    fn get_max_count_per_stack(&self) -> u8 {
        99
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack);

    fn mark_dirty(&self) {}

    async fn write_data(
        &self,
        nbt: &mut pumpkin_nbt::compound::NbtCompound,
        stacks: &[Arc<Mutex<ItemStack>>],
        include_empty: bool,
    ) {
        let mut slots = Vec::new();

        for (i, item) in stacks.iter().enumerate() {
            let stack = item.lock().await;
            if !stack.is_empty() {
                let mut item_compound = NbtCompound::new();
                item_compound.put_byte("Slot", i as i8);
                stack.write_item_stack(&mut item_compound);
                slots.push(NbtTag::Compound(item_compound));
            }
        }

        if !include_empty && slots.is_empty() {
            return;
        }

        nbt.put("Items", NbtTag::List(slots));
    }

    fn read_data(
        &self,
        nbt: &pumpkin_nbt::compound::NbtCompound,
        stacks: &[Arc<Mutex<ItemStack>>],
    ) {
        if let Some(inventory_list) = nbt.get_list("Items") {
            for tag in inventory_list {
                if let Some(item_compound) = tag.extract_compound() {
                    if let Some(slot_byte) = item_compound.get_byte("Slot") {
                        let slot = slot_byte as usize;
                        if slot < stacks.len() {
                            if let Some(item_stack) = ItemStack::read_item_stack(item_compound) {
                                // This won't error cause it's only called on initialization
                                *stacks[slot].try_lock().unwrap() = item_stack;
                            }
                        }
                    }
                }
            }
        }
    }

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
            let slot = self.get_stack(i).await;
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
            let slot = self.get_stack(i).await;
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

#[async_trait]
pub trait Clearable {
    async fn clear(&self);
}

pub struct ComparableInventory(pub Arc<dyn Inventory>);

impl PartialEq for ComparableInventory {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ComparableInventory {}

impl Hash for ComparableInventory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0);
        ptr.hash(state);
    }
}
