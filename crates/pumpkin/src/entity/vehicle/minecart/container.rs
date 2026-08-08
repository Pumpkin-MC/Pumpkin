use std::any::Any;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_hopper};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture, split_stack};
use tokio::sync::Mutex;

use crate::entity::{Entity, EntityBase, player::Player};
use crate::world::loot::fill_chest_inventory;
use pumpkin_data::chest_loot_table::get_chest_loot_table;

pub(super) struct MinecartInventory {
    items: Box<[Arc<Mutex<ItemStack>>]>,
    loot_table: StdMutex<Option<(String, i64)>>,
    drops_claimed: AtomicBool,
}

impl MinecartInventory {
    pub(super) fn new(size: usize) -> Self {
        Self {
            items: (0..size)
                .map(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
                .collect(),
            loot_table: StdMutex::new(None),
            drops_claimed: AtomicBool::new(false),
        }
    }

    pub(super) fn claim_drops(&self) -> bool {
        !self.drops_claimed.swap(true, Ordering::AcqRel)
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        let loot_table = nbt.get_string("LootTable").map(|loot_table| {
            (
                loot_table.to_owned(),
                nbt.get_long("LootTableSeed").unwrap_or(0),
            )
        });
        let has_loot_table = loot_table.is_some();
        *self.loot_table.lock().expect("Loot table mutex poisoned") = loot_table;

        if !has_loot_table {
            self.read_data(nbt, &self.items);
        }
    }

    pub(super) async fn write_nbt(&self, nbt: &mut NbtCompound) {
        let loot_table = self
            .loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .clone();
        if let Some((loot_table, seed)) = loot_table {
            nbt.put_string("LootTable", loot_table);
            if seed != 0 {
                nbt.put_long("LootTableSeed", seed);
            }
        } else {
            self.write_inventory_nbt(nbt, true).await;
        }
    }

    pub(super) fn has_loot_table(&self) -> bool {
        self.loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .is_some()
    }

    pub(super) async fn unpack_loot(self: &Arc<Self>) {
        let loot_table = self
            .loot_table
            .lock()
            .expect("Loot table mutex poisoned")
            .take();
        let Some((loot_table, seed)) = loot_table else {
            return;
        };
        let Some(table) = get_chest_loot_table(&loot_table) else {
            *self.loot_table.lock().expect("Loot table mutex poisoned") = Some((loot_table, seed));
            return;
        };

        let inventory: Arc<dyn Inventory> = self.clone();
        fill_chest_inventory(&inventory, table, seed).await;
    }
}

impl Inventory for MinecartInventory {
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
            std::mem::swap(&mut removed, &mut *self.items[slot].lock().await);
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

impl Clearable for MinecartInventory {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.items {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

struct MinecartScreenFactory {
    inventory: Arc<MinecartInventory>,
    title: TextComponent,
    hopper: bool,
}

impl ScreenHandlerFactory for MinecartScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let inventory: Arc<dyn Inventory> = self.inventory.clone();
            let handler = if self.hopper {
                create_hopper(sync_id, player_inventory, inventory).await
            } else {
                create_generic_9x3(sync_id, player_inventory, inventory).await
            };
            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        self.title.clone()
    }
}

pub(super) async fn open(
    entity: &Entity,
    player: &Arc<Player>,
    inventory: &Arc<MinecartInventory>,
    title: TextComponent,
    hopper: bool,
) -> bool {
    if player.is_spectator() && inventory.has_loot_table() {
        return false;
    }
    if !player.is_spectator() {
        inventory.unpack_loot().await;
    }

    player
        .open_handled_screen(
            &MinecartScreenFactory {
                inventory: inventory.clone(),
                title: entity.custom_name.load().as_ref().clone().unwrap_or(title),
                hopper,
            },
            None,
        )
        .await
        .is_some()
}

pub(super) async fn velocity(
    entity: &Entity,
    inventory: &MinecartInventory,
    velocity: Vector3<f64>,
) -> Vector3<f64> {
    let signal = crate::block::calculate_comparator_output(inventory).await;
    let mut friction = if inventory.has_loot_table() {
        0.98
    } else {
        0.98 + f64::from(15 - signal) * 0.001
    };
    if entity
        .touching_water
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        friction *= 0.95;
    }
    velocity.multiply(friction, 0.0, friction)
}

#[cfg(test)]
mod tests {
    use super::MinecartInventory;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_world::inventory::Inventory;

    #[tokio::test]
    async fn deferred_mineshaft_loot_is_preserved_until_unpacked() {
        let inventory = std::sync::Arc::new(MinecartInventory::new(27));
        let mut source = NbtCompound::new();
        source.put_string(
            "LootTable",
            "minecraft:chests/abandoned_mineshaft".to_string(),
        );
        source.put_long("LootTableSeed", 1234);
        inventory.read_nbt(&source);

        let mut deferred = NbtCompound::new();
        inventory.write_nbt(&mut deferred).await;
        assert_eq!(
            deferred.get_string("LootTable"),
            Some("minecraft:chests/abandoned_mineshaft")
        );
        assert_eq!(deferred.get_long("LootTableSeed"), Some(1234));
        assert!(deferred.get_list("Items").is_none());

        inventory.unpack_loot().await;
        assert!(!inventory.is_empty().await);

        let mut unpacked = NbtCompound::new();
        inventory.write_nbt(&mut unpacked).await;
        assert!(unpacked.get_string("LootTable").is_none());
        assert!(unpacked.get_list("Items").is_some());
    }

    #[tokio::test]
    async fn chest_minecart_items_round_trip_through_nbt() {
        let inventory = MinecartInventory::new(27);
        inventory
            .set_stack(8, ItemStack::new(3, &Item::POWERED_RAIL))
            .await;

        let mut nbt = NbtCompound::new();
        inventory.write_nbt(&mut nbt).await;

        let restored = MinecartInventory::new(27);
        restored.read_nbt(&nbt);
        let stack = restored.get_stack(8).await;
        let stack = stack.lock().await;
        assert_eq!(stack.get_item().id, Item::POWERED_RAIL.id);
        assert_eq!(stack.item_count, 3);
    }
}
