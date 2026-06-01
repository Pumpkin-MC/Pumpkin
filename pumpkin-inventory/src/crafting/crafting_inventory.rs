//! Crafting inventory implementation.
//!
//! This module provides a temporary inventory for crafting grids.
//! Crafting inventories are used for:
//! - The 2x2 crafting grid in the player inventory
//! - The 3x3 crafting grid in crafting tables
//! - Other recipe-based crafting mechanisms
//!
//! Unlike regular inventories, crafting grids are typically cleared when
//! the container closes, and their contents are used up when crafting.

use std::sync::Arc;
use std::{any::Any, pin::Pin};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_world::inventory::split_stack;
use tokio::sync::Mutex;

use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};

use super::recipes::RecipeInputInventory;

/// A temporary inventory for crafting grids.
///
/// Crafting inventories hold items arranged in a grid pattern for crafting recipes.
/// The grid dimensions can vary (2x2 for player inventory, 3x3 for crafting table).
///
/// # Usage
///
/// When a player places items in the crafting grid, they are stored here.
/// When the crafting result is taken, the ingredients are consumed from this inventory.
#[derive(Clone)]
pub struct CraftingInventory {
    /// Width of the crafting grid (typically 2 or 3).
    pub width: u8,
    /// Height of the crafting grid (typically 2 or 3).
    pub height: u8,
    /// Items in the crafting grid, stored row by row.
    pub items: Vec<Arc<Mutex<ItemStack>>>,
}

impl CraftingInventory {
    /// Creates a new crafting inventory with the given dimensions.
    ///
    /// # Arguments
    /// - `width` - Grid width (e.g., 2 for player crafting, 3 for crafting table)
    /// - `height` - Grid height (e.g., 2 for player crafting, 3 for crafting table)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // 2x2 player inventory crafting grid
    /// let player_crafting = CraftingInventory::new(2, 2);
    ///
    /// // 3x3 crafting table grid
    /// let table_crafting = CraftingInventory::new(3, 3);
    /// ```
    #[must_use]
    pub fn new(width: u8, height: u8) -> Self {
        Self {
            width,
            height,
            items: {
                // Creates a Vec with different Mutexes for each slot
                let mut v = Vec::with_capacity(width as usize * height as usize);
                (0..width as usize * height as usize)
                    .for_each(|_| v.push(Arc::new(Mutex::new(ItemStack::EMPTY.clone()))));
                v
            },
        }
    }
}

impl Inventory for CraftingInventory {
    fn size(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.items {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }

            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.items[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut removed = ItemStack::EMPTY.clone();
            let mut guard = self.items[slot].lock().await;
            std::mem::swap(&mut removed, &mut *guard);
            removed
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move { split_stack(&self.items, slot, amount).await })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.items[slot].lock().await = stack;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl RecipeInputInventory for CraftingInventory {
    fn get_width(&self) -> usize {
        self.width as usize
    }

    fn get_height(&self) -> usize {
        self.height as usize
    }
}

impl Clearable for CraftingInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for item in &self.items {
                *item.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}
