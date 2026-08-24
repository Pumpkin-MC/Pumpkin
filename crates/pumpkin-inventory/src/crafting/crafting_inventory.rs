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

use std::{any::Any, pin::Pin};

use pumpkin_data::item_stack::ItemStack;
use tokio::sync::RwLock;

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
#[derive(Default)]
pub struct CraftingInventory {
    /// Width of the crafting grid (typically 2 or 3).
    pub width: u8,
    /// Height of the crafting grid (typically 2 or 3).
    pub height: u8,
    /// Items in the crafting grid, stored row by row.
    pub items: RwLock<Vec<ItemStack>>,
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
            items: RwLock::new(vec![
                ItemStack::EMPTY.clone();
                width as usize * height as usize
            ]),
        }
    }
}

impl Inventory for CraftingInventory {
    fn size(&self) -> usize {
        (self.width as usize) * (self.height as usize)
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            let items = self.items.read().await;
            items.iter().all(ItemStack::is_empty)
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let items = self.items.read().await;
            items
                .get(slot)
                .cloned()
                .unwrap_or_else(|| ItemStack::EMPTY.clone())
        })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            if slot < items.len() {
                std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            if slot < items.len() && !items[slot].is_empty() && amount > 0 {
                items[slot].split(amount)
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            let mut items = self.items.write().await;
            if slot < items.len() {
                items[slot] = stack;
            }
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
            let mut items = self.items.write().await;
            items.fill_with(|| ItemStack::EMPTY.clone());
        })
    }
}
