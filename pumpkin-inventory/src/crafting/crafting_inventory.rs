use std::sync::Arc;
use std::{any::Any, pin::Pin};

use pumpkin_world::{inventory::split_stack, item::ItemStack};
use tokio::sync::Mutex;

use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};

use super::recipes::RecipeInputInventory;

#[derive(Clone)]
pub struct CraftingInventory {
    pub width: u8,
    pub height: u8,
    pub items: Vec<Arc<Mutex<ItemStack>>>,
}

impl CraftingInventory {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crafting_inventory_3x3_size() {
        let inv = CraftingInventory::new(3, 3);
        assert_eq!(inv.size(), 9);
        assert_eq!(inv.get_width(), 3);
        assert_eq!(inv.get_height(), 3);
    }

    #[test]
    fn crafting_inventory_2x2_size() {
        let inv = CraftingInventory::new(2, 2);
        assert_eq!(inv.size(), 4);
        assert_eq!(inv.get_width(), 2);
        assert_eq!(inv.get_height(), 2);
    }

    #[test]
    fn crafting_inventory_1x1_size() {
        let inv = CraftingInventory::new(1, 1);
        assert_eq!(inv.size(), 1);
    }

    #[tokio::test]
    async fn crafting_inventory_starts_empty() {
        let inv = CraftingInventory::new(3, 3);
        assert!(inv.is_empty().await);
    }

    #[tokio::test]
    async fn crafting_inventory_set_and_get_stack() {
        let inv = CraftingInventory::new(3, 3);
        let item = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        inv.set_stack(0, item.clone()).await;

        let stack = inv.get_stack(0).await;
        let stack = stack.lock().await;
        assert_eq!(stack.item_count, 1);
        assert!(!stack.is_empty());
    }

    #[tokio::test]
    async fn crafting_inventory_not_empty_after_set() {
        let inv = CraftingInventory::new(3, 3);
        let item = ItemStack::new(1, &pumpkin_data::item::Item::STONE);
        inv.set_stack(4, item).await;
        assert!(!inv.is_empty().await);
    }

    #[tokio::test]
    async fn crafting_inventory_remove_stack() {
        let inv = CraftingInventory::new(3, 3);
        let item = ItemStack::new(5, &pumpkin_data::item::Item::STONE);
        inv.set_stack(0, item).await;

        let removed = inv.remove_stack(0).await;
        assert_eq!(removed.item_count, 5);

        // Slot should now be empty
        let stack = inv.get_stack(0).await;
        assert!(stack.lock().await.is_empty());
    }

    #[tokio::test]
    async fn crafting_inventory_remove_stack_specific() {
        let inv = CraftingInventory::new(3, 3);
        let item = ItemStack::new(10, &pumpkin_data::item::Item::STONE);
        inv.set_stack(0, item).await;

        let removed = inv.remove_stack_specific(0, 3).await;
        assert_eq!(removed.item_count, 3);

        // 7 remaining
        let stack = inv.get_stack(0).await;
        assert_eq!(stack.lock().await.item_count, 7);
    }

    #[tokio::test]
    async fn crafting_inventory_clear() {
        let inv = CraftingInventory::new(3, 3);
        inv.set_stack(0, ItemStack::new(1, &pumpkin_data::item::Item::STONE))
            .await;
        inv.set_stack(4, ItemStack::new(2, &pumpkin_data::item::Item::DIRT))
            .await;

        assert!(!inv.is_empty().await);
        inv.clear().await;
        assert!(inv.is_empty().await);
    }
}
