use std::{
    any::Any,
    sync::{Arc, Mutex, RwLock},
};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::{
    Clearable, Inventory,
    generic_container_screen_handler::create_generic_9x3,
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::text::TextComponent;

pub(super) struct BoatInventory {
    items: RwLock<Vec<ItemStack>>,
    size: usize,
}

impl BoatInventory {
    pub(super) fn new(size: usize) -> Self {
        Self {
            items: RwLock::new(vec![ItemStack::EMPTY.clone(); size]),
            size,
        }
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        if let Ok(mut items) = self.items.try_write() {
            items.fill_with(|| ItemStack::EMPTY.clone());
            self.read_data(nbt, &mut items);
        }
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(items) = self.items.try_read() {
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
}

impl Inventory for BoatInventory {
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

impl Clearable for BoatInventory {
    fn clear(&self) {
        self.items
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .fill_with(|| ItemStack::EMPTY.clone());
    }
}

pub(super) struct BoatScreenFactory {
    pub(super) inventory: Arc<BoatInventory>,
    pub(super) title: TextComponent,
}

impl ScreenHandlerFactory for BoatScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let inventory: Arc<dyn Inventory> = self.inventory.clone();
        let handler = create_generic_9x3(sync_id, player_inventory, inventory, player);
        Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        self.title.clone()
    }
}
