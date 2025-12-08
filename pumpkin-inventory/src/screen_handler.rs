use crate::{
    container_click::MouseClick,
    player::player_inventory::PlayerInventory,
    slot::{NormalSlot, Slot},
    sync_handler::{SyncHandler, TrackedStack},
};
use log::warn;
use pumpkin_data::{
    data_component_impl::{EquipmentSlot, EquipmentType, EquippableImpl},
    screen::WindowType,
};
use pumpkin_protocol::{
    codec::item_stack_seralizer::OptionalItemStackHash,
    java::{
        client::play::{
            CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
            CSetPlayerInventory, CSetSelectedSlot,
        },
        server::play::SlotActionType,
    },
};
use pumpkin_util::text::TextComponent;
use pumpkin_world::item::ItemStack;
use pumpkin_world::{
    block::entities::PropertyDelegate,
    inventory::{ComparableInventory, Inventory},
};
use std::sync::atomic::{AtomicU32, Ordering};
use std::{any::Any, collections::HashMap, sync::Arc};
use std::{cmp::max, pin::Pin};
use tokio::sync::Mutex;

const SLOT_INDEX_OUTSIDE: i32 = -999;

pub struct ScreenProperty {
    old_value: i32,
    index: u8,
    value: Arc<dyn PropertyDelegate>,
}

impl ScreenProperty {
    pub fn new(value: Arc<dyn PropertyDelegate>, index: u8) -> Self {
        Self {
            old_value: value.get_property(index as i32),
            index,
            value,
        }
    }

    pub fn get(&self) -> i32 {
        self.value.get_property(self.index as i32)
    }

    pub fn set(&mut self, value: i32) {
        self.value.set_property(self.index as i32, value);
    }

    pub fn has_changed(&mut self) -> bool {
        let value = self.get();
        let has_changed = !value.eq(&self.old_value);
        self.old_value = value;
        has_changed
    }
}

pub type PlayerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait InventoryPlayer: Send + Sync {
    fn drop_item<'a>(&'a self, item: ItemStack, retain_ownership: bool) -> PlayerFuture<'a, ()>;
    fn get_inventory(&self) -> Arc<PlayerInventory>;
    fn has_infinite_materials(&self) -> bool;

    fn enqueue_inventory_packet<'a>(
        &'a self,
        packet: &'a CSetContainerContent,
    ) -> PlayerFuture<'a, ()>;
    fn enqueue_slot_packet<'a>(&'a self, packet: &'a CSetContainerSlot) -> PlayerFuture<'a, ()>;
    fn enqueue_cursor_packet<'a>(&'a self, packet: &'a CSetCursorItem) -> PlayerFuture<'a, ()>;
    fn enqueue_property_packet<'a>(
        &'a self,
        packet: &'a CSetContainerProperty,
    ) -> PlayerFuture<'a, ()>;
    fn enqueue_slot_set_packet<'a>(
        &'a self,
        packet: &'a CSetPlayerInventory,
    ) -> PlayerFuture<'a, ()>;
    fn enqueue_set_held_item_packet<'a>(
        &'a self,
        packet: &'a CSetSelectedSlot,
    ) -> PlayerFuture<'a, ()>;
    fn enqueue_equipment_change<'a>(
        &'a self,
        slot: &'a EquipmentSlot,
        stack: &'a ItemStack,
    ) -> PlayerFuture<'a, ()>;
}

pub async fn offer_or_drop_stack(player: &dyn InventoryPlayer, stack: ItemStack) {
    // TODO: Super weird disconnect logic in vanilla, investigate this later
    player
        .get_inventory()
        .offer_or_drop_stack(stack, player)
        .await;
}

pub type ScreenHandlerFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Future returning ItemStack (used by quick_move)
pub type ItemStackFuture<'a> = ScreenHandlerFuture<'a, ItemStack>;

// Future returning Option<usize>
pub type OptionUsizeFuture<'a> = ScreenHandlerFuture<'a, Option<usize>>;

//ScreenHandler.java
// TODO: Fully implement this
pub trait ScreenHandler: Send + Sync {
    // --- Synchronous Methods (Unchanged) ---

    fn window_type(&self) -> Option<WindowType> {
        self.get_behaviour().window_type
    }

    fn as_any(&self) -> &dyn Any;

    fn sync_id(&self) -> u8 {
        self.get_behaviour().sync_id
    }

    fn can_use(&self, _player: &dyn InventoryPlayer) -> bool {
        true
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour;

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour;

    fn add_slot(&mut self, slot: Arc<dyn Slot>) -> Arc<dyn Slot> {
        let behaviour = self.get_behaviour_mut();
        slot.set_id(behaviour.slots.len());
        behaviour.slots.push(slot.clone());
        behaviour.tracked_stacks.push(ItemStack::EMPTY.clone());
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

    fn set_received_hash(&mut self, slot: usize, hash: OptionalItemStackHash) {
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

    fn set_received_stack(&mut self, slot: usize, stack: ItemStack) {
        let behaviour = self.get_behaviour_mut();
        behaviour.previous_tracked_stacks[slot].set_received_stack(stack);
    }

    fn set_received_cursor_hash(&mut self, hash: OptionalItemStackHash) {
        let behaviour = self.get_behaviour_mut();
        behaviour.previous_cursor_stack.set_received_hash(hash);
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

    // --- Asynchronous Methods (Refactored) ---

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
        })
    }

    fn default_on_closed<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();

            // Lock and clone are performed inside the async block
            let mut cursor_stack_lock = behaviour.cursor_stack.lock().await;

            if !cursor_stack_lock.is_empty() {
                offer_or_drop_stack(player, cursor_stack_lock.clone()).await;
                *cursor_stack_lock = ItemStack::EMPTY.clone();
            }
        })
    }

    fn drop_inventory<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        inventory: Arc<dyn Inventory>,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            for i in 0..inventory.size() {
                offer_or_drop_stack(player, inventory.remove_stack(i).await).await;
            }
        })
    }

    fn copy_shared_slots<'a>(
        &'a mut self,
        other: Arc<Mutex<dyn ScreenHandler>>,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
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
                            other_behaviour.tracked_stacks[*other_index].clone();
                        self.get_behaviour_mut().previous_tracked_stacks[i] =
                            other_behaviour.previous_tracked_stacks[*other_index].clone();
                    }
                }
            }
        })
    }

    fn sync_state<'a>(&'a mut self) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            let mut previous_tracked_stacks = Vec::new();

            for i in 0..behaviour.slots.len() {
                let stack = behaviour.slots[i].get_cloned_stack().await;
                previous_tracked_stacks.push(stack.clone());
                behaviour.previous_tracked_stacks[i].set_received_stack(stack);
            }

            let cursor_stack = behaviour.cursor_stack.lock().await.clone();
            behaviour
                .previous_cursor_stack
                .set_received_stack(cursor_stack.clone());

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
        })
    }

    fn add_listener<'a>(
        &'a mut self,
        listener: Arc<dyn ScreenHandlerListener>,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.get_behaviour_mut().listeners.push(listener);
            self.send_content_updates().await;
        })
    }

    fn update_sync_handler<'a>(
        &'a mut self,
        sync_handler: Arc<SyncHandler>,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            behaviour.sync_handler = Some(sync_handler.clone());
            self.sync_state().await;
        })
    }

    fn update_to_client<'a>(&'a mut self) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            for i in 0..self.get_behaviour().slots.len() {
                let behaviour = self.get_behaviour_mut();
                let slot = behaviour.slots[i].clone();
                let stack = slot.get_cloned_stack().await;
                self.update_tracked_slot(i, stack).await;
            }

            let behaviour = self.get_behaviour_mut();
            let mut prop_vec = vec![];
            for (idx, prop) in behaviour.properties.iter_mut().enumerate() {
                let value = prop.get();
                if prop.has_changed() {
                    prop_vec.push((idx, value));
                }
            }

            for (idx, value) in prop_vec {
                self.update_tracked_properties(idx as i32, value).await;
                self.check_property_updates(idx as i32, value).await;
            }

            self.sync_state().await;
        })
    }

    fn update_tracked_properties<'a>(
        &'a mut self,
        idx: i32,
        value: i32,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            if idx <= behaviour.tracked_property_values.len() as i32 {
                behaviour.tracked_property_values[idx as usize] = value;
                for listener in behaviour.listeners.iter() {
                    listener
                        .on_property_update(behaviour, idx as u8, value)
                        .await;
                }
            }
        })
    }

    fn check_property_updates<'a>(
        &'a mut self,
        idx: i32,
        value: i32,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            if !behaviour.disable_sync {
                if let Some(old_value) = behaviour.tracked_property_values.get(idx as usize) {
                    let old_value = *old_value;
                    if old_value != value {
                        behaviour
                            .tracked_property_values
                            .insert(idx as usize, value);
                        if let Some(ref sync_handler) = behaviour.sync_handler {
                            sync_handler.update_property(behaviour, idx, value).await;
                        }
                    }
                }
            }
        })
    }

    fn update_tracked_slot<'a>(
        &'a mut self,
        slot: usize,
        stack: ItemStack,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            let other_stack = &behaviour.tracked_stacks[slot];
            if !other_stack.are_equal(&stack) {
                behaviour.tracked_stacks[slot] = stack.clone();

                for listener in behaviour.listeners.iter() {
                    listener
                        .on_slot_update(behaviour, slot as u8, stack.clone())
                        .await;
                }
            }
        })
    }

    fn check_slot_updates<'a>(
        &'a mut self,
        slot: usize,
        stack: ItemStack,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            if !behaviour.disable_sync {
                let prev_stack = &mut behaviour.previous_tracked_stacks[slot];

                if !prev_stack.is_in_sync(&stack) {
                    prev_stack.set_received_stack(stack.clone());
                    let next_revision = behaviour.next_revision();
                    if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                        sync_handler
                            .update_slot(behaviour, slot, &stack, next_revision)
                            .await;
                    }
                }
            }
        })
    }

    fn check_cursor_stack_updates<'a>(&'a mut self) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let behaviour = self.get_behaviour_mut();
            if !behaviour.disable_sync {
                let cursor_stack = behaviour.cursor_stack.lock().await;
                if !behaviour.previous_cursor_stack.is_in_sync(&cursor_stack) {
                    behaviour
                        .previous_cursor_stack
                        .set_received_stack(cursor_stack.clone());
                    if let Some(sync_handler) = behaviour.sync_handler.as_ref() {
                        sync_handler
                            .update_cursor_stack(behaviour, &cursor_stack)
                            .await;
                    }
                }
            }
        })
    }

    fn send_content_updates<'a>(&'a mut self) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            let slots_len = self.get_behaviour().slots.len();

            for i in 0..slots_len {
                let slot = self.get_behaviour().slots[i].clone();
                let stack = slot.get_cloned_stack().await;

                self.update_tracked_slot(i, stack.clone()).await;
                self.check_slot_updates(i, stack).await;
            }

            self.check_cursor_stack_updates().await;

            let behaviour = self.get_behaviour_mut();
            let mut prop_vec = vec![];
            for (idx, prop) in behaviour.properties.iter_mut().enumerate() {
                let value = prop.get();
                if prop.has_changed() {
                    prop_vec.push((idx, value));
                }
            }

            for (idx, value) in prop_vec {
                self.update_tracked_properties(idx as i32, value).await;
                self.check_property_updates(idx as i32, value).await;
            }
        })
    }

    fn is_slot_valid<'a>(&'a self, slot: i32) -> ScreenHandlerFuture<'a, bool> {
        Box::pin(async move {
            slot == -1 || slot == -999 || slot < self.get_behaviour().slots.len() as i32
        })
    }

    fn disable_sync(&mut self) {
        let behaviour = self.get_behaviour_mut();
        behaviour.disable_sync = true;
    }

    fn enable_sync(&mut self) {
        let behaviour = self.get_behaviour_mut();
        behaviour.disable_sync = false;
    }

    fn get_slot_index<'a>(
        &'a self,
        inventory: &'a Arc<dyn Inventory>,
        slot: usize,
    ) -> OptionUsizeFuture<'a> {
        Box::pin(async move {
            (0..self.get_behaviour().slots.len()).find(|&i| {
                Arc::ptr_eq(&self.get_behaviour().slots[i].get_inventory(), inventory)
                    && self.get_behaviour().slots[i].get_index() == slot
            })
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a>;

    fn insert_item<'a>(
        &'a mut self,
        stack: &'a mut ItemStack,
        start_index: i32,
        end_index: i32,
        from_last: bool,
    ) -> ScreenHandlerFuture<'a, bool> {
        Box::pin(async move {
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
                    let slot_stack_lock = slot.get_stack().await;
                    let mut slot_stack = slot_stack_lock.lock().await;

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
        })
    }

    fn handle_slot_click<'a>(
        &'a self,
        _player: &'a dyn InventoryPlayer,
        _click_type: MouseClick,
        _slot: Arc<dyn Slot>,
        _slot_stack: ItemStack,
        _cursor_stack: ItemStack,
    ) -> ScreenHandlerFuture<'a, bool> {
        Box::pin(async {
            // TODO: required for bundle in the future
            false
        })
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
        })
    }

    fn internal_on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            if action_type == SlotActionType::PickupAll && button == 0 {
                let behavior = self.get_behaviour_mut();
                let mut cursor_stack = behavior.cursor_stack.lock().await;
                let mut to_pick_up = cursor_stack.get_max_stack_size() - cursor_stack.item_count;

                for slot in behavior.slots.iter() {
                    if to_pick_up == 0 {
                        break;
                    }

                    let item_stack = slot.get_cloned_stack().await;
                    if !item_stack.are_items_and_components_equal(&cursor_stack) {
                        continue;
                    }

                    if !slot.allow_modification(player).await {
                        continue;
                    }

                    let taken_stack = slot
                        .safe_take(
                            item_stack.item_count.min(to_pick_up),
                            cursor_stack.get_max_stack_size() - cursor_stack.item_count,
                            player,
                        )
                        .await;
                    to_pick_up -= taken_stack.item_count;
                    cursor_stack.increment(taken_stack.item_count);
                }
            } else if action_type == SlotActionType::QuickCraft {
                let drag_type = button & 3;
                let drag_button = (button >> 2) & 3;
                let behaviour = self.get_behaviour_mut();
                if drag_type == 0 {
                    behaviour.drag_slots.clear();
                } else if drag_type == 1 {
                    if slot_index < 0 {
                        warn!("Invalid slot index for drag action: {slot_index}. Must be >= 0");
                        return;
                    }
                    let cursor_stack = behaviour.cursor_stack.lock().await;

                    let slot = &behaviour.slots[slot_index as usize];
                    let stack_lock = slot.get_stack().await;
                    let stack = stack_lock.lock().await;
                    if !cursor_stack.is_empty()
                        && slot.can_insert(&cursor_stack).await
                        && (stack.are_items_and_components_equal(&cursor_stack) || stack.is_empty())
                        && slot.get_max_item_count_for_stack(&stack).await > stack.item_count
                    {
                        behaviour.drag_slots.push(slot_index as u32);
                    }
                } else if drag_type == 2 && !behaviour.drag_slots.is_empty() {
                    // process drag end
                    if behaviour.drag_slots.len() == 1 {
                        let slot = behaviour.drag_slots[0] as i32;
                        behaviour.drag_slots.clear();
                        let _ = behaviour;
                        self.internal_on_slot_click(
                            slot,
                            drag_button,
                            SlotActionType::Pickup,
                            player,
                        )
                        .await;

                        return;
                    }
                    if drag_button == 2 && !player.has_infinite_materials() {
                        return; // Only creative
                    }

                    let mut cursor_stack = behaviour.cursor_stack.lock().await;
                    let initial_count = cursor_stack.item_count;
                    for slot_index in behaviour.drag_slots.iter() {
                        let slot = behaviour.slots[*slot_index as usize].clone();
                        let stack_lock = slot.get_stack().await;
                        let stack = stack_lock.lock().await;

                        if (stack.are_items_and_components_equal(&cursor_stack) || stack.is_empty())
                            && slot.can_insert(&cursor_stack).await
                        {
                            let mut inserting_count = if drag_button == 0 {
                                initial_count / behaviour.drag_slots.len() as u8
                            } else if drag_button == 1 {
                                1
                            } else if drag_button == 2 {
                                cursor_stack.get_max_stack_size()
                            } else {
                                panic!("Invalid drag button: {drag_button}");
                            };
                            inserting_count = inserting_count
                                .min(max(
                                    0,
                                    slot.get_max_item_count_for_stack(&stack).await
                                        - stack.item_count,
                                ))
                                .min(cursor_stack.item_count);
                            if inserting_count > 0 {
                                let mut stack_clone = stack.clone();
                                drop(stack);
                                if stack_clone.is_empty() {
                                    stack_clone = cursor_stack.copy_with_count(0);
                                }
                                stack_clone.increment(inserting_count);
                                slot.set_stack(stack_clone).await;
                                if drag_button != 2 {
                                    cursor_stack.decrement(inserting_count);
                                }
                                if cursor_stack.is_empty() {
                                    break;
                                }
                            }
                        }
                    }

                    behaviour.drag_slots.clear();
                }
            } else if action_type == SlotActionType::Throw {
                if slot_index >= 0 && self.get_behaviour().cursor_stack.lock().await.is_empty() {
                    let slot = self.get_behaviour().slots[slot_index as usize].clone();
                    let prev_stack = slot.get_cloned_stack().await;
                    if !prev_stack.is_empty() {
                        if button == 1 {
                            // Throw all
                            while slot
                                .get_cloned_stack()
                                .await
                                .are_items_and_components_equal(&prev_stack)
                            {
                                let drop_stack =
                                    slot.safe_take(prev_stack.item_count, u8::MAX, player).await;
                                player.drop_item(drop_stack, true).await;
                                // player.handleCreativeModeItemDrop(itemStack);
                            }
                        } else {
                            let drop_stack = slot.safe_take(1, u8::MAX, player).await;
                            if !drop_stack.is_empty() {
                                slot.on_take_item(player, &drop_stack).await;
                                player.drop_item(drop_stack, true).await;
                            }
                        }
                    }
                }
            } else if action_type == SlotActionType::Clone {
                if player.has_infinite_materials() && slot_index >= 0 {
                    let behaviour = self.get_behaviour_mut();
                    let slot = behaviour.slots[slot_index as usize].clone();
                    let stack_lock = slot.get_stack().await;
                    let stack = stack_lock.lock().await;
                    let mut cursor_stack = behaviour.cursor_stack.lock().await;
                    *cursor_stack = stack.copy_with_count(stack.get_max_stack_size());
                }
            } else if (action_type == SlotActionType::Pickup
                || action_type == SlotActionType::QuickMove)
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
                            *cursor_stack = ItemStack::EMPTY.clone();
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

                    let equipment_slot = cursor_stack
                        .get_data_component::<EquippableImpl>()
                        .map_or(&EquipmentSlot::MAIN_HAND, |equippable| equippable.slot);

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
                            if equipment_slot.slot_type() == EquipmentType::HumanoidArmor
                                && (5..9).contains(&slot_index)
                            {
                                player
                                    .enqueue_equipment_change(equipment_slot, &cursor_stack)
                                    .await;
                            }

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
                                slot_stack.item_count.div_ceil(2)
                            };
                            let taken =
                                slot.try_take_stack_range(take_count, u8::MAX, player).await;
                            if let Some(taken) = taken {
                                // Reverse order of operations, shouldn't affect anything
                                *cursor_stack = taken.clone();
                                slot.on_take_item(player, &taken).await;

                                if (5..9).contains(&slot_index) {
                                    let equipment_slot = cursor_stack
                                        .get_data_component::<EquippableImpl>()
                                        .map_or(&EquipmentSlot::MAIN_HAND, |equippable| {
                                            equippable.slot
                                        });
                                    player
                                        .enqueue_equipment_change(equipment_slot, ItemStack::EMPTY)
                                        .await;
                                }
                            }
                        } else if slot.can_insert(&cursor_stack).await {
                            if equipment_slot.slot_type() == EquipmentType::HumanoidArmor
                                && (5..9).contains(&slot_index)
                            {
                                player
                                    .enqueue_equipment_change(equipment_slot, &cursor_stack)
                                    .await;
                            }

                            if ItemStack::are_items_and_components_equal(&slot_stack, &cursor_stack)
                            {
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
                        } else if ItemStack::are_items_and_components_equal(
                            &slot_stack,
                            &cursor_stack,
                        ) {
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
            } else if action_type == SlotActionType::Swap && (0..9).contains(&button)
                || button == 40
            {
                let mut button_stack = player
                    .get_inventory()
                    .get_stack(button as usize)
                    .await
                    .lock()
                    .await
                    .clone();
                let source_slot = self.get_behaviour().slots[slot_index as usize].clone();
                let source_stack = source_slot.get_cloned_stack().await;

                if !button_stack.is_empty() || !source_stack.is_empty() {
                    if button_stack.is_empty() {
                        if source_slot.can_take_items(player).await {
                            player
                                .get_inventory()
                                .set_stack(button as usize, source_stack.clone())
                                .await;
                            source_slot.set_stack(ItemStack::EMPTY.clone()).await;
                            source_slot.on_take_item(player, &source_stack).await;
                        }
                    } else if source_stack.is_empty() && source_slot.can_insert(&button_stack).await
                    {
                        let max_count = source_slot
                            .get_max_item_count_for_stack(&button_stack)
                            .await;
                        if button_stack.item_count > max_count {
                            // button_stack might need to be a ref instead of a clone
                            source_slot.set_stack(button_stack.split(max_count)).await;
                        } else {
                            player
                                .get_inventory()
                                .set_stack(button as usize, ItemStack::EMPTY.clone())
                                .await;
                            source_slot.set_stack(button_stack).await;
                        }
                    }
                }
            }
        })
    }
}

pub trait ScreenHandlerListener: Send + Sync {
    fn on_slot_update<'a>(
        &'a self,
        _screen_handler: &'a ScreenHandlerBehaviour,
        _slot: u8,
        _stack: ItemStack,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
    fn on_property_update<'a>(
        &'a self,
        _screen_handler: &'a ScreenHandlerBehaviour,
        _property: u8,
        _value: i32,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type SharedScreenHandler = Arc<Mutex<dyn ScreenHandler>>;

pub trait ScreenHandlerFactory: Send + Sync {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>>;
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
    pub revision: AtomicU32,
    pub disable_sync: bool,
    pub properties: Vec<ScreenProperty>,
    pub tracked_property_values: Vec<i32>,
    pub window_type: Option<WindowType>,
    pub drag_slots: Vec<u32>,
}

impl ScreenHandlerBehaviour {
    pub fn new(sync_id: u8, window_type: Option<WindowType>) -> Self {
        Self {
            slots: Vec::new(),
            sync_id,
            listeners: Vec::new(),
            sync_handler: None,
            tracked_stacks: Vec::new(),
            cursor_stack: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            previous_tracked_stacks: Vec::new(),
            previous_cursor_stack: TrackedStack::EMPTY,
            revision: AtomicU32::new(0),
            disable_sync: false,
            properties: Vec::new(),
            tracked_property_values: Vec::new(),
            window_type,
            drag_slots: Vec::new(),
        }
    }

    pub fn next_revision(&self) -> u32 {
        self.revision.fetch_add(1, Ordering::Relaxed);
        self.revision.fetch_and(32767, Ordering::Relaxed) & 32767
    }
}
