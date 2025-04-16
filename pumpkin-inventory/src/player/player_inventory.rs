use crate::entity_equipment::EntityEquipment;
use crate::equipment_slot::EquipmentSlot;
use crate::inventory::{Clearable, Inventory};
use crate::split_stack;
use pumpkin_world::item::ItemStack;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlayerInventory {
    pub main_inventory: [ItemStack; Self::MAIN_SIZE],
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
            main_inventory: [ItemStack::EMPTY; Self::MAIN_SIZE],
            equipment_slots: Self::build_equipment_slots(),
            change_count: 0,
            selected_slot: 0,
            entity_equipment,
        }
    }

    /// getSelectedStack in source
    pub fn held_item(&self) -> &ItemStack {
        self.main_inventory
            .get(self.selected_slot as usize)
            .unwrap()
    }

    pub fn held_item_mut(&mut self) -> &mut ItemStack {
        self.main_inventory
            .get_mut(self.selected_slot as usize)
            .unwrap()
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

impl Inventory for PlayerInventory {
    fn size(&self) -> usize {
        self.main_inventory.len() + self.equipment_slots.len()
    }

    fn is_empty(&self) -> bool {
        for item in self.main_inventory.iter() {
            if !item.is_empty() {
                return false;
            }
        }

        for slot in self.equipment_slots.values() {
            if !self.entity_equipment.get(slot).is_empty() {
                return false;
            }
        }

        true
    }

    fn get_stack(&mut self, slot: usize) -> &mut ItemStack {
        if slot < self.main_inventory.len() {
            &mut self.main_inventory[slot]
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.get_mut(slot)
        }
    }

    fn get_stack_ref(&self, slot: usize) -> &ItemStack {
        if slot < self.main_inventory.len() {
            &self.main_inventory[slot]
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.get(slot)
        }
    }

    fn remove_stack_specific(&mut self, slot: usize, amount: u8) -> ItemStack {
        if slot < self.main_inventory.len() {
            split_stack(&mut self.main_inventory, slot, amount)
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();

            let stack = self.entity_equipment.get_mut(slot);
            if !stack.is_empty() {
                return stack.split(amount);
            }

            ItemStack::EMPTY
        }
    }

    fn remove_stack(&mut self, slot: usize) -> ItemStack {
        if slot < self.main_inventory.len() {
            let mut removed = ItemStack::EMPTY;
            std::mem::swap(&mut removed, &mut self.main_inventory[slot]);
            removed
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.remove(slot)
        }
    }

    fn set_stack(&mut self, slot: usize, stack: ItemStack) {
        if slot < self.main_inventory.len() {
            self.main_inventory[slot] = stack;
        } else {
            let slot = self.equipment_slots.get(&slot).unwrap();
            self.entity_equipment.put(slot, stack);
        }
    }

    fn mark_dirty(&mut self) {
        self.change_count += 1;
    }

    fn clone_box(&self) -> Box<dyn Inventory> {
        Box::new(self.clone())
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
