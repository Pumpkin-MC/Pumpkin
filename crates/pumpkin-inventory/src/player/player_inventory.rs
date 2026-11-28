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

use crate::inventory::{Clearable, Inventory};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_util::Hand;
use rustc_hash::FxHashMap;
use std::any::Any;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use tracing::warn;

/// The player's inventory.
///
/// Contains 36 main inventory slots (hotbar + main storage) plus
/// equipment slots accessed through [`EntityEquipment`].
pub struct PlayerInventory {
    /// The 36 main inventory slots (slots 0-35).
    ///
    /// The first 9 slots (0-8) are the hotbar, the remaining 27 (9-35) are the main storage.
    pub main_inventory: RwLock<[ItemStack; Self::MAIN_SIZE]>,
    /// Mapping of slot indices to equipment slot types.
    ///
    /// Used to identify which slots correspond to armor and off-hand equipment.
    pub equipment_slots: Arc<FxHashMap<usize, EquipmentSlot>>,
    /// The currently selected hotbar slot index (0-8).
    pub selected_slot: AtomicU8,
    /// The entity equipment storage for armor and off-hand items.
    ///
    /// This is separate from the main inventory and is rendered on the player model.
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
}

pub struct PlayerInventoryDelivery<'a> {
    _main: Option<std::sync::RwLockReadGuard<'a, [ItemStack; PlayerInventory::MAIN_SIZE]>>,
    _equipment: Option<std::sync::MutexGuard<'a, EntityEquipment>>,
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
        equipment_slots: Arc<FxHashMap<usize, EquipmentSlot>>,
    ) -> Self {
        Self {
            main_inventory: RwLock::new(std::array::from_fn(|_| ItemStack::EMPTY.clone())),
            equipment_slots,
            selected_slot: AtomicU8::new(0),
            entity_equipment,
        }
    }

    /// Fast non-blocking count of an item across all main inventory slots.
    pub fn count_item(&self, item: &'static Item) -> u32 {
        let mut total = 0u32;
        if let Ok(inv) = self.main_inventory.try_read() {
            for stack in inv.iter() {
                if stack.get_item().id == item.id {
                    total += u32::from(stack.item_count);
                }
            }
        }
        total
    }

    /// Fast non-blocking check if the main inventory contains a given item.
    pub fn contains_item(&self, item: &'static Item) -> bool {
        if let Ok(inv) = self.main_inventory.try_read() {
            for stack in inv.iter() {
                if !stack.is_empty() && stack.get_item().id == item.id {
                    return true;
                }
            }
        }
        false
    }

    /// Gets the item in the currently selected hotbar slot.
    ///
    /// This is the item the player is currently holding in their main hand.
    pub fn held_item(&self) -> ItemStack {
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inv[self.get_selected_slot() as usize].clone()
    }

    /// Sets the item in the currently selected hotbar slot.
    pub fn set_held_item(&self, stack: ItemStack) {
        let selected = self.get_selected_slot() as usize;
        let mut inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inv[selected] = stack;
    }

    /// Sets the item in the specified hand.
    pub fn set_stack_in_hand(&self, hand: Hand, stack: ItemStack) {
        match hand {
            Hand::Right => self.set_held_item(stack),
            Hand::Left => {
                let Some(slot) = self.equipment_slots.get(&Self::OFF_HAND_SLOT) else {
                    return;
                };
                self.entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .put(slot, stack);
            }
        }
    }

    /// Gets the item in the specified hand.
    ///
    /// # Arguments
    /// - `hand` - Which hand to get the item from
    pub fn get_stack_in_hand(&self, hand: Hand) -> ItemStack {
        match hand {
            Hand::Left => self.off_hand_item(),
            Hand::Right => self.held_item(),
        }
    }

    /// Gets the item in the off-hand.
    ///
    /// Mojang name: `getOffHandStack`
    pub fn off_hand_item(&self) -> ItemStack {
        let Some(slot) = self.equipment_slots.get(&Self::OFF_HAND_SLOT) else {
            return ItemStack::EMPTY.clone();
        };
        self.entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
    }

    /// Swaps the items between main hand and off-hand.
    ///
    /// # Returns
    /// The new main hand item and new off-hand item.
    pub fn swap_item(&self) -> (ItemStack, ItemStack) {
        let Some(slot) = self.equipment_slots.get(&Self::OFF_HAND_SLOT) else {
            return (ItemStack::EMPTY.clone(), ItemStack::EMPTY.clone());
        };
        let mut main_inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut equipment = self
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let selected = self.get_selected_slot() as usize;
        let main_hand_item = main_inv[selected].clone();
        let new_main = equipment.put(slot, main_hand_item.clone());
        main_inv[selected] = new_main.clone();
        (new_main, main_hand_item)
    }

    /// Swaps an item under the same lock order as hand swapping.
    pub fn swap_equippable(
        &self,
        hand: Hand,
        destination: &EquipmentSlot,
        expected: &ItemStack,
        creative: bool,
    ) -> Option<Vec<ItemStack>> {
        use pumpkin_data::data_component_impl::UseRemainderImpl;

        let source = match hand {
            Hand::Right => EquipmentSlot::MAIN_HAND,
            Hand::Left => EquipmentSlot::OFF_HAND,
        };
        if source == *destination || self.equipment_slot_index(destination).is_none() {
            return None;
        }
        let mut main = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut equipment = self
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let selected = self.get_selected_slot() as usize;
        let held = if source == EquipmentSlot::MAIN_HAND {
            main[selected].clone()
        } else {
            equipment.get(&source)
        };
        if held.uid != expected.uid || !held.are_equal(expected) {
            return None;
        }
        let current = if *destination == EquipmentSlot::MAIN_HAND {
            main[selected].clone()
        } else {
            equipment.get(destination)
        };
        if (!creative && current.has_prevent_armor_change())
            || current.are_items_and_components_equal(&held)
        {
            return None;
        }
        let mut extras = Vec::new();
        let mut next_hand = if held.item_count > 1 {
            let mut remainder = held.clone();
            if !creative {
                remainder.decrement(1);
            }
            if !current.is_empty() {
                extras.push(current);
            }
            remainder
        } else if creative && current.is_empty() {
            held.clone()
        } else {
            current
        };
        if held.get_base_use_time() == 0
            && let Some(component) = held.get_data_component::<UseRemainderImpl>()
        {
            next_hand =
                component.convert_into_remainder(next_hand, held.item_count, creative, |stack| {
                    extras.push(stack);
                });
        }
        let equipped = held.copy_with_count(1);
        if *destination == EquipmentSlot::MAIN_HAND {
            main[selected] = equipped;
        } else {
            equipment.put(destination, equipped);
        }
        if source == EquipmentSlot::MAIN_HAND {
            main[selected] = next_hand;
        } else {
            equipment.put(&source, next_hand);
        }
        Some(extras)
    }

    /// Checks if a slot index is a valid hotbar slot.
    #[must_use]
    pub const fn is_valid_hotbar_index(slot: usize) -> bool {
        slot < Self::HOTBAR_SIZE
    }

    fn insert_into_available_slot(&self, stack: &mut ItemStack) -> Option<usize> {
        // Select and write under the same guards. Another writer cannot take
        // the chosen slot between the search and the insertion.
        let mut main = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut equipment = self
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let selected = usize::from(self.get_selected_slot());
        let off_hand_slot = self.equipment_slots.get(&Self::OFF_HAND_SLOT);
        let off_hand = off_hand_slot.map(|slot| equipment.get(slot));
        let can_merge = |current: &ItemStack| Self::can_stack_add_more(current, stack);
        let slot = if can_merge(&main[selected]) {
            selected
        } else if let Some((slot, mut current)) = off_hand_slot.zip(off_hand)
            && can_merge(&current)
        {
            return Self::insert_into_stack(&mut current, stack).then(|| {
                equipment.put(slot, current);
                Self::OFF_HAND_SLOT
            });
        } else {
            main.iter()
                .position(can_merge)
                .or_else(|| main.iter().position(ItemStack::is_empty))?
        };
        Self::insert_into_stack(&mut main[slot], stack).then_some(slot)
    }

    fn insert_into_stack(current: &mut ItemStack, stack: &mut ItemStack) -> bool {
        let room = if current.is_empty() {
            stack.get_max_stack_size()
        } else if current.are_items_and_components_equal(stack) {
            current
                .get_max_stack_size()
                .saturating_sub(current.item_count)
        } else {
            return false;
        };
        let count = stack.item_count.min(room);
        if count == 0 {
            return false;
        }
        if current.is_empty() {
            *current = stack.copy_with_count(count);
        } else {
            current.increment(count);
        }
        stack.decrement(count);
        true
    }

    /// Adds a stack to a specific slot.
    ///
    /// Returns the number of items that couldn't fit.
    fn add_stack_to_slot(&self, slot: usize, stack: ItemStack) -> usize {
        if slot >= Self::MAIN_SIZE {
            if let Some(slot_type) = self.equipment_slots.get(&slot) {
                let mut equipment = self
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut current = equipment.get(slot_type);
                if current.is_empty() {
                    let max_stack = stack.get_max_stack_size();
                    if stack.item_count <= max_stack {
                        equipment.put(slot_type, stack);
                        return 0;
                    }

                    equipment.put(slot_type, stack.copy_with_count(max_stack));

                    return (stack.item_count - max_stack) as usize;
                } else if matches!(slot_type, EquipmentSlot::OffHand(_))
                    && current.are_items_and_components_equal(&stack)
                {
                    let count_left = current
                        .get_max_stack_size()
                        .saturating_sub(current.item_count);
                    let count_min = stack.item_count.min(count_left);
                    if count_min != 0 {
                        current.increment(count_min);
                        equipment.put(slot_type, current);
                    }
                    return (stack.item_count - count_min) as usize;
                }
            }
            return stack.item_count as usize;
        }

        let mut inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut stack = stack;
        Self::insert_into_stack(&mut inv[slot], &mut stack);
        usize::from(stack.item_count)
    }

    /// Finds an empty slot in the inventory.
    ///
    /// # Returns
    /// The slot index or -1 if inventory is full.
    fn get_empty_slot(&self) -> i16 {
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (i, stack) in inv.iter().enumerate() {
            if stack.is_empty() {
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
    fn get_occupied_slot_with_room_for_stack(&self, stack: &ItemStack) -> i16 {
        let selected = self.get_selected_slot() as usize;
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if Self::can_stack_add_more(&inv[selected], stack) {
            return selected as i16;
        }

        let off_hand = self.off_hand_item();
        if Self::can_stack_add_more(&off_hand, stack) {
            return Self::OFF_HAND_SLOT as i16;
        }

        for (i, item) in inv.iter().enumerate() {
            if Self::can_stack_add_more(item, stack) {
                return i as i16;
            }
        }

        -1
    }

    /// Whether `insert_stack_anywhere` would place at least one item of `stack`.
    pub fn can_hold_any(&self, stack: &ItemStack) -> bool {
        self.get_occupied_slot_with_room_for_stack(stack) != -1 || self.get_empty_slot() != -1
    }

    /// Inserts a stack into any available slot.
    ///
    /// # Arguments
    /// - `stack` - The stack to insert (modified in place)
    ///
    /// # Returns
    /// `true` if any items were inserted, `false` otherwise.
    pub fn insert_stack_anywhere(&self, stack: &mut ItemStack) -> bool {
        self.insert_stack(-1, stack)
    }

    /// Inserts a stack into a specific slot or any slot.
    ///
    /// # Arguments
    /// - `slot` - The slot index, or -1 for any slot
    /// - `stack` - The stack to insert (modified in place)
    ///
    /// # Returns
    /// `true` if any items were inserted, `false` otherwise.
    pub fn insert_stack(&self, slot: i16, stack: &mut ItemStack) -> bool {
        if stack.is_empty() {
            return false;
        }

        let mut i;
        let start_count = stack.item_count;

        loop {
            i = stack.item_count;
            if slot == -1 {
                if self.insert_into_available_slot(stack).is_none() {
                    break;
                }
            } else {
                stack.set_count(self.add_stack_to_slot(slot as usize, stack.clone()) as u8);
            }

            if stack.is_empty() || stack.item_count >= i {
                break;
            }
        }

        stack.item_count < start_count
    }

    /// Finds the first slot containing a matching stack.
    ///
    /// # Returns
    /// The slot index or -1 if not found.
    pub fn get_slot_with_stack(&self, stack: &ItemStack) -> i16 {
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (i, item) in inv.iter().enumerate() {
            if !item.is_empty() && item.are_items_and_components_equal(stack) {
                return i as i16;
            }
        }
        -1
    }

    /// Finds an empty hotbar slot to swap an item to.
    ///
    /// First looks for empty slots, then slots without enchantments.
    fn get_swappable_hotbar_slot(&self) -> usize {
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let selected_slot = self.get_selected_slot() as usize;
        for i in 0..Self::HOTBAR_SIZE {
            let check_index = (i + selected_slot) % 9;
            if inv[check_index].is_empty() {
                return check_index;
            }
        }

        selected_slot
    }

    /// Swaps an item stack with an item on the hotbar.
    ///
    /// Finds an empty hotbar slot and places the stack there.
    pub fn swap_stack_with_hotbar(&self, stack: ItemStack) {
        let swappable = self.get_swappable_hotbar_slot();
        self.set_selected_slot(swappable as u8);
        let selected = self.get_selected_slot() as usize;
        let mut inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(empty_slot) = inv.iter().position(ItemStack::is_empty)
            && !inv[selected].is_empty()
        {
            inv[empty_slot] = inv[selected].clone();
        }

        inv[selected] = stack;
    }

    /// Swaps the items at two slot indices.
    pub fn swap_slot_with_hotbar(&self, slot: usize) {
        let swappable = self.get_swappable_hotbar_slot();
        self.set_selected_slot(swappable as u8);
        let selected = self.get_selected_slot() as usize;
        let mut inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inv.swap(selected, slot);
    }

    /// Gets the item in the specified slot (synchronously).
    pub fn get_slot(&self, slot: usize) -> ItemStack {
        if slot < Self::MAIN_SIZE {
            let inv = self
                .main_inventory
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv[slot].clone()
        } else if let Some(slot_type) = self.equipment_slots.get(&slot) {
            self.entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(slot_type)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    /// Holds the physical inventory stable when the represented wire values still match.
    #[must_use]
    pub fn lock_matching_slots<'stack>(
        &self,
        slots: impl IntoIterator<Item = (usize, &'stack ItemStack)>,
    ) -> Option<PlayerInventoryDelivery<'_>> {
        let mut slots = slots.into_iter().peekable();
        if slots.peek().is_none() {
            return Some(PlayerInventoryDelivery {
                _main: None,
                _equipment: None,
            });
        }
        let main = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let equipment = self
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slots
            .all(|(index, expected)| {
                let Some(current) = main.get(index).or_else(|| {
                    let key = self.equipment_slots.get(&index)?;
                    Some(equipment.equipment.get(key).unwrap_or(ItemStack::EMPTY))
                }) else {
                    return false;
                };
                current.are_equal(expected) && (expected.is_empty() || current.uid == expected.uid)
            })
            .then_some(PlayerInventoryDelivery {
                _main: Some(main),
                _equipment: Some(equipment),
            })
    }

    /// Sets the item in the specified slot (synchronously).
    pub fn set_slot(&self, slot: usize, stack: ItemStack) {
        if slot < Self::MAIN_SIZE {
            let mut inv = self
                .main_inventory
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv[slot] = stack;
        } else if let Some(slot_type) = self.equipment_slots.get(&slot) {
            self.entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(slot_type, stack);
        }
    }

    /// Commits a change to a physical slot if it still holds the observed stack.
    pub fn replace_slot_if_same(
        &self,
        slot: usize,
        expected: &ItemStack,
        update: ItemStack,
    ) -> bool {
        let matches =
            |current: &ItemStack| current.uid == expected.uid && current.are_equal(expected);
        if slot < Self::MAIN_SIZE {
            let mut inv = self
                .main_inventory
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches(&inv[slot]) {
                inv[slot] = update;
                return true;
            }
        } else if let Some(slot) = self.equipment_slots.get(&slot) {
            let mut equipment = self
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if matches(equipment.equipment.get(slot).unwrap_or(ItemStack::EMPTY)) {
                equipment.put(slot, update);
                return true;
            }
        }
        false
    }

    /// Gives a stack to the player or drops it if inventory is full.
    pub fn offer_or_drop_stack(&self, stack: ItemStack, player: &dyn InventoryPlayer) {
        self.offer(stack, true, player);
    }

    /// Gives a stack to the player, optionally notifying the client.
    ///
    /// # Arguments
    /// - `stack` - The stack to give
    /// - `notify_client` - Whether to send inventory update packets
    /// - `player` - The player to give the stack to
    pub fn offer(&self, stack: ItemStack, notify_client: bool, player: &dyn InventoryPlayer) {
        let mut stack = stack;
        while !stack.is_empty() {
            let Some(slot) = self.insert_into_available_slot(&mut stack) else {
                player.drop_item(stack, false);
                break;
            };
            if notify_client {
                player.enqueue_slot_set_packet(&CSetPlayerInventory::new(
                    (slot as i32).into(),
                    &self.get_stack(slot).into(),
                ));
            }
        }
    }
}

impl Clearable for PlayerInventory {
    fn clear(&self) {
        let mut inv = self
            .main_inventory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inv.fill_with(|| ItemStack::EMPTY.clone());
        self.entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

impl Inventory for PlayerInventory {
    fn size(&self) -> usize {
        Self::MAIN_SIZE + self.equipment_slots.len()
    }

    fn is_empty(&self) -> bool {
        let inv = self
            .main_inventory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if inv.iter().any(|s| !s.is_empty()) {
            return false;
        }

        for slot in self.equipment_slots.values() {
            let eq_item = self
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(slot);
            if !eq_item.is_empty() {
                return false;
            }
        }

        true
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        if slot < Self::MAIN_SIZE {
            let inv = self
                .main_inventory
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv[slot].clone()
        } else if let Some(slot) = self.equipment_slots.get(&slot) {
            self.entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(slot)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot < Self::MAIN_SIZE {
            let mut inv = self
                .main_inventory
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::replace(&mut inv[slot], ItemStack::EMPTY.clone())
        } else if let Some(slot) = self.equipment_slots.get(&slot) {
            self.entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(slot, ItemStack::EMPTY.clone())
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot < Self::MAIN_SIZE {
            let mut inv = self
                .main_inventory
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !inv[slot].is_empty() && amount > 0 {
                inv[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            }
        } else if let Some(slot) = self.equipment_slots.get(&slot) {
            let mut equipment = self
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut stack = equipment.get(slot);

            if !stack.is_empty() && amount > 0 {
                let split = stack.split(amount);
                equipment.put(slot, stack);
                split
            } else {
                ItemStack::EMPTY.clone()
            }
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot < Self::MAIN_SIZE {
            let mut inv = self
                .main_inventory
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inv[slot] = stack;
        } else if let Some(slot) = self.equipment_slots.get(&slot) {
            self.entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(slot, stack);
        } else {
            warn!("Failed to get Equipment Slot at {slot}");
        }
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
        }
    }

    /// Gets the currently selected hotbar slot index.
    pub fn get_selected_slot(&self) -> u8 {
        self.selected_slot.load(Ordering::Relaxed)
    }

    /// Inventory slot of an equipment slot -> the main hand is the selected hotbar slot
    pub fn equipment_slot_index(&self, slot: &EquipmentSlot) -> Option<usize> {
        if matches!(slot, EquipmentSlot::MainHand(_)) {
            return Some(self.get_selected_slot() as usize);
        }
        self.equipment_slots
            .iter()
            .find_map(|(index, equipment)| (equipment == slot).then_some(*index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_equipment_slots;
    use pumpkin_data::item::Item;

    fn create_test_inventory() -> PlayerInventory {
        let equipment_slots = Arc::new(build_equipment_slots());
        let entity_equipment = Arc::new(Mutex::new(EntityEquipment::new()));
        PlayerInventory::new(entity_equipment, equipment_slots)
    }

    #[test]
    fn unmapped_equipment_swap_leaves_the_source_unchanged() {
        let inv = create_test_inventory();
        let held = ItemStack::new(1, &Item::STICK);
        inv.set_stack_in_hand(Hand::Right, held.clone());
        assert!(
            inv.swap_equippable(Hand::Right, &EquipmentSlot::BODY, &held, false)
                .is_none()
        );
        assert_eq!(inv.held_item().uid, held.uid);
        assert!(inv.held_item().are_equal(&held));
    }

    #[test]
    fn matching_empty_equipment_slot_can_be_filled() {
        let inv = create_test_inventory();
        let slot = PlayerInventory::OFF_HAND_SLOT;
        let empty = inv.get_slot(slot);
        let new_stack = ItemStack::new(1, &Item::STICK);
        assert!(inv.replace_slot_if_same(slot, &empty, new_stack.clone()));
        assert_eq!(inv.get_slot(slot).uid, new_stack.uid);
        assert!(!inv.replace_slot_if_same(slot, &empty, ItemStack::EMPTY.clone()));
    }

    #[test]
    fn add_stack_to_slot_offhand_when_empty() {
        let inv = create_test_inventory();
        let stack = ItemStack::new(10, &Item::ARROW);

        let leftover = inv.add_stack_to_slot(PlayerInventory::OFF_HAND_SLOT, stack);
        assert_eq!(leftover, 0);

        let off_hand = inv.off_hand_item();
        assert_eq!(off_hand.item.id, Item::ARROW.id);
        assert_eq!(off_hand.item_count, 10);
    }

    #[test]
    fn add_stack_to_slot_offhand_stacks_matching_item() {
        let inv = create_test_inventory();
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(10, &Item::ARROW));

        let leftover = inv.add_stack_to_slot(
            PlayerInventory::OFF_HAND_SLOT,
            ItemStack::new(15, &Item::ARROW),
        );
        assert_eq!(leftover, 0);

        let off_hand = inv.off_hand_item();
        assert_eq!(off_hand.item.id, Item::ARROW.id);
        assert_eq!(off_hand.item_count, 25);
    }

    #[test]
    fn add_stack_to_slot_offhand_stacks_up_to_max() {
        let inv = create_test_inventory();
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(60, &Item::ARROW));

        let leftover = inv.add_stack_to_slot(
            PlayerInventory::OFF_HAND_SLOT,
            ItemStack::new(10, &Item::ARROW),
        );
        assert_eq!(leftover, 6);

        let off_hand = inv.off_hand_item();
        assert_eq!(off_hand.item.id, Item::ARROW.id);
        assert_eq!(off_hand.item_count, 64);
    }

    #[test]
    fn add_stack_to_slot_offhand_rejects_different_item() {
        let inv = create_test_inventory();
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(1, &Item::SHIELD));

        let leftover = inv.add_stack_to_slot(
            PlayerInventory::OFF_HAND_SLOT,
            ItemStack::new(10, &Item::ARROW),
        );
        assert_eq!(leftover, 10);

        let off_hand = inv.off_hand_item();
        assert_eq!(off_hand.item.id, Item::SHIELD.id);
        assert_eq!(off_hand.item_count, 1);
    }

    #[test]
    fn add_stack_to_slot_armor_does_not_stack() {
        let inv = create_test_inventory();
        let feet_slot = EquipmentSlot::FEET
            .get_offset_entity_slot_id(PlayerInventory::MAIN_SIZE as i32)
            as usize;
        let boots = ItemStack::new(1, &Item::IRON_BOOTS);

        let leftover = inv.add_stack_to_slot(feet_slot, boots.clone());
        assert_eq!(leftover, 0);

        let leftover_second = inv.add_stack_to_slot(feet_slot, boots);
        assert_eq!(leftover_second, 1);
    }

    #[test]
    fn pickup_stacks_into_offhand() {
        let inv = create_test_inventory();
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(10, &Item::ARROW));

        let mut pickup = ItemStack::new(5, &Item::ARROW);
        let inserted = inv.insert_stack_anywhere(&mut pickup);

        assert!(inserted);
        assert!(pickup.is_empty());
        assert_eq!(inv.off_hand_item().item_count, 15);
    }

    #[test]
    fn pickup_prioritizes_held_main_hand_over_offhand() {
        let inv = create_test_inventory();
        inv.set_held_item(ItemStack::new(60, &Item::ARROW));
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(10, &Item::ARROW));

        let mut pickup = ItemStack::new(10, &Item::ARROW);
        let inserted = inv.insert_stack_anywhere(&mut pickup);

        assert!(inserted);
        assert!(pickup.is_empty());
        assert_eq!(inv.held_item().item_count, 64);
        assert_eq!(inv.off_hand_item().item_count, 16);
    }

    #[test]
    fn pickup_offhand_full_overflows_to_main_inventory() {
        let inv = create_test_inventory();
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(64, &Item::ARROW));

        let mut pickup = ItemStack::new(10, &Item::ARROW);
        let inserted = inv.insert_stack_anywhere(&mut pickup);

        assert!(inserted);
        assert!(pickup.is_empty());
        assert_eq!(inv.off_hand_item().item_count, 64);
        assert_eq!(inv.get_slot(0).item.id, Item::ARROW.id);
        assert_eq!(inv.get_slot(0).item_count, 10);
    }

    #[test]
    fn pickup_does_not_fill_empty_offhand() {
        let inv = create_test_inventory();

        let mut pickup = ItemStack::new(10, &Item::ARROW);
        let inserted = inv.insert_stack_anywhere(&mut pickup);

        assert!(inserted);
        assert!(pickup.is_empty());
        assert!(inv.off_hand_item().is_empty());
        assert_eq!(inv.get_slot(0).item.id, Item::ARROW.id);
        assert_eq!(inv.get_slot(0).item_count, 10);
    }

    /// When the main inventory is completely full and only the off-hand has
    /// partial room, `insert_stack` must still return `true` even though the
    /// second loop iteration makes no progress.
    #[test]
    fn insert_stack_returns_true_on_partial_offhand_with_full_main_inventory() {
        let inv = create_test_inventory();
        for i in 0..PlayerInventory::MAIN_SIZE {
            inv.set_slot(i, ItemStack::new(64, &Item::DIRT));
        }
        inv.set_stack_in_hand(Hand::Left, ItemStack::new(60, &Item::ARROW));

        let mut pickup = ItemStack::new(10, &Item::ARROW);
        let inserted = inv.insert_stack_anywhere(&mut pickup);

        assert!(
            inserted,
            "insert_stack must return true when some items were placed"
        );
        assert_eq!(
            pickup.item_count, 6,
            "6 arrows should remain after filling off-hand to 64"
        );
        assert_eq!(inv.off_hand_item().item_count, 64);
    }

    #[test]
    fn insert_stack_preserves_remainder_when_destination_changes() {
        let inv = create_test_inventory();
        let destination = inv.get_empty_slot();
        assert_eq!(destination, 0);
        inv.set_slot(0, ItemStack::new(1, &Item::DIRT));
        let mut remainder = ItemStack::new(1, &Item::STICK);

        assert!(!inv.insert_stack(destination, &mut remainder));
        assert_eq!(remainder.item, &Item::STICK);
        assert_eq!(remainder.item_count, 1);
        assert_eq!(inv.get_slot(0).item, &Item::DIRT);
        assert_eq!(inv.get_slot(0).item_count, 1);

        assert!(inv.insert_stack_anywhere(&mut remainder));
        assert!(remainder.is_empty());
        assert_eq!(inv.get_slot(1).item, &Item::STICK);
        assert_eq!(inv.get_slot(1).item_count, 1);
    }
}
