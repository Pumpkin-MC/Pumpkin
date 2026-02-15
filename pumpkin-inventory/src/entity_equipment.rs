use std::{collections::HashMap, sync::Arc};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

// EntityEquipment.java
#[derive(Clone)]
pub struct EntityEquipment {
    pub equipment: HashMap<EquipmentSlot, Arc<Mutex<ItemStack>>>,
}

impl Default for EntityEquipment {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityEquipment {
    #[must_use]
    pub fn new() -> Self {
        let mut equipment = HashMap::new();
        for slot in [
            EquipmentSlot::OFF_HAND,
            EquipmentSlot::FEET,
            EquipmentSlot::LEGS,
            EquipmentSlot::CHEST,
            EquipmentSlot::HEAD,
        ] {
            equipment.insert(slot, Arc::new(Mutex::new(ItemStack::EMPTY.clone())));
        }

        Self { equipment }
    }

    pub async fn put(&mut self, slot: &EquipmentSlot, stack: ItemStack) -> ItemStack {
        if let Some(existing) = self.equipment.get(slot) {
            let mut existing = existing.lock().await;
            return std::mem::replace(&mut *existing, stack);
        }

        self.equipment
            .insert(slot.clone(), Arc::new(Mutex::new(stack)));
        ItemStack::EMPTY.clone()
    }

    #[must_use]
    pub fn get(&self, slot: &EquipmentSlot) -> Arc<Mutex<ItemStack>> {
        self.equipment
            .get(slot)
            .cloned()
            .unwrap_or(Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
    }

    pub async fn is_empty(&self) -> bool {
        for stack in self.equipment.values() {
            if !stack.lock().await.is_empty() {
                return false;
            }
        }

        true
    }

    pub async fn clear(&mut self) {
        let stacks = self.equipment.values().cloned().collect::<Vec<_>>();
        for stack in stacks {
            *stack.lock().await = ItemStack::EMPTY.clone();
        }
    }

    // TODO: tick
}
