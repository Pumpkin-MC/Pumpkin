use async_trait::async_trait;
use pumpkin_data::item::Item;
use pumpkin_world::item::ItemStack;

// Inventory.java
#[async_trait]
pub trait Inventory: Send + Sync + Clone + Sized + IntoIterator<Item = ItemStack> {
    fn size(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn get_stack(&mut self, slot: usize) -> &mut ItemStack;

    fn get_stack_ref(&self, slot: usize) -> &ItemStack;

    fn remove_stack_specific(&mut self, slot: usize, amount: u8) -> ItemStack;

    fn get_max_count_per_stack() -> u8 {
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

    fn can_transfer_to<I: Inventory>(
        &self,
        _hopper_inventory: I,
        _slot: usize,
        _stack: &ItemStack,
    ) -> bool {
        true
    }

    fn count(&self, item: &Item) -> u8 {
        let mut count = 0;

        for stack in self.clone().into_iter() {
            if stack.get_item().id == item.id {
                count += stack.item_count;
            }
        }

        count
    }

    fn contains_any_predicate(&self, predicate: impl Fn(&ItemStack) -> bool) -> bool {
        for stack in self.clone().into_iter() {
            if predicate(&stack) {
                return true;
            }
        }

        false
    }

    fn contains_any(&self, items: &[Item]) -> bool {
        self.contains_any_predicate(move |stack| {
            !stack.is_empty() && items.contains(stack.get_item())
        })
    }

    // TODO: canPlayerUse
}

pub trait Clearable {
    fn clear(&mut self);
}

pub struct InventoryIterator<I: Inventory> {
    inventory: I,
    index: usize,
    size: usize,
}

impl<T> InventoryIterator<T>
where
    T: Inventory,
{
    pub fn new(inventory: T) -> Self {
        InventoryIterator {
            size: inventory.size(),
            inventory,
            index: 0,
        }
    }
}

impl<T> Iterator for InventoryIterator<T>
where
    T: Inventory,
{
    type Item = ItemStack;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            let item = self.inventory.get_stack_ref(self.index).clone();
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
