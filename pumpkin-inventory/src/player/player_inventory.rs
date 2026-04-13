//! Player inventory implementation.
//!
//! This module implements the player's inventory, which consists of:
//! - 36 main inventory slots (3 rows of 9 + hotbar)
//! - Equipment slots (armor + off-hand)
//!
//! The first 9 slots of the main inventory are the hotbar (accessible with number keys).
//! Slots 0-35 are the main inventory, with slot 40 being the off-hand slot.

use crate::entity_equipment::EntityEquipment;
use crate::screen_handler::InventoryPlayer;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::Hand;
use pumpkin_world::inventory::split_stack;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};
use std::any::Any;
use std::array::from_fn;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::Mutex;
use tracing::warn;

/// The player's inventory.
///
/// Contains 36 main inventory slots (hotbar + main storage) plus
/// equipment slots accessed through [`EntityEquipment`].
pub struct PlayerInventory {
    /// The 36 main inventory slots (slots 0-35).
    ///
    /// The first 9 slots (0-8) are the hotbar, the remaining 27 (9-35) are the main storage.
    pub main_inventory: [Arc<Mutex<ItemStack>>; Self::MAIN_SIZE],
    /// Mapping of slot indices to equipment slot types.
    ///
    /// Used to identify which slots correspond to armor and off-hand equipment.
    pub equipment_slots: Arc<HashMap<usize, EquipmentSlot>>,
    /// The currently selected hotbar slot index (0-8).
    selected_slot: AtomicU8,
    /// The entity equipment storage for armor and off-hand items.
    ///
    /// This is separate from the main inventory and is rendered on the player model.
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
}

impl PlayerInventory {
    /// Size of the main inventory (36 slots: 27 storage + 9 hotbar).
    pub const MAIN_SIZE: usize = 36;
    /// Size of the hotbar (9 slots).
    const HOTBAR_SIZE: usize = 9;
    /// Slot index for the off-hand (40).
    pub const OFF_HAND_SLOT: usize = 40;

    /// Creates a new player inventory.
    ///
    /// # Arguments
    /// - `entity_equipment` - The entity equipment storage for armor/off-hand
    /// - `equipment_slots` - Mapping of slot indices to equipment slots
    // TODO: Add inventory load from nbt
    pub fn new(
        entity_equipment: Arc<Mutex<EntityEquipment>>,
        equipment_slots: Arc<HashMap<usize, EquipmentSlot>>,
    ) -> Self {
        Self {
            // Normal syntax can't be used here because Arc doesn't implement Copy
            main_inventory: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            equipment_slots,
            selected_slot: AtomicU8::new(0),
            entity_equipment,
        }
    }

    /// Gets the item in the currently selected hotbar slot.
    ///
    /// This is the item the player is currently holding in their main hand.
    ///
    /// Mojang name: `getSelectedStack`
    pub fn held_item(&self) -> Arc<Mutex<ItemStack>> {
        self.main_inventory
            .get(self.get_selected_slot() as usize)
            .unwrap()
            .clone()
    }

    /// Gets the item in the specified hand.
    ///
    /// # Arguments
    /// - `hand` - Which hand to get the item from
    pub async fn get_stack_in_hand(&self, hand: Hand) -> Arc<Mutex<ItemStack>> {
        match hand {
            Hand::Left => self.off_hand_item().await,
            Hand::Right => self.held_item(),
        }
    }

    /// Gets the item in the off-hand.
    ///
    /// Mojang name: `getOffHandStack`
    pub async fn off_hand_item(&self) -> Arc<Mutex<ItemStack>> {
        let slot = self.equipment_slots.get(&Self::OFF_HAND_SLOT).unwrap();
        self.entity_equipment.lock().await.get(slot)
    }

    /// Swaps the items between main hand and off-hand.
    ///
    /// # Returns
    /// The new main hand item and new off-hand item.
    pub async fn swap_item(&self) -> (ItemStack, ItemStack) {
        let slot = self.equipment_slots.get(&Self::OFF_HAND_SLOT).unwrap();
        let mut equipment = self.entity_equipment.lock().await;
        let binding = self.held_item();
        let mut main_hand_item = binding.lock().await;
        let off_hand_item = main_hand_item.clone();
        *main_hand_item = equipment.put(slot, off_hand_item.clone()).await;
        (main_hand_item.clone(), off_hand_item)
    }

    /// Checks if a slot index is a valid hotbar slot.
    #[must_use]
    pub const fn is_valid_hotbar_index(slot: usize) -> bool {
        slot < Self::HOTBAR_SIZE
    }

    /// Adds a stack to any available slot, prioritizing stacking with existing items.
    async fn add_stack(&self, stack: ItemStack) -> usize {
        let mut slot_index = self.get_occupied_slot_with_room_for_stack(&stack).await;

        if slot_index == -1 {
            slot_index = self.get_empty_slot().await;
        }

        if slot_index == -1 {
            stack.item_count as usize
        } else {
            self.add_stack_to_slot(slot_index as usize, stack).await
        }
    }

    /// Adds a stack to a specific slot.
    ///
    /// Returns the number of items that couldn't fit.
    async fn add_stack_to_slot(&self, slot: usize, stack: ItemStack) -> usize {
        let mut stack_count = stack.item_count;
        let binding = self.get_stack(slot).await;
        let mut self_stack = binding.lock().await;

        if self_stack.is_empty() {
            *self_stack = stack.copy_with_count(0);
            //self.set_stack(slot, self_stack).await;
        }

        let count_left = self_stack.get_max_stack_size() - self_stack.item_count;
        let count_min = stack_count.min(count_left);

        if count_min != 0 {
            stack_count -= count_min;
            self_stack.increment(count_min);
        }
        stack_count as usize
    }

    /// Finds an empty slot in the inventory.
    ///
    /// # Returns
    /// The slot index or -1 if inventory is full.
    async fn get_empty_slot(&self) -> i16 {
        for i in 0..Self::MAIN_SIZE {
            if self.main_inventory[i].lock().await.is_empty() {
                return i as i16;
            }
        }

        -1
    }

    /// Checks if a stack can be added to an existing stack.
    fn can_stack_add_more(existing_stack: &ItemStack, stack: &ItemStack) -> bool {
        !existing_stack.is_empty()
            && existing_stack.are_items_and_components_equal(stack)
            && existing_stack.is_stackable()
            && existing_stack.item_count < existing_stack.get_max_stack_size()
    }

    /// Finds a slot with the same item type that has room for more items.
    ///
    /// Checks selected slot, off-hand, then other slots.
    async fn get_occupied_slot_with_room_for_stack(&self, stack: &ItemStack) -> i16 {
        if Self::can_stack_add_more(
            &*self
                .get_stack(self.get_selected_slot() as usize)
                .await
                .lock()
                .await,
            stack,
        ) {
            i16::from(self.get_selected_slot())
        } else if Self::can_stack_add_more(
            &*self.get_stack(Self::OFF_HAND_SLOT).await.lock().await,
            stack,
        ) {
            Self::OFF_HAND_SLOT as i16
        } else {
            for i in 0..Self::MAIN_SIZE {
                if Self::can_stack_add_more(&*self.main_inventory[i].lock().await, stack) {
                    return i as i16;
                }
            }

            -1
        }
    }

    /// Inserts a stack into any available slot.
    ///
    /// # Arguments
    /// - `stack` - The stack to insert (modified in place)
    ///
    /// # Returns
    /// `true` if any items were inserted, `false` otherwise.
    pub async fn insert_stack_anywhere(&self, stack: &mut ItemStack) -> bool {
        self.insert_stack(-1, stack).await
    }

    /// Inserts a stack into a specific slot or any slot.
    ///
    /// # Arguments
    /// - `slot` - The slot index, or -1 for any slot
    /// - `stack` - The stack to insert (modified in place)
    ///
    /// # Returns
    /// `true` if any items were inserted, `false` otherwise.
    pub async fn insert_stack(&self, slot: i16, stack: &mut ItemStack) -> bool {
        if stack.is_empty() {
            return false;
        }

        // TODO: if (stack.isDamaged()) {

        let mut i;

        loop {
            i = stack.item_count;
            if slot == -1 {
                stack.set_count(self.add_stack(stack.clone()).await as u8);
            } else {
                stack.set_count(self.add_stack_to_slot(slot as usize, stack.clone()).await as u8);
            }

            if stack.is_empty() || stack.item_count >= i {
                break;
            }
        }

        // TODO: Creative mode check

        stack.item_count < i
    }

    /// Finds the first slot containing a matching stack.
    ///
    /// # Returns
    /// The slot index or -1 if not found.
    pub async fn get_slot_with_stack(&self, stack: &ItemStack) -> i16 {
        for i in 0..Self::MAIN_SIZE {
            if !self.main_inventory[i].lock().await.is_empty()
                && self.main_inventory[i]
                    .lock()
                    .await
                    .are_items_and_components_equal(stack)
            {
                return i as i16;
            }
        }

        -1
    }

    /// Finds an empty hotbar slot to swap an item to.
    ///
    /// First looks for empty slots, then slots without enchantments.
    async fn get_swappable_hotbar_slot(&self) -> usize {
        let selected_slot = self.get_selected_slot() as usize;
        for i in 0..Self::HOTBAR_SIZE {
            let check_index = (i + selected_slot) % 9;
            if self.main_inventory[check_index].lock().await.is_empty() {
                return check_index;
            }
        }

        if let Some(i) = (0..Self::HOTBAR_SIZE).next() {
            let check_index = (i + selected_slot) % 9;
            return check_index;
        }

        self.get_selected_slot() as usize
    }

    /// Swaps an item stack with an item on the hotbar.
    ///
    /// Finds an empty hotbar slot and places the stack there.
    pub async fn swap_stack_with_hotbar(&self, stack: ItemStack) {
        self.set_selected_slot(self.get_swappable_hotbar_slot().await as u8);

        if !self.main_inventory[self.get_selected_slot() as usize]
            .lock()
            .await
            .is_empty()
        {
            let empty_slot = self.get_empty_slot().await;
            if empty_slot != -1 {
                self.set_stack(
                    empty_slot as usize,
                    self.main_inventory[self.get_selected_slot() as usize]
                        .lock()
                        .await
                        .clone(),
                )
                .await;
            }
        }

        self.set_stack(self.get_selected_slot() as usize, stack)
            .await;
    }

    /// Swaps the items at two slot indices.
    pub async fn swap_slot_with_hotbar(&self, slot: usize) {
        self.set_selected_slot(self.get_swappable_hotbar_slot().await as u8);
        let stack = self.main_inventory[self.get_selected_slot() as usize]
            .lock()
            .await
            .clone();
        self.set_stack(
            self.get_selected_slot() as usize,
            self.main_inventory[slot].lock().await.clone(),
        )
        .await;
        self.set_stack(slot, stack).await;
    }

    /// Gives a stack to the player or drops it if inventory is full.
    pub async fn offer_or_drop_stack(&self, stack: ItemStack, player: &dyn InventoryPlayer) {
        self.offer(stack, true, player).await;
    }

    /// Gives a stack to the player, optionally notifying the client.
    ///
    /// # Arguments
    /// - `stack` - The stack to give
    /// - `notify_client` - Whether to send inventory update packets
    /// - `player` - The player to give the stack to
    pub async fn offer(&self, stack: ItemStack, notify_client: bool, player: &dyn InventoryPlayer) {
        let mut stack = stack;
        while !stack.is_empty() {
            let mut room_for_stack = self.get_occupied_slot_with_room_for_stack(&stack).await;
            if room_for_stack == -1 {
                room_for_stack = self.get_empty_slot().await;
            }

            if room_for_stack == -1 {
                player.drop_item(stack, false).await;
                break;
            }

            let items_fit = stack.get_max_stack_size()
                - self
                    .get_stack(room_for_stack as usize)
                    .await
                    .lock()
                    .await
                    .item_count;
            if self
                .insert_stack(room_for_stack, &mut stack.split(items_fit))
                .await
                && notify_client
            {
                player
                    .enqueue_slot_set_packet(&CSetPlayerInventory::new(
                        i32::from(room_for_stack).into(),
                        &self
                            .get_stack(room_for_stack as usize)
                            .await
                            .lock()
                            .await
                            .clone()
                            .into(),
                    ))
                    .await;
            }
        }
    }
}

impl Clearable for PlayerInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for item in &self.main_inventory {
                *item.lock().await = ItemStack::EMPTY.clone();
            }

            self.entity_equipment.lock().await.clear();
        })
    }
}

impl Inventory for PlayerInventory {
    fn size(&self) -> usize {
        self.main_inventory.len() + self.equipment_slots.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for item in &self.main_inventory {
                if !item.lock().await.is_empty() {
                    return false;
                }
            }

            for slot in self.equipment_slots.values() {
                if !self
                    .entity_equipment
                    .lock()
                    .await
                    .get(slot)
                    .lock()
                    .await
                    .is_empty()
                {
                    return false;
                }
            }

            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move {
            if slot < self.main_inventory.len() {
                self.main_inventory[slot].clone()
            } else {
                let slot = self.equipment_slots.get(&slot).unwrap();
                self.entity_equipment.lock().await.get(slot)
            }
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot < self.main_inventory.len() {
                let mut removed = ItemStack::EMPTY.clone();
                let mut guard = self.main_inventory[slot].lock().await;
                std::mem::swap(&mut removed, &mut *guard);
                removed
            } else {
                let slot = self.equipment_slots.get(&slot).unwrap();
                self.entity_equipment
                    .lock()
                    .await
                    .put(slot, ItemStack::EMPTY.clone())
                    .await
            }
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            if slot < self.main_inventory.len() {
                split_stack(&self.main_inventory, slot, amount).await
            } else {
                let slot = self.equipment_slots.get(&slot).unwrap();

                let equipment = self.entity_equipment.lock().await.get(slot);
                let mut stack = equipment.lock().await;

                if !stack.is_empty() {
                    return stack.split(amount);
                }

                ItemStack::EMPTY.clone()
            }
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            if slot < self.main_inventory.len() {
                *self.main_inventory[slot].lock().await = stack;
            } else if let Some(slot) = self.equipment_slots.get(&slot) {
                self.entity_equipment.lock().await.put(slot, stack).await;
            } else {
                warn!("Failed to get Equipment Slot at {slot}");
            }
        })
    }

    fn mark_dirty(&self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PlayerInventory {
    /// Sets the selected hotbar slot.
    ///
    /// # Panics
    /// Panics if the slot index is not a valid hotbar index.
    pub fn set_selected_slot(&self, slot: u8) {
        if Self::is_valid_hotbar_index(slot as usize) {
            self.selected_slot.store(slot, Ordering::Relaxed);
        } else {
            panic!("Invalid hotbar slot: {slot}");
        }
    }

    /// Gets the currently selected hotbar slot index.
    pub fn get_selected_slot(&self) -> u8 {
        self.selected_slot.load(Ordering::Relaxed)
    }
}
