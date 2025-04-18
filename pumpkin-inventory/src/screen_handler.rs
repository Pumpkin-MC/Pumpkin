use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    inventory::Inventory,
    player::player_inventory::PlayerInventory,
    slot::{NormalSlot, Slot},
    sync_handler::{AlwaysInSyncTrackedSlot, SyncHandler, TrackedSlot},
};

pub trait InventoryPlayer {}

//ScreenHandler.java
// TODO: Fully implement this
#[async_trait]
pub trait ScreenHandler: Send + Sync {
    /// Get the window type of the screen handler, otherwise panics
    fn window_type(&self) -> WindowType;

    fn size(&self) -> usize;

    fn as_any(&self) -> &dyn Any;

    fn sync_id(&self) -> u8;

    fn on_closed(&self, player: &dyn InventoryPlayer) {}

    /// Add it into array
    fn add_slot_internal(&mut self, slot: Arc<dyn Slot>);

    fn add_tracked_slot_internal(&mut self, slot: Arc<dyn TrackedSlot>);

    fn add_tracked_stack_internal(&mut self, stack: ItemStack);

    fn get_sync_handler(&self) -> Option<Arc<SyncHandler>>;

    fn add_slot(&mut self, slot: Arc<dyn Slot>) -> Arc<dyn Slot> {
        slot.set_id(self.size());
        self.add_slot_internal(slot.clone());
        self.add_tracked_stack_internal(ItemStack::EMPTY);
        let sync_handler = self.get_sync_handler();
        if let Some(sync_handler) = sync_handler {
            self.add_tracked_slot_internal(sync_handler.create_tracked_slot());
        } else {
            self.add_tracked_slot_internal(Arc::new(
                AlwaysInSyncTrackedSlot::ALWAYS_IN_SYNC_TRACKED_SLOT,
            ));
        }
        slot
    }

    fn add_player_hotbar_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        for i in 0..9 {
            self.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), i)));
        }
    }

    fn add_player_inventory_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        for i in 0..3 {
            for j in 0..9 {
                self.add_slot(Arc::new(NormalSlot::new(
                    player_inventory.clone(),
                    j + (i + 1) * 9,
                )));
            }
        }
    }

    fn add_player_slots(&mut self, player_inventory: &Arc<Mutex<dyn Inventory>>) {
        self.add_player_inventory_slots(player_inventory);
        self.add_player_hotbar_slots(player_inventory);
    }

    async fn copy_shared_slots(&self, other: Arc<Mutex<dyn ScreenHandler>>) {
        let table: HashMap<&dyn Inventory, HashMap<i32, i32>> = HashMap::new();

        todo!()
    }

    fn get_slot(&self, index: usize) -> Arc<dyn Slot>;

    fn get_tracked_slot(&self, index: usize) -> Arc<dyn TrackedSlot>;

    fn get_cursor_stack(&self) -> Arc<Mutex<ItemStack>>;

    fn get_tracked_cursor_slot(&self) -> Arc<dyn TrackedSlot>;

    async fn sync_state(&mut self) {
        let mut item_stacks: Vec<ItemStack> = Vec::with_capacity(self.size());

        for i in 0..self.size() {
            //TODO: Look into the cloning of itemstacks
            let slot = self.get_slot(i);
            let mut inv = slot.get_inventory().lock().await;
            let item_stack = inv.get_stack(i);
            item_stacks.push(item_stack.clone());
            let tracked_slot = self.get_tracked_slot(i);
            tracked_slot
                .set_recived_stack(Arc::new(Mutex::new(item_stack.clone())))
                .await;
        }

        let cursor_stack = self.get_cursor_stack();
        self.get_tracked_cursor_slot()
            .set_recived_stack(cursor_stack)
            .await;
    }

    async fn add_listener(&mut self, listener: Arc<dyn ScreenHandlerListener>);

    async fn update_sync_handler(&mut self, sync_handler: Arc<SyncHandler>);
}

pub trait ScreenHandlerFactory: Send + Sync {
    fn crate_menu(
        &self,
        sync_id: u8,
        player_inventory: Arc<Mutex<PlayerInventory>>,
        player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>>;

    fn get_display_name(&self) -> TextComponent;
}

pub trait ScreenHandlerListener: Send + Sync {
    fn on_slot_update(&self, screen_handler: &dyn ScreenHandler, slot: u8, stack: ItemStack) {}
    fn on_property_update(&self, screen_handler: &dyn ScreenHandler, property: u8, value: i32) {}
}
