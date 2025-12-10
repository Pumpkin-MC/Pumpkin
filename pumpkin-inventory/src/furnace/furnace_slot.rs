use std::sync::{Arc, atomic::AtomicU8};

use pumpkin_data::{fuels::is_fuel, item::Item};
use pumpkin_world::inventory::Inventory;

use crate::slot::{BoxFuture, Slot};

#[derive(Debug, Clone, Copy)]
pub enum FurnaceSlotType {
    Top = 0,
    Bottom = 1,
    Side = 2,
}

pub struct FurnaceSlot {
    pub inventory: Arc<dyn Inventory>,
    pub slot_type: FurnaceSlotType,
    pub index: usize,
    pub id: AtomicU8,
}

impl FurnaceSlot {
    pub fn new(inventory: Arc<dyn Inventory>, slot_type: FurnaceSlotType) -> Self {
        Self {
            inventory,
            slot_type,
            index: slot_type as usize,
            id: AtomicU8::new(0),
        }
    }
}
impl Slot for FurnaceSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id
            .store(id as u8, std::sync::atomic::Ordering::Relaxed);
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }

    fn can_insert<'a>(&'a self, stack: &'a pumpkin_world::item::ItemStack) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self.slot_type {
                FurnaceSlotType::Top => true,
                FurnaceSlotType::Bottom => {
                    is_fuel(stack.item.id) || stack.item.id == Item::BUCKET.id
                }
                FurnaceSlotType::Side => false,
            }
        })
    }
}
