use std::{collections::HashMap, sync::Arc};

use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::equipment_slot::EquipmentSlot;

// EntityEquipment.java
#[derive(Debug, Clone)]
pub struct EntityEquipment {
    pub equipment: HashMap<EquipmentSlot, Arc<Mutex<ItemStack>>>,
}

impl Default for EntityEquipment {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityEquipment {
    pub fn new() -> Self {
        Self {
            equipment: HashMap::new(),
        }
    }

    pub async fn put(&mut self, slot: &EquipmentSlot, stack: ItemStack) -> ItemStack {
        *self
            .equipment
            .insert(slot.clone(), Arc::new(Mutex::new(stack)))
            .unwrap_or(Arc::new(Mutex::new(ItemStack::EMPTY)))
            .lock()
            .await
    }

    pub fn get(&self, slot: &EquipmentSlot) -> Arc<Mutex<ItemStack>> {
        self.equipment
            .get(slot)
            .cloned()
            .unwrap_or(Arc::new(Mutex::new(ItemStack::EMPTY)))
    }

    pub async fn is_empty(&self) -> bool {
        for stack in self.equipment.values() {
            if !stack.lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    pub fn clear(&mut self) {
        self.equipment.clear();
    }

    // TODO: tick
}
