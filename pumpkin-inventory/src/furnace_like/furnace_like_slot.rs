//! Furnace-like slot implementations.
//!
//! This module provides specialized slot types for furnace-like containers.
//! Furnaces have three slots with specific behaviors:
//! - Input slot (top): Accepts any smeltable item
//! - Fuel slot (bottom): Only accepts fuel items (coal, charcoal, etc.)
//! - Output slot: Cannot receive items, awards experience when items are taken

use std::sync::{Arc, atomic::AtomicU8};

use pumpkin_data::{fuels::is_fuel, item::Item};
use pumpkin_world::{block::entities::ExperienceContainer, inventory::Inventory};

use tracing::debug;

use crate::{
    screen_handler::InventoryPlayer,
    slot::{BoxFuture, Slot},
};

/// Type of furnace slot.
#[derive(Debug, Clone, Copy)]
pub enum FurnaceLikeSlotType {
    /// Input slot (top) - accepts items to smelt.
    Top = 0,
    /// Fuel slot (bottom) - accepts fuel items.
    Bottom = 1,
}

/// Slot for furnace input or fuel.
///
/// The input slot accepts any item, while the fuel slot only accepts
/// valid fuel items (and empty buckets for lava fuel).
pub struct FurnaceLikeSlot {
    pub inventory: Arc<dyn Inventory>,
    pub slot_type: FurnaceLikeSlotType,
    pub index: usize,
    pub id: AtomicU8,
}

impl FurnaceLikeSlot {
    /// Creates a new furnace slot.
    ///
    /// # Arguments
    /// - `inventory` - The furnace's inventory
    /// - `slot_type` - Whether this is the input (Top) or fuel (Bottom) slot
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

    /// Restricts inserts based on slot type.
    ///
    /// - Top slot: accepts any item (smeltables)
    /// - Bottom slot: only accepts fuel items and buckets
    fn can_insert<'a>(
        &'a self,
        stack: &'a pumpkin_data::item_stack::ItemStack,
    ) -> BoxFuture<'a, bool> {
        Box::pin(async move {
            match self.slot_type {
                FurnaceLikeSlotType::Top => true,
                FurnaceLikeSlotType::Bottom => {
                    is_fuel(stack.item.id) || stack.item.id == Item::BUCKET.id
                }
            }
        })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}

/// Output slot for furnace-like containers.
///
/// This slot cannot receive items directly (items are placed here by smelting).
/// When items are taken from this slot, the player receives experience
/// based on the smelting recipes used.
pub struct FurnaceOutputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub experience_container: Arc<dyn ExperienceContainer>,
    pub id: AtomicU8,
}

impl FurnaceOutputSlot {
    /// Creates a new furnace output slot.
    ///
    /// # Arguments
    /// - `inventory` - The furnace's inventory
    /// - `experience_container` - Container that tracks accumulated experience
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

    /// Awards experience when items are taken from this slot.
    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        _stack: &'a pumpkin_data::item_stack::ItemStack,
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

    /// Output slot cannot receive inserted items.
    fn can_insert<'a>(
        &'a self,
        _stack: &'a pumpkin_data::item_stack::ItemStack,
    ) -> BoxFuture<'a, bool> {
        // Cannot insert items into the output slot
        Box::pin(async move { false })
    }

    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
}
