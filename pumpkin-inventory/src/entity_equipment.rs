use std::collections::HashMap;

use pumpkin_world::item::ItemStack;

use crate::equipment_slot::EquipmentSlot;

// EntityEquipment.java
#[derive(Debug, Clone)]
pub struct EntityEquipment {
    pub equipment: HashMap<EquipmentSlot, ItemStack>,
}

impl EntityEquipment {
    pub fn new() -> Self {
        Self {
            equipment: HashMap::new(),
        }
    }

    pub fn put(&mut self, slot: &EquipmentSlot, stack: ItemStack) {
        self.equipment.insert(slot.clone(), stack);
    }

    pub fn remove(&mut self, slot: &EquipmentSlot) -> ItemStack {
        self.equipment.remove(slot).unwrap_or(ItemStack::EMPTY)
    }

    pub fn get(&self, slot: &EquipmentSlot) -> &ItemStack {
        self.equipment.get(slot).unwrap_or(&ItemStack::EMPTY)
    }

    pub fn get_mut(&mut self, slot: &EquipmentSlot) -> &mut ItemStack {
        self.equipment
            .entry(slot.clone())
            .or_insert(ItemStack::EMPTY)
    }

    pub fn is_empty(&self) -> bool {
        for stack in self.equipment.values() {
            if !stack.is_empty() {
                return false;
            }
        }

        true
    }

    // TODO: tick
}
