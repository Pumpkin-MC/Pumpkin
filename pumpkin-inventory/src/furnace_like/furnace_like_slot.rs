use std::sync::{Arc, atomic::AtomicU8};

use pumpkin_data::{fuels::is_fuel, item::Item};
use pumpkin_world::{
    block::entities::furnace_like_block_entity::ExperienceContainer, inventory::Inventory,
};

use tracing::debug;

use crate::{
    screen_handler::InventoryPlayer,
    slot::{BoxFuture, Slot},
};

#[derive(Debug, Clone, Copy)]
pub enum FurnaceLikeSlotType {
    Top = 0,
    Bottom = 1,
}

/// Slot for furnace input (top) and fuel (bottom)
pub struct FurnaceLikeSlot {
    pub inventory: Arc<dyn Inventory>,
    pub slot_type: FurnaceLikeSlotType,
    pub index: usize,
    pub id: AtomicU8,
}

impl FurnaceLikeSlot {
    pub fn new(inventory: Arc<dyn Inventory>, slot_type: FurnaceLikeSlotType) -> Self {
        Self {
            inventory,
            slot_type,
            index: slot_type as usize,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for FurnaceLikeSlot {
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
                FurnaceLikeSlotType::Top => true,
                FurnaceLikeSlotType::Bottom => {
                    is_fuel(stack.item.id) || stack.item.id == Item::BUCKET.id
                }
            }
        })
    }
}

/// Output slot for furnace that awards experience when items are taken
pub struct FurnaceOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub experience_container: Arc<dyn ExperienceContainer>,
    pub id: AtomicU8,
}

impl FurnaceOutputSlot {
    pub fn new(
        inventory: Arc<dyn Inventory>,
        experience_container: Arc<dyn ExperienceContainer>,
    ) -> Self {
        Self {
            inventory,
            experience_container,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for FurnaceOutputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        2 // Output slot is always index 2
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

    fn can_insert<'a>(&'a self, _stack: &'a pumpkin_world::item::ItemStack) -> BoxFuture<'a, bool> {
        // Cannot insert items into the output slot
        Box::pin(async move { false })
    }

    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        _stack: &'a pumpkin_world::item::ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            debug!("FurnaceOutputSlot: on_take_item called");
            // Extract accumulated experience and award to player
            let experience = self.experience_container.extract_experience();
            debug!("FurnaceOutputSlot: extracted experience = {experience}");
            if experience > 0 {
                debug!("FurnaceOutputSlot: awarding {experience} xp to player");
                player.award_experience(experience).await;
            }
            self.mark_dirty().await;
        })
    }
}
