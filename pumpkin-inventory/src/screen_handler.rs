use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_protocol::server::play::SlotActionType;
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    container_click::{ClickType, MouseClick},
    inventory::Inventory,
    player::player_inventory::PlayerInventory,
    slot::{NormalSlot, Slot},
    sync_handler::{AlwaysInSyncTrackedSlot, SyncHandler, TrackedSlot},
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

    async fn is_slot_valid(&self, slot: i32) -> bool {
        slot == -1 || slot == -999 || slot < self.size() as i32
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
                        player.drop_item(cursor_stack.clone(), true).await;
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
                let mut inventory = slot.get_inventory().lock().await;
                let slot_stack = inventory.get_stack(slot_index as usize);
                let mut cursor_stack = self.get_behaviour().cursor_stack.lock().await;

                if self
                    .handle_slot_click(
                        player,
                        click_type.clone(),
                        slot.clone(),
                        slot_stack.clone(),
                        cursor_stack.clone(),
                    )
                    .await
                {
                    return;
                }

                if slot_stack.is_empty() {
                    if !cursor_stack.is_empty() {
                        let transfer_count = if click_type == MouseClick::Left {
                            cursor_stack.item_count
                        } else {
                            1
                        };
                        *cursor_stack = slot
                            .insert_stack_count(cursor_stack.clone(), transfer_count)
                            .await;
                    }
                } else if slot.can_take_items(player).await {
                    if cursor_stack.is_empty() {
                        let take_count = if click_type == MouseClick::Left {
                            slot_stack.item_count
                        } else {
                            (slot_stack.item_count + 1) / 2
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
                            *cursor_stack = slot
                                .insert_stack_count(cursor_stack.clone(), insert_count)
                                .await;
                        } else if cursor_stack.item_count
                            <= slot.get_max_item_count_for_stack(&cursor_stack).await
                        {
                            let old_cursor_stack = cursor_stack.clone();
                            *cursor_stack = slot_stack.clone();
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

                slot.mark_dirty().await;
            }

            /*
                        {
                ClickType clickType = button == 0 ? ClickType.LEFT : ClickType.RIGHT;

                if (slotIndex == EMPTY_SPACE_SLOT_INDEX) {
                    ItemStack cursorStack = this.getCursorStack();
                    if (!cursorStack.isEmpty()) {
                        if (clickType == ClickType.LEFT) {
                            player.dropItem(cursorStack, true);
                            this.setCursorStack(ItemStack.EMPTY);
                        } else {
                            player.dropItem(cursorStack.split(1), true);
                        }
                    }
                } else if (actionType == SlotActionType.QUICK_MOVE) {
                    if (slotIndex < 0) {
                        return;
                    }

                    Slot slot = this.slots.get(slotIndex);
                    if (!slot.canTakeItems(player)) {
                        return;
                    }

                    ItemStack movedStack = this.quickMove(player, slotIndex);

                    while (!movedStack.isEmpty() && ItemStack.areItemsEqual(slot.getStack(), movedStack)) {
                        movedStack = this.quickMove(player, slotIndex);
                    }
                } else {
                    if (slotIndex < 0) {
                        return;
                    }

                    Slot slot = this.slots.get(slotIndex);
                    ItemStack slotStack = slot.getStack();
                    ItemStack cursorStack = this.getCursorStack();

                    player.onPickupSlotClick(cursorStack, slotStack, clickType);

                    if (!this.handleSlotClick(player, clickType, slot, slotStack, cursorStack)) {
                        if (slotStack.isEmpty()) {
                            if (!cursorStack.isEmpty()) {
                                int transferCount = clickType == ClickType.LEFT ? cursorStack.getCount() : 1;
                                this.setCursorStack(slot.insertStack(cursorStack, transferCount));
                            }
                        } else if (slot.canTakeItems(player)) {
                            if (cursorStack.isEmpty()) {
                                int takeCount = clickType == ClickType.LEFT ? slotStack.getCount() : (slotStack.getCount() + 1) / 2;
                                Optional<ItemStack> taken = slot.tryTakeStackRange(takeCount, Integer.MAX_VALUE, player);
                                taken.ifPresent(stack -> {
                                    this.setCursorStack(stack);
                                    slot.onTakeItem(player, stack);
                                });
                            } else if (slot.canInsert(cursorStack)) {
                                if (ItemStack.areItemsAndComponentsEqual(slotStack, cursorStack)) {
                                    int insertCount = clickType == ClickType.LEFT ? cursorStack.getCount() : 1;
                                    this.setCursorStack(slot.insertStack(cursorStack, insertCount));
                                } else if (cursorStack.getCount() <= slot.getMaxItemCount(cursorStack)) {
                                    this.setCursorStack(slotStack);
                                    slot.setStack(cursorStack);
                                }
                            } else if (ItemStack.areItemsAndComponentsEqual(slotStack, cursorStack)) {
                                Optional<ItemStack> taken = slot.tryTakeStackRange(slotStack.getCount(), cursorStack.getMaxCount() - cursorStack.getCount(), player);
                                taken.ifPresent(stack -> {
                                    cursorStack.increment(stack.getCount());
                                    slot.onTakeItem(player, stack);
                                });
                            }
                        }
                    }

                    slot.markDirty();
                }
            } */
        }
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
