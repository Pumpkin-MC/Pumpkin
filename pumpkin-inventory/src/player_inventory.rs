use crate::equipment_slot::EquipmentSlot;
use crate::inventory::{Clearable, Inventory, InventoryIterator};
use pumpkin_world::item::ItemStack;
use std::collections::HashMap;

/*
    Inventory Layout:
    - 0: Crafting Output
    - 1-4: Crafting Input
    - 5-8: Armor
    - 9-35: Main Inventory
    - 36-44: Hotbar
    - 45: Offhand

*/

#[derive(Debug, Clone)]
pub struct PlayerInventory {
    pub main_inventory: [ItemStack; Self::MAIN_SIZE],
    pub equipment_slots: HashMap<i32, EquipmentSlot>,
    pub change_count: u32,
    pub selected_slot: usize,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInventory {
    const MAIN_SIZE: usize = 36;
    const HOTBAR_SIZE: usize = 9;
    const OFF_HAND_SLOT: usize = 45;

    // TODO: Add inventory load from nbt
    pub fn new() -> Self {
        Self {
            main_inventory: [ItemStack::EMPTY; Self::MAIN_SIZE],
            equipment_slots: Self::build_equipment_slots(),
            change_count: 0,
            selected_slot: 0,
        }
    }

    /// getSelectedStack in source
    pub fn held_item(&self) -> &ItemStack {
        self.main_inventory.get(self.selected_slot).unwrap()
    }

    pub fn held_item_mut(&mut self) -> &mut ItemStack {
        self.main_inventory.get_mut(self.selected_slot).unwrap()
    }

    pub fn is_valid_hotbar_index(slot: usize) -> bool {
        slot <= Self::HOTBAR_SIZE
    }

    fn build_equipment_slots() -> HashMap<i32, EquipmentSlot> {
        let mut equipment_slots = HashMap::new();
        equipment_slots.insert(
            EquipmentSlot::FEET.get_offset_entity_slot_id(Self::MAIN_SIZE as i32),
            EquipmentSlot::FEET,
        );
        equipment_slots.insert(
            EquipmentSlot::LEGS.get_offset_entity_slot_id(Self::MAIN_SIZE as i32),
            EquipmentSlot::LEGS,
        );
        equipment_slots.insert(
            EquipmentSlot::CHEST.get_offset_entity_slot_id(Self::MAIN_SIZE as i32),
            EquipmentSlot::CHEST,
        );
        equipment_slots.insert(
            EquipmentSlot::HEAD.get_offset_entity_slot_id(Self::MAIN_SIZE as i32),
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

        // TODO: Check equipment slots

        true
    }

    fn get_stack(&mut self, slot: usize) -> &mut ItemStack {
        if slot < self.main_inventory.len() {
            &mut self.main_inventory[slot]
        } else {
            todo!()
        }
    }

    fn get_stack_ref(&self, slot: usize) -> &ItemStack {
        if slot < self.main_inventory.len() {
            &self.main_inventory[slot]
        } else {
            todo!()
        }
    }

    fn remove_stack_specific(&mut self, slot: usize, amount: u8) -> ItemStack {
        todo!()
    }

    fn remove_stack(&mut self, slot: usize) -> ItemStack {
        todo!()
    }

    fn set_stack(&mut self, slot: usize, stack: ItemStack) {
        todo!()
    }

    fn mark_dirty(&mut self) {
        self.change_count += 1;
    }
}

impl IntoIterator for PlayerInventory {
    type Item = ItemStack;
    type IntoIter = InventoryIterator<PlayerInventory>;

    fn into_iter(self) -> Self::IntoIter {
        InventoryIterator::new(self)
    }
}
