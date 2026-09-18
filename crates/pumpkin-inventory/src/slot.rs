//! Inventory slot implementations.
//!
//! This module defines the [`Slot`] trait and its implementations. Slots represent
//! individual positions in an inventory that can hold items.
//!
//! # Slot Types
//!
//! - [`NormalSlot`] - A basic inventory slot with no restrictions
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

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use crate::screen_handler::InventoryPlayer;

use crate::inventory::Inventory;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Enchantment, data_component_impl::EquipmentSlot};

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
    fn on_quick_move_crafted(&self, _stack: ItemStack, _stack_prev: ItemStack) {}

    /// Callback for when an item is taken from this slot.
    ///
    /// Also see: [`safe_take`]
    fn on_take_item(&self, _player: &dyn InventoryPlayer, _stack: &ItemStack) {
        self.mark_dirty();
    }

    /// Plugin callback for slot clicks.
    ///
    /// Called when a player clicks on this slot. Can be used by
    /// plugins to intercept or modify click behavior.
    fn on_click(&self, _player: &dyn InventoryPlayer) {}

    /// Checks if the given stack can be inserted into this slot.
    fn can_insert(&self, _stack: &ItemStack) -> bool {
        true
    }

    /// Gets the stack in this slot.
    fn get_stack(&self) -> ItemStack {
        self.get_inventory().get_stack(self.get_index())
    }

    /// Gets a copy of the stack in this slot.
    fn get_cloned_stack(&self) -> ItemStack {
        self.get_stack()
    }

    /// Checks if this slot has a non-empty stack.
    fn has_stack(&self) -> bool {
        !self.get_stack().is_empty()
    }

    /// Sets the stack in this slot.
    ///
    /// # Note
    /// Make sure to drop any locks to the slot stack before calling this.
    fn set_stack(&self, stack: ItemStack) {
        self.set_stack_no_callbacks(stack);
    }

    /// Sets the stack with previous stack reference.
    ///
    /// Some slots (like armor) need to know the previous stack for callbacks.
    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) {
        self.set_stack_no_callbacks(stack);
    }

    /// Sets the stack without calling callbacks.
    fn set_stack_no_callbacks(&self, stack: ItemStack) {
        let inv = self.get_inventory();
        inv.set_stack(self.get_index(), stack);
        self.mark_dirty();
    }

    /// Marks this slot as changed.
    ///
    /// Must be implemented by concrete types.
    fn mark_dirty(&self);

    /// Gets the maximum item count for this slot.
    fn get_max_item_count(&self) -> u8 {
        self.get_inventory().get_max_count_per_stack()
    }

    /// Gets the maximum item count for the given stack in this slot.
    fn get_max_item_count_for_stack(&self, stack: &ItemStack) -> u8 {
        self.get_max_item_count().min(stack.get_max_stack_size())
    }

    /// Removes a specific amount of items from this slot.
    ///
    /// Mojang name: `remove`
    fn take_stack(&self, amount: u8) -> ItemStack {
        let inv = self.get_inventory();
        inv.remove_stack_specific(self.get_index(), amount)
    }

    /// Checks if the player can take items from this slot.
    ///
    /// Mojang name: `mayPickup`
    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        true
    }

    /// Checks if this slot can be modified by the player.
    ///
    /// Mojang name: `allowModification`
    fn allow_modification(&self, player: &dyn InventoryPlayer) -> bool {
        self.can_insert(&self.get_cloned_stack()) && self.can_take_items(player)
    }

    /// Tries to take a stack in the given range.
    ///
    /// Returns `None` if can't take items or if slot is empty.
    /// For result slots, cannot take partial stacks.
    ///
    /// Mojang name: `tryRemove`
    fn try_take_stack_range(
        &self,
        min: u8,
        max: u8,
        player: &dyn InventoryPlayer,
    ) -> Option<ItemStack> {
        if !self.can_take_items(player) {
            return None;
        }
        if !self.allow_modification(player) && self.get_cloned_stack().item_count > max {
            // If the slot is not allowed to be modified, we cannot take a partial stack from it.
            return None;
        }
        let min = min.min(max);
        let stack = self.take_stack(min);

        if stack.is_empty() {
            None
        } else {
            if self.get_cloned_stack().is_empty() {
                self.set_stack_prev(ItemStack::EMPTY.clone(), stack.clone());
            }

            Some(stack)
        }
    }

    /// Safely tries to take a stack of items from the slot.
    ///
    /// Returns an empty stack if can't take. Triggers callbacks.
    ///
    /// Mojang name: `safeTake`
    fn safe_take(&self, min: u8, max: u8, player: &dyn InventoryPlayer) -> ItemStack {
        let stack = self.try_take_stack_range(min, max, player);

        if let Some(stack) = &stack {
            self.on_take_item(player, stack);
        }

        stack.unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    /// Inserts a stack into this slot.
    ///
    /// Returns any leftover items that couldn't fit.
    fn insert_stack(&self, stack: ItemStack) -> ItemStack {
        let stack_item_count = stack.item_count;
        self.insert_stack_count(stack, stack_item_count)
    }

    /// Inserts a specific count from a stack.
    ///
    /// Returns any leftover items.
    fn insert_stack_count(&self, mut stack: ItemStack, count: u8) -> ItemStack {
        if !stack.is_empty() && self.can_insert(&stack) {
            let mut stack_self = self.get_stack();
            let min_count = count
                .min(stack.item_count)
                .min(self.get_max_item_count_for_stack(&stack) - stack_self.item_count);

            if min_count != 0 {
                if stack_self.is_empty() {
                    self.set_stack(stack.split(min_count));
                } else if stack.are_items_and_components_equal(&stack_self) {
                    stack.decrement(min_count);
                    stack_self.increment(min_count);
                    self.set_stack(stack_self);
                }
            }
        }
        if stack.is_empty() {
            ItemStack::EMPTY.clone()
        } else {
            stack
        }
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

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
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

fn can_take_armor_stack(stack: &ItemStack, is_creative: bool) -> bool {
    is_creative || stack.get_enchantment_level(&Enchantment::BINDING_CURSE) <= 0
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
    fn can_insert(&self, stack: &ItemStack) -> bool {
        match self.equipment_slot {
            EquipmentSlot::Head(_) => {
                stack.is_helmet() || stack.is_skull() || stack.item == &Item::CARVED_PUMPKIN
            }
            EquipmentSlot::Chest(_) => stack.is_chestplate() || stack.item == &Item::ELYTRA,
            EquipmentSlot::Legs(_) => stack.is_leggings(),
            EquipmentSlot::Feet(_) => stack.is_boots(),
            EquipmentSlot::Saddle(_) => stack
                .get_data_component::<pumpkin_data::data_component_impl::EquippableImpl>()
                .map_or_else(
                    || stack.item == &Item::SADDLE,
                    |equippable| matches!(equippable.slot, EquipmentSlot::Saddle(_)),
                ),
            EquipmentSlot::Body(_) => stack
                .get_data_component::<pumpkin_data::data_component_impl::EquippableImpl>()
                .map_or_else(
                    || {
                        stack.item.registry_key.ends_with("_horse_armor")
                            || stack.item.registry_key.ends_with("_nautilus_armor")
                            || stack.item == &Item::WOLF_ARMOR
                    },
                    |equippable| matches!(equippable.slot, EquipmentSlot::Body(_)),
                ),
            _ => true,
        }
    }

    fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) {
        self.set_stack_no_callbacks(stack);
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }

    /// Armor slots can only hold one item.
    fn get_max_item_count(&self) -> u8 {
        1
    }

    fn can_take_items(&self, player: &dyn InventoryPlayer) -> bool {
        can_take_armor_stack(&self.get_stack(), player.is_creative())
    }
}

#[cfg(test)]
mod tests {
    use super::can_take_armor_stack;
    use crate::build_equipment_slots;
    use crate::entity_equipment::EntityEquipment;
    use crate::player::player_inventory::PlayerInventory;
    use crate::player::player_screen_handler::PlayerScreenHandler;
    use crate::screen_handler::{InventoryPlayer, ScreenHandler};
    use pumpkin_data::Enchantment;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::screen::WindowType;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory, CSetSelectedSlot,
    };
    use pumpkin_protocol::java::server::play::SlotActionType;
    use std::sync::{Arc, Mutex};

    struct DummyPlayer {
        creative: bool,
        inventory: Arc<PlayerInventory>,
    }

    impl DummyPlayer {
        fn new(creative: bool) -> Self {
            Self {
                creative,
                inventory: Arc::new(PlayerInventory::new(
                    Arc::new(Mutex::new(EntityEquipment::new())),
                    Arc::new(build_equipment_slots()),
                )),
            }
        }
    }

    impl InventoryPlayer for DummyPlayer {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) {}

        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }

        fn has_infinite_materials(&self) -> bool {
            self.creative
        }

        fn is_creative(&self) -> bool {
            self.creative
        }

        fn experience_level(&self) -> i32 {
            0
        }

        fn add_experience_levels(&self, _levels: i32) {}

        fn enchantment_seed(&self) -> i32 {
            0
        }

        fn set_enchantment_seed(&self, _seed: i32) {}

        fn enqueue_inventory_packet(
            &self,
            _packet: &CSetContainerContent,
            _window_type: Option<WindowType>,
        ) {
        }

        fn enqueue_slot_packet(
            &self,
            _packet: &CSetContainerSlot,
            _window_type: Option<WindowType>,
            _total_slots: usize,
        ) {
        }

        fn enqueue_cursor_packet(&self, _packet: &CSetCursorItem) {}

        fn enqueue_property_packet(&self, _packet: &CSetContainerProperty) {}

        fn enqueue_slot_set_packet(&self, _packet: &CSetPlayerInventory) {}

        fn enqueue_set_held_item_packet(&self, _packet: &CSetSelectedSlot) {}

        fn enqueue_equipment_change(
            &self,
            _slot: &pumpkin_data::data_component_impl::EquipmentSlot,
            _stack: &ItemStack,
        ) {
        }

        fn award_experience(&self, _amount: i32) {}

        fn increment_stat(
            &self,
            _category: pumpkin_data::statistic::StatisticCategory,
            _stat_id: i32,
            _amount: i32,
        ) {
        }

        fn play_block_sound(&self, _sound: pumpkin_data::sound::Sound, _pitch: f32) {}
    }

    fn cursed_helmet() -> ItemStack {
        let mut stack = ItemStack::new(1, &Item::DIAMOND_HELMET);
        stack.add_enchantment(&Enchantment::BINDING_CURSE, 1);
        stack
    }

    #[test]
    fn binding_curse_blocks_non_creative_removal() {
        let stack = cursed_helmet();

        assert!(!can_take_armor_stack(&stack, false));
        assert!(can_take_armor_stack(&stack, true));
    }

    #[test]
    fn unbound_armor_remains_removable() {
        let stack = ItemStack::new(1, &Item::DIAMOND_HELMET);

        assert!(can_take_armor_stack(&stack, false));
    }

    #[test]
    fn binding_curse_blocks_pickup_and_quick_move() {
        let player = DummyPlayer::new(false);
        let stack = cursed_helmet();
        player.inventory.set_slot(39, stack.clone());
        let mut handler = PlayerScreenHandler::new(&player.inventory, None, 0, None);

        handler.on_slot_click(5, 0, SlotActionType::Pickup, &player);

        assert!(
            player
                .inventory
                .get_slot(39)
                .are_items_and_components_equal(&stack)
        );
        assert!(
            handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );

        handler.on_slot_click(5, 0, SlotActionType::QuickMove, &player);

        assert!(
            player
                .inventory
                .get_slot(39)
                .are_items_and_components_equal(&stack)
        );
    }

    #[test]
    fn creative_can_pick_up_binding_curse_armor() {
        let player = DummyPlayer::new(true);
        let stack = cursed_helmet();
        player.inventory.set_slot(39, stack.clone());
        let mut handler = PlayerScreenHandler::new(&player.inventory, None, 0, None);

        handler.on_slot_click(5, 0, SlotActionType::Pickup, &player);

        assert!(player.inventory.get_slot(39).is_empty());
        assert!(
            handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .are_items_and_components_equal(&stack)
        );
    }

    #[test]
    fn unbound_armor_can_be_quick_moved() {
        let player = DummyPlayer::new(false);
        player
            .inventory
            .set_slot(39, ItemStack::new(1, &Item::DIAMOND_HELMET));
        let mut handler = PlayerScreenHandler::new(&player.inventory, None, 0, None);

        handler.on_slot_click(5, 0, SlotActionType::QuickMove, &player);

        assert!(player.inventory.get_slot(39).is_empty());
        assert!(player.inventory.contains_item(&Item::DIAMOND_HELMET));
    }
}
