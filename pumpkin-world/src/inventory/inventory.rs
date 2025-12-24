use crate::item::ItemStack;
use pumpkin_data::item::Item;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use std::any::Any;
use std::pin::Pin;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub type InventoryFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Inventory: Send + Sync + Clearable {
    fn size(&self) -> usize;

    // --- Asynchronous Methods (Using BlockFuture) ---

    fn is_empty(&self) -> InventoryFuture<'_, bool>;

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>>;

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack>;

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack>;

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()>;

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async {})
    }
    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async {})
    }

    fn count<'a>(&'a self, item: &'a Item) -> InventoryFuture<'a, u8> {
        Box::pin(async move {
            let mut count = 0;

            for i in 0..self.size() {
                let slot = self.get_stack(i).await;
                let stack = slot.lock().await;
                if stack.get_item().id == item.id {
                    count += stack.item_count;
                }
            }

            count
        })
    }

    fn contains_any_predicate<'a>(
        &'a self,
        predicate: &'a (dyn Fn(OwnedMutexGuard<ItemStack>) -> bool + Sync),
    ) -> InventoryFuture<'a, bool> {
        Box::pin(async move {
            for i in 0..self.size() {
                let slot = self.get_stack(i).await;
                let stack = slot.lock_owned().await;
                if predicate(stack) {
                    return true;
                }
            }

            false
        })
    }

    fn contains_any<'a>(&'a self, items: &'a [Item]) -> InventoryFuture<'a, bool> {
        Box::pin(async move {
            self.contains_any_predicate(&|stack| {
                !stack.is_empty() && items.contains(stack.get_item())
            })
            .await
        })
    }

    // --- Default Implementation: write_data (Using BlockFuture) ---
    fn write_data(
        &self,
        nbt: &mut NbtCompound,
        stacks: &[Arc<Mutex<ItemStack>>],
        include_empty: bool,
    ) -> InventoryFuture<'_, ()> {
        // Clone for the move block (requires `nbt` and `stacks` to be cloneable/to_owned)
        let nbt = nbt.to_owned();
        let stacks = stacks.to_owned();

        Box::pin(async move {
            let mut slots = Vec::new();
            let mut nbt = nbt;

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
        })
    }

    // --- Synchronous Methods (No Change) ---

    fn get_max_count_per_stack(&self) -> u8 {
        99
    }

    fn mark_dirty(&self) {}

    fn read_data(&self, nbt: &NbtCompound, stacks: &[Arc<Mutex<ItemStack>>]) {
        if let Some(inventory_list) = nbt.get_list("Items") {
            for tag in inventory_list {
                if let Some(item_compound) = tag.extract_compound()
                    && let Some(slot_byte) = item_compound.get_byte("Slot")
                {
                    let slot = slot_byte as usize;
                    if slot < stacks.len()
                        && let Some(item_stack) = ItemStack::read_item_stack(item_compound)
                    {
                        *stacks[slot].try_lock().unwrap() = item_stack;
                    }
                }
            }
        }
    }

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

    fn as_any(&self) -> &dyn Any;
}

pub trait Clearable {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
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
