use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_protocol::server::play::SlotActionType;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    container_click::MouseClick,
    inventory::Inventory,
    player::player_inventory::PlayerInventory,
    slot::{NormalSlot, Slot},
    sync_handler::SyncHandler,
};

const SLOT_INDEX_OUTSIDE: i32 = -999;

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

#[async_trait]
pub trait InventoryPlayer: Send + Sync {
    async fn drop_item(&self, item: ItemStack, retain_ownership: bool);
}

//ScreenHandler.java
// TODO: Fully implement this
#[async_trait]
pub trait ScreenHandler: Send + Sync {
    /// Get the window type of the screen handler, otherwise panics
    fn window_type(&self) -> Option<WindowType>;

    fn as_any(&self) -> &dyn Any;

    fn sync_id(&self) -> u8;

    fn on_closed(&self, player: &dyn InventoryPlayer) {}

    fn can_use(&self, player: &dyn InventoryPlayer) -> bool {
        true
    }

    fn get_behaviour(&self) -> &DefaultScreenHandlerBehaviour;

    fn get_behaviour_mut(&mut self) -> &mut DefaultScreenHandlerBehaviour;

    fn add_slot(&mut self, slot: Arc<dyn Slot>) -> Arc<dyn Slot> {
        let behaviour = self.get_behaviour_mut();
        slot.set_id(behaviour.slots.len());
        behaviour.slots.push(slot.clone());
        behaviour.tracked_stacks.push(ItemStack::EMPTY);
        behaviour.previous_tracked_stacks.push(ItemStack::EMPTY);

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
        let behaviour = self.get_behaviour_mut();

        for i in 0..behaviour.slots.len() {
            behaviour.previous_tracked_stacks[i] = behaviour.slots[i].get_cloned_stack().await;
        }

        behaviour.previous_cursor_stack = *behaviour.cursor_stack.lock().await;

        for i in 0..behaviour.properties.len() {
            let property_val = behaviour.properties[i].get();
            behaviour.tracked_property_values[i] = property_val;
        }

        if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
            sync_handler
                .update_state(
                    behaviour,
                    &behaviour.previous_tracked_stacks,
                    &behaviour.previous_cursor_stack,
                    behaviour.tracked_property_values.clone(),
                )
                .await;
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
        self.sync_state().await;
    }

    fn add_property(&mut self, property: ScreenProperty) {
        let behaviour = self.get_behaviour_mut();
        behaviour.properties.push(property);
        behaviour.tracked_property_values.push(0);
    }

    fn add_properties(&mut self, properties: Vec<ScreenProperty>) {
        for property in properties {
            self.add_property(property);
        }
    }

    async fn update_to_client(&mut self) {
        for i in 0..self.get_behaviour().slots.len() {
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
            behaviour.tracked_stacks[slot] = stack;

            for listener in behaviour.listeners.iter() {
                listener.on_slot_update(behaviour, slot as u8, stack);
            }
        }
    }

    async fn check_slot_updates(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let prev_stack = behaviour.previous_tracked_stacks[slot];

            if prev_stack != stack {
                behaviour.previous_tracked_stacks[slot] = stack;
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler.update_slot(behaviour, slot, &stack).await;
                }
            }
        }
    }

    async fn check_cursor_stack_updates(&mut self) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let cursor_stack = behaviour.cursor_stack.lock().await;
            if !cursor_stack.are_equal(&behaviour.previous_cursor_stack) {
                behaviour.previous_cursor_stack = *cursor_stack;
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler
                        .update_cursor_stack(behaviour, &behaviour.previous_cursor_stack)
                        .await;
                }
            }
        }
    }

    async fn send_content_updates(&mut self) {
        let slots_len = self.get_behaviour().slots.len();

        for i in 0..slots_len {
            let slot = self.get_behaviour().slots[i].clone();
            //TODO: We might need to avoid using clone here
            let stack = slot.get_cloned_stack().await;

            self.update_tracked_slot(i, stack).await;
            self.check_slot_updates(i, stack).await;
        }

        self.check_cursor_stack_updates().await;

        /* TODO: Implement this
        for i in 0..self.prop_size() {
            let property = self.get_property(i);
            self.set_tracked_property(i, property);
        } */
    }

    async fn is_slot_valid(&self, slot: i32) -> bool {
        slot == -1 || slot == -999 || slot < self.get_behaviour().slots.len() as i32
    }

    async fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack;

    async fn handle_slot_click(
        &self,
        _player: &dyn InventoryPlayer,
        _click_type: MouseClick,
        _slot: Arc<dyn Slot>,
        _slot_stack: ItemStack,
        _cursor_stack: ItemStack,
    ) -> bool {
        // TODO: required for bundle in the future
        false
    }

    async fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player)
            .await;
    }

    async fn internal_on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        if (action_type == SlotActionType::Pickup || action_type == SlotActionType::QuickMove)
            && (button == 0 || button == 1)
        {
            let click_type = if button == 0 {
                MouseClick::Left
            } else {
                MouseClick::Right
            };

            // Drop item if outside inventory
            if slot_index == SLOT_INDEX_OUTSIDE {
                let mut cursor_stack = self.get_behaviour().cursor_stack.lock().await;
                if !cursor_stack.is_empty() {
                    if click_type == MouseClick::Left {
                        player.drop_item(*cursor_stack, true).await;
                        *cursor_stack = ItemStack::EMPTY;
                    } else {
                        player.drop_item(cursor_stack.split(1), true).await;
                    }
                }
            } else if action_type == SlotActionType::QuickMove {
                if slot_index < 0 {
                    return;
                }

                let slot = self.get_behaviour().slots[slot_index as usize].clone();

                if !slot.can_take_items(player).await {
                    return;
                }

                let mut moved_stack = self.quick_move(player, slot_index).await;

                while !moved_stack.is_empty()
                    && ItemStack::are_items_and_components_equal(
                        &slot.get_cloned_stack().await,
                        &moved_stack,
                    )
                {
                    moved_stack = self.quick_move(player, slot_index).await;
                }
            } else {
                if slot_index < 0 {
                    return;
                }

                let slot = self.get_behaviour().slots[slot_index as usize].clone();
                //TODO: Check if we can really use clone here
                let slot_stack = slot.get_cloned_stack().await;
                let mut cursor_stack = self.get_behaviour().cursor_stack.lock().await;

                if self
                    .handle_slot_click(
                        player,
                        click_type.clone(),
                        slot.clone(),
                        slot_stack,
                        *cursor_stack,
                    )
                    .await
                {
                    return;
                }

                if slot_stack.is_empty() {
                    if !cursor_stack.is_empty() {
                        println!("Cursor -> Slot");
                        let transfer_count = if click_type == MouseClick::Left {
                            cursor_stack.item_count
                        } else {
                            1
                        };
                        *cursor_stack =
                            slot.insert_stack_count(*cursor_stack, transfer_count).await;
                    }
                } else if slot.can_take_items(player).await {
                    if cursor_stack.is_empty() {
                        println!("Slot -> Cursor");
                        let take_count = if click_type == MouseClick::Left {
                            slot_stack.item_count
                        } else {
                            slot_stack.item_count.div_ceil(2)
                        };
                        let taken = slot.try_take_stack_range(take_count, u8::MAX, player).await;
                        if let Some(taken) = taken {
                            // Reverse order of operations, shouldn't affect anything
                            slot.on_take_item(&taken).await;
                            *cursor_stack = taken;
                        }
                    } else if slot.can_insert(&cursor_stack).await {
                        if ItemStack::are_items_and_components_equal(&slot_stack, &cursor_stack) {
                            let insert_count = if click_type == MouseClick::Left {
                                cursor_stack.item_count
                            } else {
                                1
                            };
                            *cursor_stack =
                                slot.insert_stack_count(*cursor_stack, insert_count).await;
                        } else if cursor_stack.item_count
                            <= slot.get_max_item_count_for_stack(&cursor_stack).await
                        {
                            let old_cursor_stack = *cursor_stack;
                            *cursor_stack = slot_stack;
                            slot.set_stack(old_cursor_stack).await;
                        }
                    } else if ItemStack::are_items_and_components_equal(&slot_stack, &cursor_stack)
                    {
                        let taken = slot
                            .try_take_stack_range(
                                slot_stack.item_count,
                                cursor_stack
                                    .get_max_stack_size()
                                    .saturating_sub(cursor_stack.item_count),
                                player,
                            )
                            .await;

                        if let Some(taken) = taken {
                            cursor_stack.increment(taken.item_count);
                            slot.on_take_item(&taken).await;
                        }
                    }
                }

                println!("Done");
                slot.mark_dirty().await;
            }
        }
    }

    async fn disable_sync(&mut self) {
        let behaviour = self.get_behaviour_mut();
        behaviour.disable_sync = true;
    }

    async fn enable_sync(&mut self) {
        let behaviour = self.get_behaviour_mut();
        behaviour.disable_sync = false;
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
    pub tracked_stacks: Vec<ItemStack>,
    pub cursor_stack: Arc<Mutex<ItemStack>>,
    pub previous_tracked_stacks: Vec<ItemStack>,
    pub previous_cursor_stack: ItemStack,
    pub revision: u32,
    pub disable_sync: bool,
    pub properties: Vec<ScreenProperty>,
    pub tracked_property_values: Vec<i32>,
    pub window_type: Option<WindowType>,
}

impl DefaultScreenHandlerBehaviour {
    pub fn new(sync_id: u8, window_type: Option<WindowType>) -> Self {
        Self {
            slots: Vec::new(),
            sync_id,
            listeners: Vec::new(),
            sync_handler: None,
            tracked_stacks: Vec::new(),
            cursor_stack: Arc::new(Mutex::new(ItemStack::EMPTY)),
            previous_tracked_stacks: Vec::new(),
            previous_cursor_stack: ItemStack::EMPTY,
            revision: 0,
            disable_sync: false,
            properties: Vec::new(),
            tracked_property_values: Vec::new(),
            window_type,
        }
    }
}
