use std::any::Any;
use std::sync::Arc;
use tokio::sync::Mutex;

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use pumpkin_inventory::slot::NormalSlot;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Clearable, Inventory, InventoryFuture};

pub struct PluginGui {
    pub window_type: WindowType,
    pub title: TextComponent,
    pub inventory: Arc<PluginInventory>,
    pub allow_grab_items: bool,
    pub allow_put_items: bool,
}

pub struct PluginInventory {
    pub slots: Vec<Arc<Mutex<ItemStack>>>,
}

impl PluginInventory {
    #[must_use]
    pub fn new(size: usize) -> Self {
        let mut slots = Vec::with_capacity(size);
        for _ in 0..size {
            slots.push(Arc::new(Mutex::new(ItemStack::EMPTY.clone())));
        }
        Self { slots }
    }
}

impl Clearable for PluginInventory {
    fn clear(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            for slot in &self.slots {
                *slot.lock().await = ItemStack::EMPTY.clone();
            }
        })
    }
}

impl Inventory for PluginInventory {
    fn size(&self) -> usize {
        self.slots.len()
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move {
            for slot in &self.slots {
                if !slot.lock().await.is_empty() {
                    return false;
                }
            }
            true
        })
    }

    fn get_stack(&self, slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.slots[slot].clone() })
    }

    fn remove_stack(&self, slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut stack = self.slots[slot].lock().await;
            std::mem::replace(&mut *stack, ItemStack::EMPTY.clone())
        })
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            let mut stack = self.slots[slot].lock().await;
            stack.split(amount)
        })
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.slots[slot].lock().await = stack;
        })
    }

    fn on_open(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {})
    }

    fn on_close(&self) -> InventoryFuture<'_, ()> {
        Box::pin(async move {})
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct PluginScreenHandler {
    pub inventory: Arc<PluginInventory>,
    behaviour: ScreenHandlerBehaviour,
}

impl PluginScreenHandler {
    #[must_use]
    pub fn new(
        sync_id: u8,
        window_type: WindowType,
        inventory: &Arc<PluginInventory>,
        allow_grab_items: bool,
        allow_put_items: bool,
    ) -> Self {
        let mut behaviour = ScreenHandlerBehaviour::new(sync_id, Some(window_type));
        behaviour.allow_grab_items = allow_grab_items;
        behaviour.allow_put_items = allow_put_items;
        behaviour.container_slots = inventory.size();

        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour,
        };

        for i in 0..inventory.size() {
            handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), i)));
        }

        handler
    }
}

impl ScreenHandler for PluginScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn quick_move<'a>(
        &'a mut self,
        _player: &'a dyn InventoryPlayer,
        _slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move { ItemStack::EMPTY.clone() })
    }
}
