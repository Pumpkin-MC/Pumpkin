use std::{any::Any, collections::HashMap, sync::Arc};

use async_trait::async_trait;
use log::warn;
use pumpkin_data::screen::WindowType;
use pumpkin_protocol::{
    client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory,
    },
    codec::item_stack_seralizer::OptionalItemStackHash,
    server::play::SlotActionType,
};
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{ComparableInventory, Inventory};
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    container_click::MouseClick,
    player::player_inventory::PlayerInventory,
    slot::{NormalSlot, Slot},
    sync_handler::{SyncHandler, TrackedStack},
};

const SLOT_INDEX_OUTSIDE: i32 = -999;

pub struct ScreenProperty {
    _old_value: i32,
    _index: u8,
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
    fn get_inventory(&self) -> Arc<PlayerInventory>;
    async fn enqueue_inventory_packet(&self, packet: &CSetContainerContent);
    async fn enqueue_slot_packet(&self, packet: &CSetContainerSlot);
    async fn enqueue_cursor_packet(&self, packet: &CSetCursorItem);
    async fn enqueue_property_packet(&self, packet: &CSetContainerProperty);
    async fn enqueue_slot_set_packet(&self, packet: &CSetPlayerInventory);
}

pub async fn offer_or_drop_stack(player: &dyn InventoryPlayer, stack: ItemStack) {
    // TODO: Super weird disconnect logic in vanilla, investigate this later
    player
        .get_inventory()
        .offer_or_drop_stack(stack, player)
        .await;
}

//ScreenHandler.java
// TODO: Fully implement this
#[async_trait]
pub trait ScreenHandler: Send + Sync {
    /// Get the window type of the screen handler, otherwise panics
    fn window_type(&self) -> Option<WindowType> {
        self.get_behaviour().window_type
    }

    fn as_any(&self) -> &dyn Any;

    fn sync_id(&self) -> u8 {
        self.get_behaviour().sync_id
    }

    async fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player).await;
    }

    async fn default_on_closed(&mut self, player: &dyn InventoryPlayer) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.cursor_stack.lock().await.is_empty() {
            offer_or_drop_stack(player, *behaviour.cursor_stack.lock().await).await;
            *behaviour.cursor_stack.lock().await = ItemStack::EMPTY;
        }
    }

    fn can_use(&self, _player: &dyn InventoryPlayer) -> bool {
        true
    }

    async fn drop_inventory(&self, player: &dyn InventoryPlayer, inventory: Arc<dyn Inventory>) {
        for i in 0..inventory.size() {
            offer_or_drop_stack(player, inventory.remove_stack(i).await).await;
        }
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour;

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour;

    fn add_slot(&mut self, slot: Arc<dyn Slot>) -> Arc<dyn Slot> {
        let behaviour = self.get_behaviour_mut();
        slot.set_id(behaviour.slots.len());
        behaviour.slots.push(slot.clone());
        behaviour.tracked_stacks.push(ItemStack::EMPTY);
        behaviour.previous_tracked_stacks.push(TrackedStack::EMPTY);

        slot
    }

    fn add_player_hotbar_slots(&mut self, player_inventory: &Arc<dyn Inventory>) {
        for i in 0..9 {
            self.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), i)));
        }
    }

    fn add_player_inventory_slots(&mut self, player_inventory: &Arc<dyn Inventory>) {
        for i in 0..3 {
            for j in 0..9 {
                self.add_slot(Arc::new(NormalSlot::new(
                    player_inventory.clone(),
                    j + (i + 1) * 9,
                )));
            }
        }
    }

    fn add_player_slots(&mut self, player_inventory: &Arc<dyn Inventory>) {
        self.add_player_inventory_slots(player_inventory);
        self.add_player_hotbar_slots(player_inventory);
    }

    async fn copy_shared_slots(&mut self, other: Arc<Mutex<dyn ScreenHandler>>) {
        let mut table: HashMap<ComparableInventory, HashMap<usize, usize>> = HashMap::new();
        let other_binding = other.lock().await;
        let other_behaviour = other_binding.get_behaviour();

        for i in 0..other_behaviour.slots.len() {
            let other_slot = other_behaviour.slots[i].clone();
            let mut hash_map = HashMap::new();
            hash_map.insert(other_slot.get_index(), i);
            table.insert(
                ComparableInventory(other_slot.get_inventory().clone()),
                hash_map,
            );
        }

        for i in 0..self.get_behaviour().slots.len() {
            let slot = self.get_behaviour().slots[i].clone();
            let inventory = slot.get_inventory();
            let index = slot.get_index();

            if let Some(hash_map) = table.get(&ComparableInventory(inventory.clone())) {
                if let Some(other_index) = hash_map.get(&index) {
                    self.get_behaviour_mut().tracked_stacks[i] =
                        other_behaviour.tracked_stacks[*other_index];
                    self.get_behaviour_mut().previous_tracked_stacks[i] =
                        other_behaviour.previous_tracked_stacks[*other_index].clone();
                }
            }
        }
    }

    async fn set_received_hash(&mut self, slot: usize, hash: OptionalItemStackHash) {
        let behaviour = self.get_behaviour_mut();
        if slot < behaviour.previous_tracked_stacks.len() {
            behaviour.previous_tracked_stacks[slot].set_received_hash(hash);
        } else {
            warn!(
                "Incorrect slot index: {} available slots: {}",
                slot,
                behaviour.previous_tracked_stacks.len()
            );
        }
    }

    async fn set_received_stack(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        behaviour.previous_tracked_stacks[slot].set_received_stack(stack);
    }

    async fn set_received_cursor_hash(&mut self, hash: OptionalItemStackHash) {
        let behaviour = self.get_behaviour_mut();
        behaviour.previous_cursor_stack.set_received_hash(hash);
    }

    async fn sync_state(&mut self) {
        let behaviour = self.get_behaviour_mut();
        let mut previous_tracked_stacks = Vec::new();

        for i in 0..behaviour.slots.len() {
            let stack = behaviour.slots[i].get_cloned_stack().await;
            previous_tracked_stacks.push(stack);
            behaviour.previous_tracked_stacks[i].set_received_stack(stack);
        }

        let cursor_stack = *behaviour.cursor_stack.lock().await;
        behaviour
            .previous_cursor_stack
            .set_received_stack(cursor_stack);

        for i in 0..behaviour.properties.len() {
            let property_val = behaviour.properties[i].get();
            behaviour.tracked_property_values[i] = property_val;
        }

        let next_revision = behaviour.next_revision();

        if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
            sync_handler
                .update_state(
                    behaviour,
                    &previous_tracked_stacks,
                    &cursor_stack,
                    behaviour.tracked_property_values.clone(),
                    next_revision,
                )
                .await;
        }
    }

    async fn add_listener(&mut self, listener: Arc<dyn ScreenHandlerListener>) {
        self.get_behaviour_mut().listeners.push(listener);
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
        if !other_stack.are_equal(&stack) {
            behaviour.tracked_stacks[slot] = stack;

            for listener in behaviour.listeners.iter() {
                listener.on_slot_update(behaviour, slot as u8, stack);
            }
        }
    }

    async fn check_slot_updates(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let prev_stack = &mut behaviour.previous_tracked_stacks[slot];

            if !prev_stack.is_in_sync(&stack) {
                prev_stack.set_received_stack(stack);
                let next_revision = behaviour.next_revision();
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler
                        .update_slot(behaviour, slot, &stack, next_revision)
                        .await;
                }
            }
        }
    }

    async fn check_cursor_stack_updates(&mut self) {
        let behaviour = self.get_behaviour_mut();
        if !behaviour.disable_sync {
            let cursor_stack = behaviour.cursor_stack.lock().await;
            if !behaviour.previous_cursor_stack.is_in_sync(&cursor_stack) {
                behaviour
                    .previous_cursor_stack
                    .set_received_stack(*cursor_stack);
                if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                    sync_handler
                        .update_cursor_stack(behaviour, &cursor_stack)
                        .await;
                }
            }
        }
    }

    async fn send_content_updates(&mut self) {
        let slots_len = self.get_behaviour().slots.len();

        for i in 0..slots_len {
            let slot = self.get_behaviour().slots[i].clone();
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

    async fn get_slot_index(&self, inventory: &Arc<dyn Inventory>, slot: usize) -> Option<usize> {
        for i in 0..self.get_behaviour().slots.len() {
            if Arc::ptr_eq(self.get_behaviour().slots[i].get_inventory(), inventory)
                && self.get_behaviour().slots[i].get_index() == slot
            {
                return Some(i);
            }
        }

        None
    }

    async fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack;

    async fn insert_item(
        &mut self,
        stack: &mut ItemStack,
        start_index: i32,
        end_index: i32,
        from_last: bool,
    ) -> bool {
        let mut success = false;
        let mut current_index = if from_last {
            end_index - 1
        } else {
            start_index
        };

        if stack.is_stackable() {
            while !stack.is_empty()
                && (if from_last {
                    current_index >= start_index
                } else {
                    current_index < end_index
                })
            {
                let slot = self.get_behaviour().slots[current_index as usize].clone();
                let slot_stack = slot.get_stack().await;
                let mut slot_stack = slot_stack.lock().await;

                if !slot_stack.is_empty() && slot_stack.are_items_and_components_equal(stack) {
                    let combined_count = slot_stack.item_count + stack.item_count;
                    let max_slot_count = slot.get_max_item_count_for_stack(&slot_stack).await;
                    if combined_count <= max_slot_count {
                        stack.set_count(0);
                        slot_stack.set_count(combined_count);
                        drop(slot_stack);
                        slot.mark_dirty().await;
                        success = true;
                    } else if slot_stack.item_count < max_slot_count {
                        stack.decrement(max_slot_count - slot_stack.item_count);
                        slot_stack.set_count(max_slot_count);
                        drop(slot_stack);
                        slot.mark_dirty().await;
                        success = true;
                    }
                }

                if from_last {
                    current_index -= 1;
                } else {
                    current_index += 1;
                }
            }
        }

        if !stack.is_empty() {
            if from_last {
                current_index = end_index - 1;
            } else {
                current_index = start_index;
            }

            while if from_last {
                current_index >= start_index
            } else {
                current_index < end_index
            } {
                let slot = self.get_behaviour().slots[current_index as usize].clone();
                let slot_stack = slot.get_stack().await;
                let slot_stack = slot_stack.lock().await;

                if slot_stack.is_empty() && slot.can_insert(stack).await {
                    let max_count = slot.get_max_item_count_for_stack(stack).await;
                    drop(slot_stack);
                    slot.set_stack(stack.split(max_count.min(stack.item_count)))
                        .await;
                    slot.mark_dirty().await;
                    success = true;
                    break;
                }

                if from_last {
                    current_index -= 1;
                } else {
                    current_index += 1;
                }
            }
        }

        success
    }

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
        //TODO: Implement quickcraft, Clone, PickupAll, Throw
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
                // Pickup
                if slot_index < 0 {
                    return;
                }

                let slot = self.get_behaviour().slots[slot_index as usize].clone();

                if click_type == MouseClick::Left {
                    slot.on_click(player).await;
                }

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
                        //println!("Cursor -> Slot");
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
                        //println!("Slot -> Cursor");
                        let take_count = if click_type == MouseClick::Left {
                            slot_stack.item_count
                        } else {
                            slot_stack.item_count.div_ceil(2)
                        };
                        let taken = slot.try_take_stack_range(take_count, u8::MAX, player).await;
                        if let Some(taken) = taken {
                            // Reverse order of operations, shouldn't affect anything
                            *cursor_stack = taken;
                            slot.on_take_item(player, &taken).await;
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
                            slot.on_take_item(player, &taken).await;
                        }
                    }
                }

                slot.mark_dirty().await;
            }
        } else if action_type == SlotActionType::Swap && (0..9).contains(&button) || button == 40 {
            let mut button_stack = *player
                .get_inventory()
                .get_stack(button as usize)
                .await
                .lock()
                .await;
            let source_slot = self.get_behaviour().slots[slot_index as usize].clone();
            let source_stack = source_slot.get_cloned_stack().await;

            if !button_stack.is_empty() || !source_stack.is_empty() {
                if button_stack.is_empty() {
                    if source_slot.can_take_items(player).await {
                        player
                            .get_inventory()
                            .set_stack(button as usize, source_stack)
                            .await;
                        source_slot.on_take(source_stack.item_count);
                        source_slot.set_stack(ItemStack::EMPTY).await;
                        source_slot.on_take_item(player, &source_stack).await;
                    }
                } else if source_stack.is_empty() {
                    if source_slot.can_insert(&button_stack).await {
                        let max_count = source_slot
                            .get_max_item_count_for_stack(&button_stack)
                            .await;
                        if button_stack.item_count > max_count {
                            // button_stack might need to be a ref instead of a clone
                            source_slot.set_stack(button_stack.split(max_count)).await;
                        } else {
                            player
                                .get_inventory()
                                .set_stack(button as usize, ItemStack::EMPTY)
                                .await;
                            source_slot.set_stack(button_stack).await;
                        }
                    }
                } else if source_slot.can_take_items(player).await
                    && source_slot.can_insert(&button_stack).await
                {
                    let max_count = source_slot
                        .get_max_item_count_for_stack(&button_stack)
                        .await;
                    if button_stack.item_count > max_count {
                        source_slot.set_stack(button_stack.split(max_count)).await;
                        source_slot.on_take_item(player, &button_stack).await;
                        if !player
                            .get_inventory()
                            .insert_stack_anywhere(&mut button_stack)
                            .await
                        {
                            player.drop_item(button_stack, true).await;
                        }
                    } else {
                        player
                            .get_inventory()
                            .set_stack(button as usize, source_stack)
                            .await;
                        source_slot.set_stack(button_stack).await;
                        source_slot.on_take_item(player, &button_stack).await;
                    }
                }
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

pub trait ScreenHandlerListener: Send + Sync {
    fn on_slot_update(
        &self,
        _screen_handler: &ScreenHandlerBehaviour,
        _slot: u8,
        _stack: ItemStack,
    ) {
    }
    fn on_property_update(
        &self,
        _screen_handler: &ScreenHandlerBehaviour,
        _property: u8,
        _value: i32,
    ) {
    }
}

pub trait ScreenHandlerFactory: Send + Sync {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<Arc<Mutex<dyn ScreenHandler>>>;
    fn get_display_name(&self) -> TextComponent;
}

pub struct ScreenHandlerBehaviour {
    pub slots: Vec<Arc<dyn Slot>>,
    pub sync_id: u8,
    pub listeners: Vec<Arc<dyn ScreenHandlerListener>>,
    pub sync_handler: Option<Arc<SyncHandler>>,
    //TODO: Check if this is needed
    pub tracked_stacks: Vec<ItemStack>,
    pub cursor_stack: Arc<Mutex<ItemStack>>,
    pub previous_tracked_stacks: Vec<TrackedStack>,
    pub previous_cursor_stack: TrackedStack,
    pub revision: u32,
    pub disable_sync: bool,
    pub properties: Vec<ScreenProperty>,
    pub tracked_property_values: Vec<i32>,
    pub window_type: Option<WindowType>,
}

impl ScreenHandlerBehaviour {
    pub fn new(sync_id: u8, window_type: Option<WindowType>) -> Self {
        Self {
            slots: Vec::new(),
            sync_id,
            listeners: Vec::new(),
            sync_handler: None,
            tracked_stacks: Vec::new(),
            cursor_stack: Arc::new(Mutex::new(ItemStack::EMPTY)),
            previous_tracked_stacks: Vec::new(),
            previous_cursor_stack: TrackedStack::EMPTY,
            revision: 0,
            disable_sync: false,
            properties: Vec::new(),
            tracked_property_values: Vec::new(),
            window_type,
        }
    }

    pub fn next_revision(&mut self) -> u32 {
        self.revision = (self.revision + 1) & 32767;
        self.revision
    }
}
