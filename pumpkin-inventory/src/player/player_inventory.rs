use crate::entity_equipment::EntityEquipment;
use crate::equipment_slot::EquipmentSlot;
use crate::inventory::{Clearable, Inventory};
use crate::split_stack;
use async_trait::async_trait;
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
    pub fn new(entity_equipment: EntityEquipment) -> Self {
        Self {
            // Normal syntax can't be used here because Arc doesn't implement Copy
            main_inventory: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            equipment_slots: Self::build_equipment_slots(),
            selected_slot: AtomicU8::new(0),
            entity_equipment: Arc::new(Mutex::new(entity_equipment)),
        }
    }

    /// getSelectedStack in source
    pub fn held_item(&self) -> Arc<Mutex<ItemStack>> {
        self.main_inventory
            .get(self.get_selected_slot() as usize)
            .unwrap()
            .clone()
    }

    pub fn is_valid_hotbar_index(slot: usize) -> bool {
        slot <= Self::HOTBAR_SIZE
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
        equipment_slots.insert(40, EquipmentSlot::OFF_HAND);
        equipment_slots
    }

    async fn add_stack(&self, stack: ItemStack) -> usize {
        let mut slot_index = self.get_occupied_slot_with_room_for_stack(&stack).await;

        println!("slot_index: {}", slot_index);

        if slot_index == -1 {
            slot_index = self.get_empty_slot().await;
        }

        if slot_index == -1 {
            return stack.item_count as usize;
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

        println!("self_stack: {:?}", self_stack);

        let count_left = self_stack.get_max_stack_size() - self_stack.item_count;
        let count_min = stack_count.min(count_left);

        if count_min == 0 {
            return stack_count as usize;
        } else {
            stack_count -= count_min;
            self_stack.increment(count_min);
            return stack_count as usize;
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

    fn can_stack_add_more(&self, exsiting_stack: &ItemStack, stack: &ItemStack) -> bool {
        return !exsiting_stack.is_empty()
            && exsiting_stack.are_items_and_components_equal(stack)
            && exsiting_stack.is_stackable()
            && exsiting_stack.item_count < exsiting_stack.get_max_stack_size();
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
            return self.get_selected_slot() as i16;
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

        return stack.item_count < i;
    }
}

impl Clearable for PlayerInventory {
    fn clear(&self) {
        todo!()
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
        println!("set_stack: {:?}, slot: {}", stack, slot);
        if slot < self.main_inventory.len() {
            *self.main_inventory[slot].lock().await = stack;
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            println!("setting stack: {:?}, slot: {:?}", stack, slot);
            self.entity_equipment.lock().await.put(slot, stack).await;
        }
    }

    fn mark_dirty(&self) {}
}

pub fn is_valid_hotbar_slot(slot: u8) -> bool {
    slot < 9
}

impl PlayerInventory {
    pub fn set_selected_slot(&self, slot: u8) {
        if is_valid_hotbar_slot(slot) {
            self.selected_slot
                .store(slot, std::sync::atomic::Ordering::Relaxed);
        } else {
            panic!("Invalid hotbar slot: {}", slot);
        }
    }

    pub fn get_selected_slot(&self) -> u8 {
        self.selected_slot
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
