use std::sync::{Arc, atomic::AtomicU8};

use async_trait::async_trait;
use pumpkin_inventory::slot::Slot;
use pumpkin_world::inventory::Inventory;

use crate::item::fuel_registry::FuelRegistry;

#[derive(Debug, Clone, Copy)]
pub enum FurnaceSlotType {
    Top = 0,
    Bottom = 1,
    Side = 2,
}

#[derive(Debug)]
pub struct FurnaceSlot {
    pub fuel_registry: Arc<FuelRegistry>,
    pub inventory: Arc<dyn Inventory>,
    pub slot_type: FurnaceSlotType,
    pub index: usize,
    pub id: AtomicU8,
}

impl FurnaceSlot {
    pub fn new(
        fuel_registry: Arc<FuelRegistry>,
        inventory: Arc<dyn Inventory>,
        slot_type: FurnaceSlotType,
    ) -> Self {
        Self {
            fuel_registry,
            inventory,
            slot_type,
            index: slot_type as usize,
            id: AtomicU8::new(0),
        }
    }
}
#[async_trait]
impl Slot for FurnaceSlot {
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

    async fn can_insert(&self, stack: &pumpkin_world::item::ItemStack) -> bool {
        match self.slot_type {
            FurnaceSlotType::Top => true,
            FurnaceSlotType::Bottom => self.fuel_registry.is_fuel(stack.item),
            FurnaceSlotType::Side => false,
        }
    }
}
