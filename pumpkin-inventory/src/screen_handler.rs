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

pub struct ScreenProperty {
    old_value: i32,
    index: u8,
    value: i32,
}

impl ScreenProperty {
    pub fn get(&self) -> i32 {
        self.value
    }

    pub fn set(&mut self, value: i32) {
        self.value = value;
    }
}

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

    fn get_behaviour(&self) -> &DefaultScreenHandlerBehaviour;

    fn get_behaviour_mut(&mut self) -> &mut DefaultScreenHandlerBehaviour;

    fn add_slot(&mut self, slot: Arc<dyn Slot>) -> Arc<dyn Slot> {
        slot.set_id(self.size());
        let behaviour = self.get_behaviour_mut();
        behaviour.slots.push(slot.clone());
        let sync_handler = behaviour.sync_handler.as_ref();
        if let Some(sync_handler) = sync_handler {
            behaviour
                .tracked_slots
                .push(sync_handler.create_tracked_slot());
        } else {
            behaviour.tracked_slots.push(Arc::new(
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

    async fn sync_state(&mut self) {
        let mut item_stacks: Vec<ItemStack> = Vec::with_capacity(self.size());

        for i in 0..self.size() {
            let behaviour = self.get_behaviour_mut();
            //TODO: Look into the cloning of itemstacks
            let slot = behaviour.slots[i].clone();
            let mut inv = slot.get_inventory().lock().await;
            let item_stack = inv.get_stack(i);
            item_stacks.push(item_stack.clone());
            let tracked_slot = behaviour.tracked_slots[i].clone();
            tracked_slot.set_recived_stack(item_stack.clone()).await;
        }

        let behaviour = self.get_behaviour_mut();

        let cursor_stack = behaviour.cursor_stack.clone();
        behaviour
            .tracked_cursor_slot
            .set_recived_stack(cursor_stack.lock().await.clone())
            .await;

        for i in 0..behaviour.properties.len() {
            let property_val = behaviour.properties[i].get();
            behaviour.tracked_properties.push(property_val);
        }

        if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
            let sync_handler = sync_handler.clone();
            let tracked_properties = behaviour.tracked_properties.clone();
            sync_handler.update_state(
                &behaviour,
                &item_stacks,
                &cursor_stack.lock().await.clone(),
                tracked_properties,
            );
        }
    }

    async fn add_listener(&mut self, listener: Arc<dyn ScreenHandlerListener>) {
        let behaviour = self.get_behaviour_mut();
        behaviour.listeners.push(listener);
        self.send_content_updates().await;
    }

    async fn update_sync_handler(&mut self, sync_handler: Arc<SyncHandler>) {
        let behaviour = self.get_behaviour_mut();
        behaviour.sync_handler = Some(sync_handler.clone());
        behaviour.tracked_cursor_slot = sync_handler.create_tracked_slot();
        behaviour.tracked_slots.iter_mut().for_each(|slot| {
            *slot = sync_handler.create_tracked_slot();
        });
        self.sync_state().await;
    }

    fn add_property(&mut self, property: ScreenProperty) {
        let behaviour = self.get_behaviour_mut();
        behaviour.properties.push(property);
        behaviour.tracked_properties.push(0);
    }

    fn add_properties(&mut self, properties: Vec<ScreenProperty>) {
        for property in properties {
            self.add_property(property);
        }
    }

    async fn update_to_client(&mut self) {
        for i in 0..self.size() {
            let behaviour = self.get_behaviour_mut();
            let slot = behaviour.slots[i].clone();
            let stack = slot.get_cloned_stack().await;
            self.update_tracked_slot(i, stack).await;
        }

        /* TODO: Implement this
        for i in 0..self.prop_size() {
            let property = self.get_property(i);
            self.set_tracked_property(i, property);
        } */

        self.sync_state().await;
    }

    async fn update_tracked_slot(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        let other_stack = &behaviour.tracked_stacks[slot];
        if other_stack != &stack {
            behaviour.tracked_stacks[slot] = stack.clone();

            for listener in behaviour.listeners.iter() {
                listener.on_slot_update(&behaviour, slot as u8, stack.clone());
            }
        }
    }

    async fn check_slot_updates(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let tracked_slot = behaviour.tracked_slots[slot].clone();
            if !tracked_slot.is_in_sync(stack.clone()).await {
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler.update_slot(&behaviour, slot, &stack);
                }
            }
        }
    }

    async fn check_cursor_stack_updates(&mut self) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let cursor_stack = behaviour.cursor_stack.clone();
            if !behaviour
                .tracked_cursor_slot
                .is_in_sync(cursor_stack.lock().await.clone())
                .await
            {
                behaviour
                    .tracked_cursor_slot
                    .set_recived_stack(cursor_stack.lock().await.clone())
                    .await;
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler
                        .update_cursor_stack(&behaviour, &cursor_stack.lock().await.clone());
                }
            }
        }
    }

    async fn send_content_updates(&mut self) {
        let slots_len = self.get_behaviour().slots.len();

        for i in 0..slots_len {
            let slot = self.get_behaviour().slots[i].clone();
            let stack = slot.get_cloned_stack().await;

            self.update_tracked_slot(i, stack.clone()).await;
            self.check_slot_updates(i, stack).await;
        }

        self.check_cursor_stack_updates().await;

        /* TODO: Implement this
        for i in 0..self.prop_size() {
            let property = self.get_property(i);
            self.set_tracked_property(i, property);
        } */
    }
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
    fn on_slot_update(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        slot: u8,
        stack: ItemStack,
    ) {
    }
    fn on_property_update(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        property: u8,
        value: i32,
    ) {
    }
}

pub struct DefaultScreenHandlerBehaviour {
    pub slots: Vec<Arc<dyn Slot>>,
    pub sync_id: u8,
    pub listeners: Vec<Arc<dyn ScreenHandlerListener>>,
    pub sync_handler: Option<Arc<SyncHandler>>,
    pub tracked_slots: Vec<Arc<dyn TrackedSlot>>,
    pub tracked_stacks: Vec<ItemStack>,
    pub cursor_stack: Arc<Mutex<ItemStack>>,
    pub tracked_cursor_slot: Arc<dyn TrackedSlot>,
    pub revision: u32,
    pub disable_sync: bool,
    pub properties: Vec<ScreenProperty>,
    pub tracked_properties: Vec<i32>,
    pub window_type: Option<WindowType>,
}

impl DefaultScreenHandlerBehaviour {
    pub fn new(sync_id: u8, window_type: Option<WindowType>) -> Self {
        Self {
            slots: Vec::new(),
            sync_id,
            listeners: Vec::new(),
            sync_handler: None,
            tracked_slots: Vec::new(),
            tracked_stacks: Vec::new(),
            cursor_stack: Arc::new(Mutex::new(ItemStack::EMPTY)),
            tracked_cursor_slot: Arc::new(AlwaysInSyncTrackedSlot::ALWAYS_IN_SYNC_TRACKED_SLOT),
            revision: 0,
            disable_sync: false,
            properties: Vec::new(),
            tracked_properties: Vec::new(),
            window_type,
        }
    }
}
