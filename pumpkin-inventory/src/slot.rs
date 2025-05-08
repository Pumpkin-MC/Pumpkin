#![warn(unused)]
use std::{
    fmt::Debug,
    sync::{Arc, atomic::AtomicU8},
    time::Duration,
};

use async_trait::async_trait;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use tokio::{sync::Mutex, time::timeout};

use crate::{equipment_slot::EquipmentSlot, screen_handler::InventoryPlayer};

// Slot.java
// This is a trait due to crafting slots being a thing
#[async_trait]
pub trait Slot: Send + Sync + Debug {
    fn get_inventory(&self) -> &Arc<dyn Inventory>;

    fn get_index(&self) -> usize;

    fn set_id(&self, index: usize);

    fn on_quick_transfer(&self, new_item: ItemStack, original: ItemStack) {
        let diff = new_item.item_count - original.item_count;
        if diff > 0 {
            self.on_crafted(original, diff);
        }
    }

    fn on_crafted(&self, _stack: ItemStack, _amount: u8) {}

    fn on_crafted_single(&self, _stack: ItemStack) {}

    fn on_take(&self, _amount: u8) {}

    async fn on_take_item(&self, _player: &dyn InventoryPlayer, _stack: &ItemStack) {
        self.mark_dirty().await;
    }

    // Used for plugins
    async fn on_click(&self, _player: &dyn InventoryPlayer) {}

    async fn can_insert(&self, _stack: &ItemStack) -> bool {
        true
    }

    async fn get_stack(&self) -> Arc<Mutex<ItemStack>> {
        self.get_inventory().get_stack(self.get_index()).await
    }

    async fn get_cloned_stack(&self) -> ItemStack {
        let stack = self.get_inventory().get_stack(self.get_index()).await;
        let lock = timeout(Duration::from_secs(5), stack.lock())
            .await
            .expect("Timed out while trying to acquire lock");

        *lock
    }

    async fn has_stack(&self) -> bool {
        let inv = self.get_inventory();
        !inv.get_stack(self.get_index())
            .await
            .lock()
            .await
            .is_empty()
    }

    /// Make sure to drop any locks to the slot stack before calling this
    async fn set_stack(&self, stack: ItemStack) {
        self.set_stack_no_callbacks(stack).await;
    }

    async fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) {
        self.set_stack_no_callbacks(stack).await;
    }

    async fn set_stack_no_callbacks(&self, stack: ItemStack) {
        let inv = self.get_inventory();
        inv.set_stack(self.get_index(), stack).await;
        self.mark_dirty().await;
    }

    async fn mark_dirty(&self);

    async fn get_max_item_count(&self) -> u8 {
        self.get_inventory().get_max_count_per_stack()
    }

    async fn get_max_item_count_for_stack(&self, stack: &ItemStack) -> u8 {
        self.get_max_item_count()
            .await
            .min(stack.get_max_stack_size())
    }

    async fn take_stack(&self, amount: u8) -> ItemStack {
        let inv = self.get_inventory();

        inv.remove_stack_specific(self.get_index(), amount).await
    }

    async fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        true
    }

    async fn try_take_stack_range(
        &self,
        min: u8,
        max: u8,
        _player: &dyn InventoryPlayer,
    ) -> Option<ItemStack> {
        let min = min.min(max);
        let stack = self.take_stack(min).await;

        if stack.is_empty() {
            None
        } else {
            if self
                .get_inventory()
                .get_stack(self.get_index())
                .await
                .lock()
                .await
                .is_empty()
            {
                self.set_stack_prev(ItemStack::EMPTY, stack).await;
            }

            Some(stack)
        }
    }

    async fn take_stack_range(&self, min: u8, max: u8, player: &dyn InventoryPlayer) -> ItemStack {
        let stack = self.try_take_stack_range(min, max, player).await;

        if let Some(stack) = &stack {
            self.on_take_item(player, stack).await;
        }

        stack.unwrap_or(ItemStack::EMPTY)
    }

    async fn insert_stack(&self, stack: ItemStack) -> ItemStack {
        let stack_item_count = stack.item_count;
        self.insert_stack_count(stack, stack_item_count).await
    }

    async fn insert_stack_count(&self, mut stack: ItemStack, count: u8) -> ItemStack {
        if !stack.is_empty() && self.can_insert(&stack).await {
            let stack_mutex = self.get_stack().await;
            let mut stack_self = stack_mutex.lock().await;
            let min_count = count
                .min(stack.item_count)
                .min(self.get_max_item_count_for_stack(&stack).await - stack_self.item_count);

            if min_count == 0 {
                return stack;
            } else {
                if stack_self.is_empty() {
                    drop(stack_self);
                    self.set_stack(stack.split(min_count)).await;
                } else if stack.are_items_and_components_equal(&stack_self) {
                    stack.decrement(min_count);
                    stack_self.increment(min_count);
                    let cloned_stack = *stack_self;
                    drop(stack_self);
                    self.set_stack(cloned_stack).await;
                }

                return stack;
            }
        } else {
            stack
        }
    }
}

#[derive(Debug)]
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
#[async_trait]
impl Slot for NormalSlot {
    fn get_inventory(&self) -> &Arc<dyn Inventory> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

// ArmorSlot.java
#[derive(Debug)]
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

#[async_trait]
impl Slot for ArmorSlot {
    fn get_inventory(&self) -> &Arc<dyn Inventory> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    async fn get_max_item_count(&self) -> u8 {
        1
    }

    async fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) {
        //TODO: this.entity.onEquipStack(this.equipmentSlot, previousStack, stack);
        self.set_stack_no_callbacks(stack).await;
    }

    async fn can_insert(&self, _stack: &ItemStack) -> bool {
        // TODO: return this.entity.canEquip(stack, this.equipmentSlot);
        true
    }

    async fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        // TODO: Check enchantments
        true
    }

    async fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}
