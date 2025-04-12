use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{equipment_slot::EquipmentSlot, inventory::Inventory};

// Slot.java
// This is a trait due to crafting slots being a thing
#[async_trait]
pub trait Slot: Send + Sync + Debug {
    fn get_inventory(&self) -> &Arc<Mutex<dyn Inventory>>;

    fn get_index(&self) -> usize;

    fn set_id(&mut self, index: usize);

    fn on_quick_transfer(&self, new_item: ItemStack, original: ItemStack) {
        let diff = new_item.item_count - original.item_count;
        if diff > 0 {
            self.on_crafted(original, diff);
        }
    }

    fn on_crafted(&self, _stack: ItemStack, _amount: u8) {}

    fn on_crafted_single(&self, _stack: ItemStack) {}

    fn on_take(&self, _amount: u8) {}

    // TODO: Source takes player as parameter
    async fn on_take_item(&self, _stack: &ItemStack) {
        self.mark_dirty().await;
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        true
    }

    /* Not used due to Rust's borrow checker, use get_inventory() instead
    async fn get_stack(&self) -> &mut ItemStack {

    }
     */

    async fn has_stack(&self) -> bool {
        let mut inv = self.get_inventory().lock().await;
        !inv.get_stack(self.get_index()).is_empty()
    }

    async fn set_stack(&self, stack: ItemStack) {
        self.set_stack_no_callbacks(stack).await;
    }

    async fn set_stack_prev(&self, stack: ItemStack, _previous_stack: ItemStack) {
        self.set_stack_no_callbacks(stack).await;
    }

    async fn set_stack_no_callbacks(&self, stack: ItemStack) {
        let mut inv = self.get_inventory().lock().await;
        inv.set_stack(self.get_index(), stack);
        drop(inv);
        self.mark_dirty().await;
    }

    async fn mark_dirty(&self);

    async fn get_max_item_count(&self) -> u8 {
        self.get_inventory().lock().await.get_max_count_per_stack()
    }

    async fn get_max_item_count_for_stack(&self, stack: &ItemStack) -> u8 {
        self.get_max_item_count()
            .await
            .min(stack.get_max_stack_size())
    }

    async fn take_stack(&self, amount: u8) -> ItemStack {
        let mut inv = self.get_inventory().lock().await;
        let stack = inv.remove_stack_specific(self.get_index(), amount);
        drop(inv);
        stack
    }

    // TODO: Source takes player as parameter
    fn can_take_items(&self) -> bool {
        true
    }

    fn is_enabled(&self) -> bool {
        true
    }

    async fn try_take_stack_range(&self, min: u8, max: u8) -> Option<ItemStack> {
        // TODO: Player is passed in here

        let min = min.min(max);
        let stack = self.take_stack(min).await;

        if stack.is_empty() {
            None
        } else {
            if self
                .get_inventory()
                .lock()
                .await
                .get_stack(self.get_index())
                .is_empty()
            {
                self.set_stack_prev(ItemStack::EMPTY, stack.clone()).await;
            }

            Some(stack)
        }
    }

    async fn take_stack_range(&self, min: u8, max: u8) -> ItemStack {
        let stack = self.try_take_stack_range(min, max).await;

        if let Some(stack) = &stack {
            self.on_take_item(stack);
        }

        stack.unwrap_or(ItemStack::EMPTY)
    }

    async fn insert_stack(&self, stack: ItemStack) -> ItemStack {
        let stack_item_count = stack.item_count;
        self.insert_stack_count(stack, stack_item_count).await
    }

    async fn insert_stack_count(&self, mut stack: ItemStack, count: u8) -> ItemStack {
        if !stack.is_empty() && self.can_insert(&stack) {
            let mut inv = self.get_inventory().lock().await;
            let stack_self = inv.get_stack(self.get_index());
            let min_count = count
                .min(stack.item_count)
                .min(self.get_max_item_count_for_stack(&stack).await - stack_self.item_count);

            if min_count <= 0 {
                return stack;
            } else {
                if stack_self.is_empty() {
                    self.set_stack(stack.split(min_count)).await;
                } else if stack.are_items_and_components_equal(&stack_self) {
                    stack.decrement(min_count);
                    stack_self.increment(min_count);
                    self.set_stack(stack_self.clone()).await;
                }

                return stack;
            }
        } else {
            stack
        }
    }
}

#[derive(Debug, Clone)]
/// Just called Slot in Vanilla
pub struct NormalSlot {
    pub inventory: Arc<Mutex<dyn Inventory>>,
    pub index: usize,
    pub id: usize,
}

impl NormalSlot {
    pub fn new(inventory: Arc<Mutex<dyn Inventory>>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: 0,
        }
    }
}
#[async_trait]
impl Slot for NormalSlot {
    fn get_inventory(&self) -> &Arc<Mutex<dyn Inventory>> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    async fn mark_dirty(&self) {
        self.inventory.lock().await.mark_dirty();
    }
}

// ArmorSlot.java
#[derive(Debug, Clone)]
pub struct ArmorSlot {
    pub inventory: Arc<Mutex<dyn Inventory>>,
    pub index: usize,
    pub id: usize,
    pub equipment_slot: EquipmentSlot,
}

impl ArmorSlot {
    pub fn new(
        inventory: Arc<Mutex<dyn Inventory>>,
        index: usize,
        equipment_slot: EquipmentSlot,
    ) -> Self {
        Self {
            inventory,
            index,
            id: 0,
            equipment_slot,
        }
    }
}

#[async_trait]
impl Slot for ArmorSlot {
    fn get_inventory(&self) -> &Arc<Mutex<dyn Inventory>> {
        &self.inventory
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    async fn get_max_item_count(&self) -> u8 {
        1
    }

    async fn set_stack_prev(&self, stack: ItemStack, previous_stack: ItemStack) {
        //TODO: this.entity.onEquipStack(this.equipmentSlot, previousStack, stack);
        Slot::set_stack_prev(self, stack, previous_stack).await;
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        // TODO: return this.entity.canEquip(stack, this.equipmentSlot);
        true
    }

    fn can_take_items(&self) -> bool {
        // TODO: Check enchantments
        true
    }

    async fn mark_dirty(&self) {
        self.inventory.lock().await.mark_dirty();
    }
}
