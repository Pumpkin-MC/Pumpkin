use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use crate::screen_handler::InventoryPlayer;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::{sync::Mutex, time::timeout};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Slot.java
// This is a trait due to crafting slots being a thing
pub trait Slot: Send + Sync {
    fn get_inventory(&self) -> Arc<dyn Inventory>;

    fn get_index(&self) -> usize;

    fn set_id(&self, index: usize);

    /// Used to notify result slots that they need to update their contents. (e.g. refill)
    /// Note that you **MUST** call this after changing the stack in the slot, and releasing any
    /// locks to the stack to avoid deadlocks.
    ///
    /// Also see: `ScreenHandler::quick_move`
    fn on_quick_move_crafted(
        &self,
        _stack: ItemStack,
        _stack_prev: ItemStack,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async {}) // Default implementation
    }

    /// Callback for when an item is taken from the slot.
    ///
    /// Also see: `safe_take`
    fn on_take_item<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        // Default implementation logic:
        Box::pin(async move {
            self.mark_dirty().await;
        })
    }

    // Used for plugins
    fn on_click(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, ()> {
        Box::pin(async {}) // Default implementation
    }

    fn can_insert<'a>(&'a self, _stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        // Default implementation logic:
        Box::pin(async move { true })
    }

    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        // Default implementation logic:
        Box::pin(async move { self.get_inventory().get_stack(self.get_index()).await })
    }

    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            let stack = self.get_stack().await;
            let lock = timeout(Duration::from_secs(5), stack.lock())
                .await
                .expect("Timed out while trying to acquire lock");

            lock.clone()
        })
    }

    fn has_stack(&self) -> BoxFuture<'_, bool> {
        // Default implementation logic:
        Box::pin(async move {
            let inv = self.get_inventory();
            !inv.get_stack(self.get_index())
                .await
                .lock()
                .await
                .is_empty()
        })
    }

    /// Make sure to drop any locks to the slot stack before calling this
    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            self.set_stack_no_callbacks(stack).await;
        })
    }

    /// Changes the stack in the slot with the given `stack`.
    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            self.set_stack_no_callbacks(stack).await;
        })
    }

    fn set_stack_no_callbacks(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            let inv = self.get_inventory();
            inv.set_stack(self.get_index(), stack).await;
            self.mark_dirty().await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()>; // This method must be implemented by concrete types

    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        // Default implementation logic:
        Box::pin(async move { self.get_inventory().get_max_count_per_stack() })
    }

    fn get_max_item_count_for_stack<'a>(&'a self, stack: &'a ItemStack) -> BoxFuture<'a, u8> {
        // Default implementation logic:
        Box::pin(async move {
            self.get_max_item_count()
                .await
                .min(stack.get_max_stack_size())
        })
    }

    /// Removes a specific amount of items from the slot.
    ///
    /// Mojang name: `remove`
    fn take_stack(&self, amount: u8) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            let inv = self.get_inventory();
            inv.remove_stack_specific(self.get_index(), amount).await
        })
    }

    /// Mojang name: `mayPickup`
    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        // Default implementation logic:
        Box::pin(async move { true })
    }

    /// Mojang name: `allowModification`
    fn allow_modification<'a>(&'a self, player: &'a dyn InventoryPlayer) -> BoxFuture<'a, bool> {
        // Default implementation logic:
        Box::pin(async move {
            self.can_insert(&self.get_cloned_stack().await).await
                && self.can_take_items(player).await
        })
    }

    /// Mojang name: `tryRemove`
    fn try_take_stack_range<'a>(
        &'a self,
        min: u8,
        max: u8,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<ItemStack>> {
        // Default implementation logic:
        Box::pin(async move {
            if !self.can_take_items(player).await {
                return None;
            }
            if !self.allow_modification(player).await
                && self.get_cloned_stack().await.item_count > max
            {
                // If the slot is not allowed to be modified, we cannot take a partial stack from it.
                return None;
            }
            let min = min.min(max);
            let stack = self.take_stack(min).await;

            if stack.is_empty() {
                None
            } else {
                if self.get_cloned_stack().await.is_empty() {
                    self.set_stack_prev(ItemStack::EMPTY.clone(), stack.clone())
                        .await;
                }

                Some(stack)
            }
        })
    }

    /// Safely tries to take a stack of items from the slot, returning `None` if the stack is empty.
    /// Considering such as result slots, as their stacks cannot split.
    ///
    /// Mojang name: `safeTake`
    fn safe_take<'a>(
        &'a self,
        min: u8,
        max: u8,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, ItemStack> {
        Box::pin(async move {
            let stack = self.try_take_stack_range(min, max, player).await;

            if let Some(stack) = &stack {
                self.on_take_item(player, stack).await;
            }

            stack.unwrap_or(ItemStack::EMPTY.clone())
        })
    }

    fn insert_stack(&self, stack: ItemStack) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            let stack_item_count = stack.item_count;
            self.insert_stack_count(stack, stack_item_count).await
        })
    }

    fn insert_stack_count(&self, mut stack: ItemStack, count: u8) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            if !stack.is_empty() && self.can_insert(&stack).await {
                let stack_mutex = self.get_stack().await;
                let mut stack_self = stack_mutex.lock().await;
                let min_count = count
                    .min(stack.item_count)
                    .min(self.get_max_item_count_for_stack(&stack).await - stack_self.item_count);

                if min_count == 0 {
                    stack
                } else {
                    if stack_self.is_empty() {
                        drop(stack_self);
                        self.set_stack(stack.split(min_count)).await;
                    } else if stack.are_items_and_components_equal(&stack_self) {
                        stack.decrement(min_count);
                        stack_self.increment(min_count);
                        let cloned_stack = stack_self.clone();
                        drop(stack_self);
                        self.set_stack(cloned_stack).await;
                    }

                    stack
                }
            } else {
                stack
            }
        })
    }
}

/// Just called Slot in Vanilla
pub struct NormalSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl NormalSlot {
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}
impl Slot for NormalSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

// ArmorSlot.java
pub struct ArmorSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub equipment_slot: EquipmentSlot,
}

impl ArmorSlot {
    pub fn new(inventory: Arc<dyn Inventory>, index: usize, equipment_slot: EquipmentSlot) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            equipment_slot,
        }
    }
}

impl Slot for ArmorSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move { 1 })
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            //TODO: this.entity.onEquipStack(this.equipmentSlot, previousStack, stack);
            self.set_stack_no_callbacks(stack).await;
        })
    }

    fn can_insert<'a>(&'a self, stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self.equipment_slot {
                EquipmentSlot::Head(_) => {
                    stack.is_helmet() || stack.is_skull() || stack.item == &Item::CARVED_PUMPKIN
                }
                EquipmentSlot::Chest(_) => stack.is_chestplate() || stack.item == &Item::ELYTRA,
                EquipmentSlot::Legs(_) => stack.is_leggings(),
                EquipmentSlot::Feet(_) => stack.is_boots(),
                _ => true,
            }
        })
    }

    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            // TODO: Check enchantments
            true
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}
