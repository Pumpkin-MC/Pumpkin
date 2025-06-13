use crate::entity_equipment::EntityEquipment;
use crate::equipment_slot::EquipmentSlot;
use crate::screen_handler::InventoryPlayer;
use async_trait::async_trait;
use pumpkin_protocol::client::play::CSetPlayerInventory;
use pumpkin_world::inventory::split_stack;
use pumpkin_world::inventory::{Clearable, Inventory};
use pumpkin_world::item::ItemStack;
use std::array::from_fn;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct PlayerInventory {
    pub main_inventory: [Arc<Mutex<ItemStack>>; Self::MAIN_SIZE],
    pub equipment_slots: HashMap<usize, EquipmentSlot>,
    selected_slot: AtomicU8,
    pub entity_equipment: Arc<Mutex<EntityEquipment>>,
}

impl PlayerInventory {
    const MAIN_SIZE: usize = 36;
    const HOTBAR_SIZE: usize = 9;
    const OFF_HAND_SLOT: usize = 40;

    // TODO: Add inventory load from nbt
    pub fn new(entity_equipment: Arc<Mutex<EntityEquipment>>) -> Self {
        Self {
            // Normal syntax can't be used here because Arc doesn't implement Copy
            main_inventory: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            equipment_slots: Self::build_equipment_slots(),
            selected_slot: AtomicU8::new(0),
            entity_equipment,
        }
    }

    /// getSelectedStack in source
    pub fn held_item(&self) -> Arc<Mutex<ItemStack>> {
        self.main_inventory
            .get(self.get_selected_slot() as usize)
            .unwrap()
            .clone()
    }

    pub async fn swap_item(&self) -> (ItemStack, ItemStack) {
        let slot = self
            .equipment_slots
            .get(&PlayerInventory::OFF_HAND_SLOT)
            .unwrap();
        let mut equipment = self.entity_equipment.lock().await;
        let binding = self.held_item();
        let mut main_hand_item = binding.lock().await;
        let off_hand_item = *main_hand_item;
        *main_hand_item = equipment.put(slot, off_hand_item).await;
        (*main_hand_item, off_hand_item)
    }

    pub fn is_valid_hotbar_index(slot: usize) -> bool {
        slot < Self::HOTBAR_SIZE
    }

    fn build_equipment_slots() -> HashMap<usize, EquipmentSlot> {
        let mut equipment_slots = HashMap::new();
        equipment_slots.insert(
            EquipmentSlot::FEET.get_offset_entity_slot_id(Self::MAIN_SIZE as i32) as usize,
            EquipmentSlot::FEET,
        );
        equipment_slots.insert(
            EquipmentSlot::LEGS.get_offset_entity_slot_id(Self::MAIN_SIZE as i32) as usize,
            EquipmentSlot::LEGS,
        );
        equipment_slots.insert(
            EquipmentSlot::CHEST.get_offset_entity_slot_id(Self::MAIN_SIZE as i32) as usize,
            EquipmentSlot::CHEST,
        );
        equipment_slots.insert(
            EquipmentSlot::HEAD.get_offset_entity_slot_id(Self::MAIN_SIZE as i32) as usize,
            EquipmentSlot::HEAD,
        );
        equipment_slots.insert(PlayerInventory::OFF_HAND_SLOT, EquipmentSlot::OFF_HAND);
        equipment_slots
    }

    async fn add_stack(&self, stack: ItemStack) -> usize {
        let mut slot_index = self.get_occupied_slot_with_room_for_stack(&stack).await;

        if slot_index == -1 {
            slot_index = self.get_empty_slot().await;
        }

        if slot_index == -1 {
            stack.item_count as usize
        } else {
            return self.add_stack_to_slot(slot_index as usize, stack).await;
        }
    }

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

        if count_min == 0 {
            stack_count as usize
        } else {
            stack_count -= count_min;
            self_stack.increment(count_min);
            stack_count as usize
        }
    }

    async fn get_empty_slot(&self) -> i16 {
        for i in 0..Self::MAIN_SIZE {
            if self.main_inventory[i].lock().await.is_empty() {
                return i as i16;
            }
        }

        -1
    }

    fn can_stack_add_more(&self, existing_stack: &ItemStack, stack: &ItemStack) -> bool {
        !existing_stack.is_empty()
            && existing_stack.are_items_and_components_equal(stack)
            && existing_stack.is_stackable()
            && existing_stack.item_count < existing_stack.get_max_stack_size()
    }

    async fn get_occupied_slot_with_room_for_stack(&self, stack: &ItemStack) -> i16 {
        if self.can_stack_add_more(
            &*self
                .get_stack(self.get_selected_slot() as usize)
                .await
                .lock()
                .await,
            stack,
        ) {
            self.get_selected_slot() as i16
        } else if self.can_stack_add_more(
            &*self.get_stack(Self::OFF_HAND_SLOT).await.lock().await,
            stack,
        ) {
            return Self::OFF_HAND_SLOT as i16;
        } else {
            for i in 0..Self::MAIN_SIZE {
                if self.can_stack_add_more(&*self.main_inventory[i].lock().await, stack) {
                    return i as i16;
                }
            }

            return -1;
        }
    }

    pub async fn insert_stack_anywhere(&self, stack: &mut ItemStack) -> bool {
        self.insert_stack(-1, stack).await
    }

    pub async fn insert_stack(&self, slot: i16, stack: &mut ItemStack) -> bool {
        if stack.is_empty() {
            return false;
        }

        // TODO: if (stack.isDamaged()) {

        let mut i;

        loop {
            i = stack.item_count;
            if slot == -1 {
                stack.set_count(self.add_stack(*stack).await as u8);
            } else {
                stack.set_count(self.add_stack_to_slot(slot as usize, *stack).await as u8);
            }

            if stack.is_empty() || stack.item_count >= i {
                break;
            }
        }

        // TODO: Creative mode check

        stack.item_count < i
    }

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

    pub async fn get_swappable_hotbar_slot(&self) -> usize {
        let selected_slot = self.get_selected_slot() as usize;
        for i in 0..Self::HOTBAR_SIZE {
            let check_index = (i + selected_slot) % 9;
            if self.main_inventory[check_index].lock().await.is_empty() {
                return check_index;
            }
        }

        for i in 0..Self::HOTBAR_SIZE {
            let check_index = (i + selected_slot) % 9;
            if true
            /*TODO: If item has an enchantment skip it */
            {
                return check_index;
            }
        }

        self.get_selected_slot() as usize
    }

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
                    *self.main_inventory[self.get_selected_slot() as usize]
                        .lock()
                        .await,
                )
                .await;
            }
        }

        self.set_stack(self.get_selected_slot() as usize, stack)
            .await;
    }

    pub async fn swap_slot_with_hotbar(&self, slot: usize) {
        self.set_selected_slot(self.get_swappable_hotbar_slot().await as u8);
        let stack = *self.main_inventory[self.get_selected_slot() as usize]
            .lock()
            .await;
        self.set_stack(
            self.get_selected_slot() as usize,
            *self.main_inventory[slot].lock().await,
        )
        .await;
        self.set_stack(slot, stack).await;
    }

    pub async fn offer_or_drop_stack(&self, stack: ItemStack, player: &dyn InventoryPlayer) {
        self.offer(stack, true, player).await;
    }

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
                        (room_for_stack as i32).into(),
                        &stack.into(),
                    ))
                    .await;
            }
        }
    }
}

#[async_trait]
impl Clearable for PlayerInventory {
    async fn clear(&self) {
        for item in self.main_inventory.iter() {
            *item.lock().await = ItemStack::EMPTY;
        }

        self.entity_equipment.lock().await.clear();
    }
}

#[async_trait]
impl Inventory for PlayerInventory {
    fn size(&self) -> usize {
        self.main_inventory.len() + self.equipment_slots.len()
    }

    async fn is_empty(&self) -> bool {
        for item in self.main_inventory.iter() {
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
    }

    async fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        if slot < self.main_inventory.len() {
            self.main_inventory[slot].clone()
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.lock().await.get(slot)
        }
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot < self.main_inventory.len() {
            split_stack(&self.main_inventory, slot, amount).await
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();

            let equipment = self.entity_equipment.lock().await.get(slot);
            let mut stack = equipment.lock().await;

            if !stack.is_empty() {
                return stack.split(amount);
            }

            ItemStack::EMPTY
        }
    }

    async fn remove_stack(&self, slot: usize) -> ItemStack {
        if slot < self.main_inventory.len() {
            let mut removed = ItemStack::EMPTY;
            let mut guard = self.main_inventory[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment
                .lock()
                .await
                .put(slot, ItemStack::EMPTY)
                .await
        }
    }

    async fn set_stack(&self, slot: usize, stack: ItemStack) {
        if slot < self.main_inventory.len() {
            *self.main_inventory[slot].lock().await = stack;
        } else {
            match self.equipment_slots.get(&slot) {
                Some(slot) => {
                    self.entity_equipment.lock().await.put(slot, stack).await;
                }
                None => log::warn!("Failed to get Equipment Slot at {slot}"),
            }
        }
    }

    fn mark_dirty(&self) {}
}

impl PlayerInventory {
    pub fn set_selected_slot(&self, slot: u8) {
        if Self::is_valid_hotbar_index(slot as usize) {
            self.selected_slot
                .store(slot, std::sync::atomic::Ordering::Relaxed);
        } else {
            panic!("Invalid hotbar slot: {slot}");
        }
    }

    pub fn get_selected_slot(&self) -> u8 {
        self.selected_slot
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
