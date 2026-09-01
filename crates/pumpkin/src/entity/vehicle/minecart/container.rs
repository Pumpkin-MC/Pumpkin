use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_hopper};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Clearable, Inventory};

use crate::entity::{Entity, player::Player};
use crate::world::loot::fill_chest_inventory;
use pumpkin_data::loot_table::get_loot_table;

pub(super) struct MinecartInventory {
    items: RwLock<Vec<ItemStack>>,
    size: usize,
    loot_table: Mutex<Option<(String, i64)>>,
    drops_claimed: AtomicBool,
}

impl MinecartInventory {
    pub(super) fn new(size: usize) -> Self {
        Self {
            items: RwLock::new(vec![ItemStack::EMPTY.clone(); size]),
            size,
            loot_table: Mutex::new(None),
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
        if let Ok(mut guard) = self.loot_table.try_lock() {
            *guard = loot_table;
        }

        if !has_loot_table && let Ok(mut items) = self.items.try_write() {
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.read_data(nbt, &mut items);
        }
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        let loot_table = self
            .loot_table
            .try_lock()
            .ok()
            .and_then(|guard| guard.clone());
        if let Some((loot_table, seed)) = loot_table {
            nbt.put_string("LootTable", loot_table);
            if seed != 0 {
                nbt.put_long("LootTableSeed", seed);
            }
        } else if let Ok(items) = self.items.try_read() {
            let mut list: Vec<pumpkin_nbt::tag::NbtTag> = Vec::new();
            for (slot, stack) in items.iter().enumerate() {
                if !stack.is_empty() {
                    let mut compound = NbtCompound::new();
                    compound.put_byte("Slot", slot as i8);
                    stack.write_item_stack(&mut compound);
                    list.push(pumpkin_nbt::tag::NbtTag::Compound(compound));
                }
            }
            nbt.put("Items", pumpkin_nbt::tag::NbtTag::List(list));
        }
    }

    pub(super) fn has_loot_table(&self) -> bool {
        self.loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    pub(super) fn unpack_loot(self: &Arc<Self>) {
        let loot_table = self
            .loot_table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        let Some((loot_table, seed)) = loot_table else {
            return;
        };
        let Some(table) = get_loot_table(&loot_table) else {
            *self
                .loot_table
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((loot_table, seed));
            return;
        };

        let inventory: Arc<dyn Inventory> = self.clone();
        fill_chest_inventory(&inventory, table, seed);
    }
}

impl Inventory for MinecartInventory {
    fn size(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> bool {
        let items = self
            .items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        items.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        self.items
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[slot]
            .clone()
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::replace(&mut items[slot], ItemStack::EMPTY.clone())
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut items = self
            .items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !items[slot].is_empty() && amount > 0 {
            items[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)[slot] = stack;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for MinecartInventory {
    fn clear(&self) {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .fill_with(|| ItemStack::EMPTY.clone());
    }
}

struct MinecartScreenFactory {
    inventory: Arc<MinecartInventory>,
    title: TextComponent,
    hopper: bool,
}

impl ScreenHandlerFactory for MinecartScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let inventory: Arc<dyn Inventory> = self.inventory.clone();
        let handler = if self.hopper {
            create_hopper(sync_id, player_inventory, inventory)
        } else {
            create_generic_9x3(sync_id, player_inventory, inventory)
        };
        Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        self.title.clone()
    }
}

pub(super) fn open(
    custom_name: Option<TextComponent>,
    player: &Arc<Player>,
    inventory: &Arc<MinecartInventory>,
    title: TextComponent,
    hopper: bool,
) -> bool {
    if player.is_spectator() && inventory.has_loot_table() {
        return false;
    }
    if !player.is_spectator() {
        inventory.unpack_loot();
    }

    player
        .open_handled_screen(
            &MinecartScreenFactory {
                inventory: inventory.clone(),
                title: custom_name.unwrap_or(title),
                hopper,
            },
            None,
        )
        .is_some()
}

pub(super) fn velocity(
    entity: &Entity,
    inventory: &MinecartInventory,
    velocity: Vector3<f64>,
) -> Vector3<f64> {
    let has_loot = inventory
        .loot_table
        .try_lock()
        .is_ok_and(|guard| guard.is_some());
    let signal = if has_loot {
        0
    } else if let Ok(items) = inventory.items.try_read() {
        let mut total_fill = 0.0;
        let mut has_items = false;
        for stack in items.iter() {
            if !stack.is_empty() {
                let max_count = stack.get_max_stack_size();
                total_fill += f64::from(stack.item_count) / f64::from(max_count);
                has_items = true;
            }
        }
        if has_items {
            let factor = total_fill / inventory.size as f64;
            (factor * 14.0).floor() as u8 + 1
        } else {
            0
        }
    } else {
        0
    };
    let mut friction = if has_loot {
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

    #[test]
    fn deferred_mineshaft_loot_is_preserved_until_unpacked() {
        let inventory = std::sync::Arc::new(MinecartInventory::new(27));
        let mut source = NbtCompound::new();
        source.put_string(
            "LootTable",
            "minecraft:chests/abandoned_mineshaft".to_string(),
        );
        source.put_long("LootTableSeed", 1234);
        inventory.read_nbt(&source);

        let mut deferred = NbtCompound::new();
        inventory.write_nbt(&mut deferred);
        assert_eq!(
            deferred.get_string("LootTable"),
            Some("minecraft:chests/abandoned_mineshaft")
        );
        assert_eq!(deferred.get_long("LootTableSeed"), Some(1234));
        assert!(deferred.get_list("Items").is_none());

        inventory.unpack_loot();
        assert!(!inventory.is_empty());

        let mut unpacked = NbtCompound::new();
        inventory.write_nbt(&mut unpacked);
        assert!(unpacked.get_string("LootTable").is_none());
        assert!(unpacked.get_list("Items").is_some());
    }

    #[test]
    fn chest_minecart_items_round_trip_through_nbt() {
        let inventory = MinecartInventory::new(27);
        inventory.set_stack(8, ItemStack::new(3, &Item::POWERED_RAIL));

        let mut nbt = NbtCompound::new();
        inventory.write_nbt(&mut nbt);

        let restored = MinecartInventory::new(27);
        restored.read_nbt(&nbt);
        let stack = restored.get_stack(8);
        assert_eq!(stack.get_item().id, Item::POWERED_RAIL.id);
        assert_eq!(stack.item_count, 3);
    }
}
