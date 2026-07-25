//! Inventory slot implementations.
//!
//! This module defines the [`Slot`] trait and its implementations. Slots represent
//! individual positions in an inventory that can hold items.
//!
//! # Slot Types
//!
//! - [`NormalSlot`] - A basic inventory slot with no restrictions
//! - [`TakeOnlySlot`] - Output/result slot: take allowed, insert and `PickupAll` sweep blocked
//! - [`ArmorSlot`] - An armor slot that only accepts appropriate item types
//!   (helmets in head slot, chestplates in chest slot, etc.)
//!
//! # Slot Operations
//!
//! Slots support various operations:
//! - Getting/setting the item stack
//! - Checking if items can be inserted
//! - Taking items from the slot
//! - Marking the slot as changed (dirty)
//! - Callbacks for slot interaction events

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
use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::inventory::Inventory;
use tokio::{sync::Mutex, time::timeout};

/// Type alias for async slot operations.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A slot in an inventory.
///
/// The slot trait defines how individual inventory positions behave.
/// Different slot types (normal, armor, result slots) implement this
/// trait to enforce their specific restrictions.
// Slot.java
pub trait Slot: Send + Sync {
    /// Returns the inventory containing this slot.
    fn get_inventory(&self) -> Arc<dyn Inventory>;

    /// Returns the index of this slot within its inventory.
    fn get_index(&self) -> usize;

    /// Sets the protocol ID of this slot.
    fn set_id(&self, index: usize);

    /// Callback for when an item is quick-moved from this slot.
    ///
    /// Used to notify result slots (like crafting output) that they
    /// need to refill their contents.
    ///
    /// # Note
    /// You **MUST** call this after changing the stack and releasing
    /// any locks to avoid deadlocks.
    ///
    /// Also see: [`ScreenHandler::quick_move`](crate::screen_handler::ScreenHandler::quick_move)
    fn on_quick_move_crafted(
        &self,
        _stack: ItemStack,
        _stack_prev: ItemStack,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async {}) // Default implementation
    }

    /// Callback for when an item is taken from this slot.
    ///
    /// Also see: [`safe_take`]
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

    /// Plugin callback for slot clicks.
    ///
    /// Called when a player clicks on this slot. Can be used by
    /// plugins to intercept or modify click behavior.
    fn on_click(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, ()> {
        Box::pin(async {}) // Default implementation
    }

    /// Checks if the given stack can be inserted into this slot.
    fn can_insert<'a>(&'a self, _stack: &'a ItemStack) -> BoxFuture<'a, bool> {
        // Default implementation logic:
        Box::pin(async move { true })
    }

    /// Gets the stack in this slot.
    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        // Default implementation logic:
        Box::pin(async move { self.get_inventory().get_stack(self.get_index()).await })
    }

    /// Gets a copy of the stack in this slot.
    ///
    /// Acquires a lock and returns a clone of the stack.
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

    /// Checks if this slot has a non-empty stack.
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

    /// Sets the stack in this slot.
    ///
    /// # Note
    /// Make sure to drop any locks to the slot stack before calling this.
    fn set_stack(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            self.set_stack_no_callbacks(stack).await;
        })
    }

    /// Sets the stack with previous stack reference.
    ///
    /// Some slots (like armor) need to know the previous stack for callbacks.
    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            self.set_stack_no_callbacks(stack).await;
        })
    }

    /// Sets the stack without calling callbacks.
    fn set_stack_no_callbacks(&self, stack: ItemStack) -> BoxFuture<'_, ()> {
        // Default implementation logic:
        Box::pin(async move {
            let inv = self.get_inventory();
            inv.set_stack(self.get_index(), stack).await;
            self.mark_dirty().await;
        })
    }

    /// Marks this slot as changed.
    ///
    /// Must be implemented by concrete types.
    fn mark_dirty(&self) -> BoxFuture<'_, ()>;

    /// Gets the maximum item count for this slot.
    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        // Default implementation logic:
        Box::pin(async move { self.get_inventory().get_max_count_per_stack() })
    }

    /// Gets the maximum item count for the given stack in this slot.
    fn get_max_item_count_for_stack<'a>(&'a self, stack: &'a ItemStack) -> BoxFuture<'a, u8> {
        // Default implementation logic:
        Box::pin(async move {
            self.get_max_item_count()
                .await
                .min(stack.get_max_stack_size())
        })
    }

    /// Removes a specific amount of items from this slot.
    ///
    /// Mojang name: `remove`
    fn take_stack(&self, amount: u8) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            let inv = self.get_inventory();
            inv.remove_stack_specific(self.get_index(), amount).await
        })
    }

    /// Checks if the player can take items from this slot.
    ///
    /// Mojang name: `mayPickup`
    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        // Default implementation logic:
        Box::pin(async move { true })
    }

    /// Checks if this slot can be modified by the player.
    ///
    /// Mojang name: `allowModification`
    fn allow_modification<'a>(&'a self, player: &'a dyn InventoryPlayer) -> BoxFuture<'a, bool> {
        // Default implementation logic:
        Box::pin(async move {
            self.can_insert(&self.get_cloned_stack().await).await
                && self.can_take_items(player).await
        })
    }

    /// Tries to take a stack in the given range.
    ///
    /// Returns `None` if can't take items or if slot is empty.
    /// For result slots, cannot take partial stacks.
    ///
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

    /// Safely tries to take a stack of items from the slot.
    ///
    /// Returns an empty stack if can't take. Triggers callbacks.
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

            stack.unwrap_or_else(|| ItemStack::EMPTY.clone())
        })
    }

    /// Inserts a stack into this slot.
    ///
    /// Returns any leftover items that couldn't fit.
    fn insert_stack(&self, stack: ItemStack) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            let stack_item_count = stack.item_count;
            self.insert_stack_count(stack, stack_item_count).await
        })
    }

    /// Inserts a specific count from a stack.
    ///
    /// Returns any leftover items.
    fn insert_stack_count(&self, mut stack: ItemStack, count: u8) -> BoxFuture<'_, ItemStack> {
        // Default implementation logic:
        Box::pin(async move {
            if !stack.is_empty() && self.can_insert(&stack).await {
                let stack_mutex = self.get_stack().await;
                let mut stack_self = stack_mutex.lock().await;
                let min_count = count
                    .min(stack.item_count)
                    .min(self.get_max_item_count_for_stack(&stack).await - stack_self.item_count);

                if min_count != 0 {
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
                }
            }
            if stack.is_empty() {
                ItemStack::EMPTY.clone()
            } else {
                stack
            }
        })
    }
}

/// A normal inventory slot.
///
/// Just called `Slot` in vanilla Minecraft. This is the basic
/// slot implementation with no special restrictions.
pub struct NormalSlot {
    /// The inventory containing this slot.
    pub inventory: Arc<dyn Inventory>,
    /// Index of this slot within its inventory.
    pub index: usize,
    /// Protocol ID for this slot (assigned by screen handler).
    pub id: AtomicU8,
}

impl NormalSlot {
    /// Creates a new normal slot.
    ///
    /// # Arguments
    /// - `inventory` - The containing inventory
    /// - `index` - The slot index within the inventory
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

/// Charge logic attached to a [`TakeOnlySlot`] output (merchant/anvil result).
///
/// Every take path that actually removes items from a slot ends in
/// [`Slot::on_take_item`], and every path checks [`Slot::can_take_items`]
/// first. Putting the charge here (rather than in per-action click gates)
/// guarantees that inputs/XP are consumed exactly once per verified take,
/// no matter which action (pickup, shift-click, throw, swap) performed it.
pub trait TakeSlotCharge: Send + Sync {
    /// Whether the player may currently take from the output slot
    /// (e.g. trade not depleted, enough XP levels).
    fn can_take(&self, player: &dyn InventoryPlayer) -> bool;

    /// Charges one taken output stack (consumes inputs, counts uses, XP).
    fn on_take<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()>;
}

/// Result/output slot: items may be taken, but never inserted by the player.
///
/// `allow_modification` is `can_insert && can_take`, so with `can_insert = false`
/// `PickupAll` skips this slot when sweeping matching stacks. That is the correct
/// defense for merchant/anvil output steal (F-INV-03/04); a `slot_index == 2`
/// guard cannot work because `PickupAll` is invoked on a *different* slot.
pub struct TakeOnlySlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    /// Optional charge logic fired on take (merchant/anvil output).
    charge: Option<Arc<dyn TakeSlotCharge>>,
}

impl TakeOnlySlot {
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            charge: None,
        }
    }

    /// Creates a take-only slot that charges via `charge` on every take.
    pub fn with_charge(
        inventory: Arc<dyn Inventory>,
        index: usize,
        charge: Arc<dyn TakeSlotCharge>,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            charge: Some(charge),
        }
    }
}

impl Slot for TakeOnlySlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }

    fn can_take_items(&self, player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        let allowed = self
            .charge
            .as_ref()
            .is_none_or(|charge| charge.can_take(player));
        Box::pin(async move { allowed })
    }

    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if let Some(charge) = &self.charge {
                charge.on_take(player, stack).await;
            }
            self.mark_dirty().await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

/// An armor equipment slot.
///
/// Restricts which items can be placed based on the equipment slot type:
/// - Head: Helmets, skulls, carved pumpkins
/// - Chest: Chestplates, elytra
/// - Legs: Leggings
/// - Feet: Boots
// ArmorSlot.java
pub struct ArmorSlot {
    /// The inventory containing this slot (usually player inventory).
    pub inventory: Arc<dyn Inventory>,
    /// Index of this slot within its inventory.
    pub index: usize,
    /// Protocol ID for this slot (assigned by screen handler).
    pub id: AtomicU8,
    /// The equipment slot type (head, chest, legs, feet, or off-hand).
    pub equipment_slot: EquipmentSlot,
}

impl ArmorSlot {
    /// Creates a new armor slot.
    ///
    /// # Arguments
    /// - `inventory` - The containing inventory
    /// - `index` - The slot index
    /// - `equipment_slot` - The equipment slot type (head, chest, legs, feet)
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

    /// Restricts inserts to appropriate armor types.
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

    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            //TODO: this.entity.onEquipStack(this.equipmentSlot, previousStack, stack);
            self.set_stack_no_callbacks(stack).await;
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }

    /// Armor slots can only hold one item.
    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move { 1 })
    }

    /// TODO: Check for curse of binding enchantment.
    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            // TODO: Check enchantments
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_world::inventory::SimpleInventory;

    #[tokio::test]
    async fn take_only_slot_rejects_insert() {
        let inv = Arc::new(SimpleInventory::new(1));
        let stack = ItemStack::new(16, &Item::EMERALD);
        inv.set_stack(0, stack.clone()).await;
        let slot = TakeOnlySlot::new(inv, 0);
        assert!(
            !slot.can_insert(&stack).await,
            "result/output slots must not accept inserts (blocks PickupAll sweep)"
        );
        assert!(slot.has_stack().await);
        assert_eq!(slot.get_cloned_stack().await.item.id, Item::EMERALD.id);
    }

    #[tokio::test]
    async fn normal_slot_accepts_insert() {
        let inv = Arc::new(SimpleInventory::new(1));
        let stack = ItemStack::new(16, &Item::EMERALD);
        let slot = NormalSlot::new(inv, 0);
        assert!(slot.can_insert(&stack).await);
    }
}
