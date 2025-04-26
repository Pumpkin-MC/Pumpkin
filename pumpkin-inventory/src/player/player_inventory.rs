use crate::entity_equipment::EntityEquipment;
use crate::equipment_slot::EquipmentSlot;
use crate::inventory::{Clearable, Inventory};
use crate::split_stack;
use async_trait::async_trait;
use pumpkin_world::item::ItemStack;
use std::array::from_fn;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PlayerInventory {
    pub main_inventory: [Arc<Mutex<ItemStack>>; Self::MAIN_SIZE],
    pub equipment_slots: HashMap<usize, EquipmentSlot>,
    change_count: u32,
    selected_slot: u8,
    pub entity_equipment: EntityEquipment,
}

impl PlayerInventory {
    const MAIN_SIZE: usize = 36;
    const HOTBAR_SIZE: usize = 9;
    const OFF_HAND_SLOT: usize = 45;

    // TODO: Add inventory load from nbt
    pub fn new(entity_equipment: EntityEquipment) -> Self {
        Self {
            // Normal syntax can't be used here because Arc doesn't implement Copy
            main_inventory: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY))),
            equipment_slots: Self::build_equipment_slots(),
            change_count: 0,
            selected_slot: 0,
            entity_equipment,
        }
    }

    /// getSelectedStack in source
    pub fn held_item(&self) -> Arc<Mutex<ItemStack>> {
        self.main_inventory
            .get(self.selected_slot as usize)
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
}

impl Clearable for PlayerInventory {
    fn clear(&mut self) {
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
            if !self.entity_equipment.get(slot).lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    fn get_stack(&self, slot: usize) -> Arc<Mutex<ItemStack>> {
        if slot < self.main_inventory.len() {
            self.main_inventory[slot].clone()
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.get(slot)
        }
    }

    async fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        if slot < self.main_inventory.len() {
            split_stack(&self.main_inventory, slot, amount).await
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();

            let equipment = self.entity_equipment.get(slot);
            let mut stack = equipment.lock().await;

            if !stack.is_empty() {
                return stack.split(amount);
            }

            ItemStack::EMPTY
        }
    }

    async fn remove_stack(&mut self, slot: usize) -> ItemStack {
        if slot < self.main_inventory.len() {
            let mut removed = ItemStack::EMPTY;
            let mut guard = self.main_inventory[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.put(slot, ItemStack::EMPTY).await
        }
    }

    async fn set_stack(&mut self, slot: usize, stack: ItemStack) {
        println!("set_stack: {:?}, slot: {}", stack, slot);
        if slot < self.main_inventory.len() {
            *self.main_inventory[slot].lock().await = stack;
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.put(slot, stack).await;
        }
    }

    fn mark_dirty(&mut self) {
        self.change_count += 1;
    }
}

pub fn is_valid_hotbar_slot(slot: u8) -> bool {
    slot < 9
}

impl PlayerInventory {
    pub fn set_selected_slot(&mut self, slot: u8) {
        if is_valid_hotbar_slot(slot) {
            self.selected_slot = slot;
        } else {
            panic!("Invalid hotbar slot: {}", slot);
        }
    }

    pub fn get_selected_slot(&self) -> u8 {
        self.selected_slot
    }
}
