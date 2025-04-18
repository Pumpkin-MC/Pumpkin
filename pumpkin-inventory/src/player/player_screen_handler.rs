use std::{
    any::Any,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    crafting::{
        crafting_inventory::CraftingInventory,
        crafting_screen_handler::CraftingScreenHandler,
        recipies::{RecipeFinderScreenHandler, RecipeInputInventory},
    },
    equipment_slot::EquipmentSlot,
    inventory::Inventory,
    screen_handler::{ScreenHandler, ScreenHandlerListener, ScreenProperty},
    slot::{ArmorSlot, NormalSlot, Slot},
    sync_handler::{AlwaysInSyncTrackedSlot, SyncHandler, TrackedSlot},
};

use super::player_inventory::PlayerInventory;

pub struct PlayerScreenHandler {
    slots: Vec<Arc<dyn Slot>>,
    sync_id: AtomicU8,
    listeners: Vec<Arc<dyn ScreenHandlerListener>>,
    sync_handler: Option<Arc<SyncHandler>>,
    tracked_slots: Vec<Arc<dyn TrackedSlot>>,
    tracked_stacks: Vec<ItemStack>,
    cursor_stack: Arc<Mutex<ItemStack>>,
    tracked_cursor_slot: Arc<dyn TrackedSlot>,
    revision: u32,
    disable_sync: bool,
    properties: Vec<ScreenProperty>,
    tracked_properties: Vec<i32>,
}

impl RecipeFinderScreenHandler for PlayerScreenHandler {}

impl CraftingScreenHandler<CraftingInventory> for PlayerScreenHandler {}

// TODO: Fully implement this
impl PlayerScreenHandler {
    const EQUIPMENT_SLOT_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];

    fn is_in_hotbar(slot: u8) -> bool {
        slot >= 36 && slot < 45 || slot == 45
    }

    pub async fn new(
        player_inventory: &Arc<Mutex<PlayerInventory>>,
        window_type: Option<WindowType>,
        sync_id: u8,
    ) -> Self {
        let mut player_screen_handler = PlayerScreenHandler {
            slots: Vec::new(),
            sync_id: AtomicU8::new(sync_id),
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
        };
        let crafting_inventory: Arc<Mutex<dyn RecipeInputInventory>> =
            Arc::new(Mutex::new(CraftingInventory {
                width: 2,
                height: 2,
            }));

        player_screen_handler
            .add_result_slot(&crafting_inventory)
            .await;

        player_screen_handler
            .add_input_slots(&crafting_inventory)
            .await;

        for i in 0..4 {
            player_screen_handler.add_slot(Arc::new(ArmorSlot::new(
                player_inventory.clone(),
                39 - i,
                Self::EQUIPMENT_SLOT_ORDER[i].clone(),
            )));
        }

        let player_inventory: Arc<Mutex<dyn Inventory>> = player_inventory.clone();

        player_screen_handler.add_player_slots(&player_inventory);

        // Offhand
        // TODO: public void setStack(ItemStack stack, ItemStack previousStack) { owner.onEquipStack(EquipmentSlot.OFFHAND, previousStack, stack);
        player_screen_handler.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), 40)));

        player_screen_handler
    }
}

#[async_trait]
impl ScreenHandler for PlayerScreenHandler {
    fn window_type(&self) -> WindowType {
        unreachable!()
    }

    fn size(&self) -> usize {
        self.slots.len()
    }

    fn add_slot_internal(&mut self, slot: Arc<dyn Slot>) {
        self.slots.push(slot.clone());
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn sync_id(&self) -> u8 {
        self.sync_id.load(Ordering::Relaxed)
    }

    async fn add_listener(&mut self, listener: Arc<dyn ScreenHandlerListener>) {
        self.listeners.push(listener);
        //TODO: self.send_content_updates();
    }

    async fn update_sync_handler(&mut self, sync_handler: Arc<SyncHandler>) {
        self.sync_handler = Some(sync_handler);
        self.tracked_cursor_slot = self.sync_handler.as_ref().unwrap().create_tracked_slot();
        self.tracked_slots.iter_mut().for_each(|slot| {
            *slot = self.sync_handler.as_ref().unwrap().create_tracked_slot();
        });
        self.sync_state().await;
    }

    async fn update_to_client(&mut self) {
        for i in 0..self.size() {
            let slot = self.get_slot(i);
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
        let other_stack = &self.tracked_stacks[slot];
        if other_stack != &stack {
            self.tracked_stacks[slot] = stack.clone();

            for listener in self.listeners.iter() {
                listener.on_slot_update(self, slot as u8, stack.clone());
            }
        }
    }

    async fn check_slot_updates(&mut self, slot: usize, stack: ItemStack) {
        if !self.disable_sync {
            let tracked_slot = self.get_tracked_slot(slot);
            if !tracked_slot.is_in_sync(stack.clone()).await {
                if let Some(sync_handler) = self.get_sync_handler() {
                    sync_handler.update_slot(self, slot, &stack);
                }
            }
        }
    }

    async fn check_cursor_stack_updates(&mut self) {
        if !self.disable_sync {
            let cursor_stack = self.get_cursor_stack();
            if !self
                .get_tracked_cursor_slot()
                .is_in_sync(cursor_stack.lock().await.clone())
                .await
            {
                self.tracked_cursor_slot
                    .set_recived_stack(cursor_stack.lock().await.clone())
                    .await;
                if let Some(sync_handler) = self.get_sync_handler() {
                    sync_handler.update_cursor_stack(self, &cursor_stack.lock().await.clone());
                }
            }
        }
    }

    async fn send_content_updates(&mut self) {
        for i in 0..self.size() {
            let slot = self.get_slot(i);
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

    fn get_sync_handler(&self) -> Option<Arc<SyncHandler>> {
        self.sync_handler.clone()
    }

    fn add_tracked_slot_internal(&mut self, slot: Arc<dyn TrackedSlot>) {
        self.tracked_slots.push(slot);
    }

    fn add_tracked_stack_internal(&mut self, stack: ItemStack) {
        self.tracked_stacks.push(stack);
    }

    fn get_slot(&self, index: usize) -> Arc<dyn Slot> {
        self.slots[index].clone()
    }

    fn get_tracked_slot(&self, index: usize) -> Arc<dyn TrackedSlot> {
        self.tracked_slots[index].clone()
    }

    fn get_cursor_stack(&self) -> Arc<Mutex<ItemStack>> {
        self.cursor_stack.clone()
    }

    fn get_tracked_cursor_slot(&self) -> Arc<dyn TrackedSlot> {
        self.tracked_cursor_slot.clone()
    }

    fn add_property(&mut self, property: ScreenProperty) {
        self.properties.push(property);
        self.tracked_properties.push(0);
    }

    fn prop_size(&self) -> usize {
        self.properties.len()
    }

    fn get_property(&self, index: usize) -> i32 {
        self.properties[index].get()
    }

    fn set_tracked_property(&mut self, index: usize, value: i32) {
        self.tracked_properties[index] = value;
    }

    fn get_tracked_properties(&self) -> Vec<i32> {
        self.tracked_properties.clone()
    }
}
