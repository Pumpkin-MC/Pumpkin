//! Crafter screen handler (`CrafterMenu`).

use std::{
    any::Any,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};

use crate::{
    SimpleInventory,
    crafting::{
        crafting_screen_handler::match_crafting_recipe, recipe_provider::RecipeProvider,
        recipes::RecipeInputInventory,
    },
    inventory::Inventory,
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
        ScreenProperty,
    },
    slot::Slot,
    window_property::PropertyDelegate,
};

/// Input slot in a `CrafterMenu` (`CrafterSlot`).
pub struct CrafterSlot {
    pub inventory: Arc<dyn RecipeInputInventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub property_delegate: Arc<dyn PropertyDelegate>,
}

impl CrafterSlot {
    pub fn new(
        inventory: Arc<dyn RecipeInputInventory>,
        index: usize,
        property_delegate: Arc<dyn PropertyDelegate>,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            property_delegate,
        }
    }
}

impl Slot for CrafterSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        self.property_delegate.get_property(self.index as i32)
            != CrafterScreenHandler::SLOT_DISABLED
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

/// Non-interactive recipe preview slot at the end of `CrafterMenu` (`NonInteractiveResultSlot`).
pub struct NonInteractiveResultSlot {
    pub inventory: Arc<SimpleInventory>,
    pub id: AtomicU8,
}

impl NonInteractiveResultSlot {
    #[must_use]
    pub const fn new(inventory: Arc<SimpleInventory>) -> Self {
        Self {
            inventory,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for NonInteractiveResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        0
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        false
    }

    fn allow_modification(&self, _player: &dyn InventoryPlayer) -> bool {
        false
    }

    fn take_stack(&self, _amount: u8) -> ItemStack {
        ItemStack::EMPTY.clone()
    }

    fn on_take_item(&self, _player: &dyn InventoryPlayer, _stack: &ItemStack) {}

    fn try_take_stack_range(
        &self,
        _min: u8,
        _max: u8,
        _player: &dyn InventoryPlayer,
    ) -> Option<ItemStack> {
        None
    }

    fn safe_take(&self, _min: u8, _max: u8, _player: &dyn InventoryPlayer) -> ItemStack {
        ItemStack::EMPTY.clone()
    }

    fn insert_stack(&self, stack: ItemStack) -> ItemStack {
        stack
    }

    fn insert_stack_count(&self, stack: ItemStack, _count: u8) -> ItemStack {
        stack
    }
}

struct CrafterScreenListener {
    container: Arc<dyn RecipeInputInventory>,
    result_inventory: Arc<SimpleInventory>,
    recipe_provider: Option<Arc<dyn RecipeProvider>>,
}

impl CrafterScreenListener {
    fn refresh_recipe_result(&self) -> ItemStack {
        let result =
            match_crafting_recipe(self.container.as_ref(), self.recipe_provider.as_deref())
                .map_or_else(|| ItemStack::EMPTY.clone(), |r| r.to_item_stack());
        self.result_inventory.set_stack(0, result.clone());
        result
    }
}

impl ScreenHandlerListener for CrafterScreenListener {
    fn on_slot_update(
        &self,
        _screen_handler: &ScreenHandlerBehaviour,
        slot: u8,
        _stack: ItemStack,
    ) {
        if (slot as usize) < CrafterScreenHandler::SLOT_COUNT {
            self.refresh_recipe_result();
        }
    }

    fn on_property_update(
        &self,
        screen_handler: &ScreenHandlerBehaviour,
        property: u8,
        value: i32,
    ) {
        if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
            sync_handler.update_property(screen_handler, i32::from(property), value);
        }
    }
}

/// Server-side `CrafterMenu` implementation.
pub struct CrafterScreenHandler {
    pub container: Arc<dyn RecipeInputInventory>,
    pub property_delegate: Arc<dyn PropertyDelegate>,
    pub result_inventory: Arc<SimpleInventory>,
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
    pub is_spectator: bool,
    behaviour: ScreenHandlerBehaviour,
}

impl CrafterScreenHandler {
    pub const SLOT_COUNT: usize = 9;
    pub const INV_SLOT_START: usize = 9;
    pub const INV_SLOT_END: usize = 36;
    pub const USE_ROW_SLOT_START: usize = 36;
    pub const USE_ROW_SLOT_END: usize = 45;
    pub const RESULT_SLOT: usize = 45;

    pub const SLOT_ENABLED: i32 = 0;
    pub const SLOT_DISABLED: i32 = 1;
    pub const DATA_TRIGGERED: i32 = 9;
    pub const NUM_DATA: usize = 10;

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        container: Arc<dyn RecipeInputInventory>,
        property_delegate: Arc<dyn PropertyDelegate>,
        recipe_provider: Option<Arc<dyn RecipeProvider>>,
        is_spectator: bool,
    ) -> Self {
        let mut behaviour = ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Crafter3x3));
        behaviour.container_slots = Self::SLOT_COUNT;
        behaviour.allow_grab_items = !is_spectator;
        behaviour.allow_put_items = !is_spectator;

        let result_inventory = Arc::new(SimpleInventory::new(1));
        let mut handler = Self {
            container: container.clone(),
            property_delegate,
            result_inventory: result_inventory.clone(),
            recipe_provider: recipe_provider.clone(),
            is_spectator,
            behaviour,
        };

        if !is_spectator {
            handler.container.on_open();
        }

        for row in 0..3 {
            for col in 0..3 {
                let index = col + row * 3;
                let delegate = handler.property_delegate.clone();
                handler.add_slot(Arc::new(CrafterSlot::new(
                    container.clone(),
                    index,
                    delegate,
                )));
            }
        }

        let player_inventory_dyn: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory_dyn);
        handler.add_slot(Arc::new(NonInteractiveResultSlot::new(
            result_inventory.clone(),
        )));

        for i in 0..Self::NUM_DATA {
            let delegate = handler.property_delegate.clone();
            handler.add_property(ScreenProperty::new(delegate, i as u8));
        }

        let listener = Arc::new(CrafterScreenListener {
            container,
            result_inventory,
            recipe_provider,
        });
        listener.refresh_recipe_result();
        handler.add_listener(listener);

        handler
    }

    pub fn set_slot_state(&mut self, slot: usize, enabled: bool) {
        if slot < Self::SLOT_COUNT {
            let value = if enabled {
                Self::SLOT_ENABLED
            } else {
                Self::SLOT_DISABLED
            };
            self.property_delegate.set_property(slot as i32, value);
            self.send_content_updates();
        }
    }

    #[must_use]
    pub fn is_slot_disabled(&self, slot: usize) -> bool {
        slot < Self::SLOT_COUNT
            && self.property_delegate.get_property(slot as i32) == Self::SLOT_DISABLED
    }

    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.property_delegate.get_property(Self::DATA_TRIGGERED) == 1
    }
}

impl ScreenHandler for CrafterScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        if !self.is_spectator && !player.is_spectator() {
            self.container.on_close();
        }
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        if !(0..Self::USE_ROW_SLOT_END as i32).contains(&slot_index) {
            return ItemStack::EMPTY.clone();
        }

        let Some(slot) = self.get_behaviour().slots.get(slot_index as usize).cloned() else {
            return ItemStack::EMPTY.clone();
        };
        if !slot.has_stack() {
            return ItemStack::EMPTY.clone();
        }

        let mut stack = slot.get_stack();
        let clicked = stack.clone();

        if (slot_index as usize) < Self::SLOT_COUNT {
            if !self.insert_item(
                &mut stack,
                Self::INV_SLOT_START as i32,
                Self::USE_ROW_SLOT_END as i32,
                true,
            ) {
                return ItemStack::EMPTY.clone();
            }
        } else if !self.insert_item(&mut stack, 0, Self::SLOT_COUNT as i32, false) {
            return ItemStack::EMPTY.clone();
        }

        if stack.is_empty() {
            slot.set_stack(ItemStack::EMPTY.clone());
        } else {
            slot.set_stack(stack.clone());
        }

        if stack.item_count == clicked.item_count {
            return ItemStack::EMPTY.clone();
        }

        let mut taken_stack = clicked.clone();
        taken_stack.set_count(clicked.item_count - stack.item_count);
        slot.on_take_item(player, &taken_stack);
        clicked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        crafting::crafting_inventory::CraftingInventory, entity_equipment::EntityEquipment,
    };
    use pumpkin_data::{
        data_component_impl::EquipmentSlot, item::Item, sound::Sound, statistic::StatisticCategory,
    };
    use pumpkin_protocol::java::{
        client::play::{
            CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
            CSetPlayerInventory, CSetSelectedSlot,
        },
        server::play::SlotActionType,
    };
    use std::sync::Mutex;

    struct DummyCrafterDelegate {
        values: Mutex<[i32; CrafterScreenHandler::NUM_DATA]>,
    }

    impl DummyCrafterDelegate {
        fn new() -> Self {
            Self {
                values: Mutex::new([0; CrafterScreenHandler::NUM_DATA]),
            }
        }
    }

    impl PropertyDelegate for DummyCrafterDelegate {
        fn get_property(&self, index: i32) -> i32 {
            self.values
                .lock()
                .unwrap()
                .get(index as usize)
                .copied()
                .unwrap_or(0)
        }

        fn set_property(&self, index: i32, value: i32) {
            if let Some(slot) = self.values.lock().unwrap().get_mut(index as usize) {
                *slot = value;
            }
        }

        fn get_properties_size(&self) -> i32 {
            CrafterScreenHandler::NUM_DATA as i32
        }
    }

    struct DummyPlayer {
        inventory: Arc<PlayerInventory>,
    }

    impl InventoryPlayer for DummyPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) {}
        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }
        fn has_infinite_materials(&self) -> bool {
            false
        }
        fn is_creative(&self) -> bool {
            false
        }
        fn experience_level(&self) -> i32 {
            0
        }
        fn add_experience_levels(&self, _levels: i32) {}
        fn enchantment_seed(&self) -> i32 {
            0
        }
        fn set_enchantment_seed(&self, _seed: i32) {}
        fn enqueue_inventory_packet(
            &self,
            _packet: &CSetContainerContent,
            _window_type: Option<WindowType>,
        ) {
        }
        fn enqueue_slot_packet(
            &self,
            _packet: &CSetContainerSlot,
            _window_type: Option<WindowType>,
            _total_slots: usize,
        ) {
        }
        fn enqueue_cursor_packet(&self, _packet: &CSetCursorItem) {}
        fn enqueue_property_packet(
            &self,
            _packet: &CSetContainerProperty,
            _window_type: Option<WindowType>,
        ) {
        }
        fn enqueue_slot_set_packet(&self, _packet: &CSetPlayerInventory) {}
        fn enqueue_set_held_item_packet(&self, _packet: &CSetSelectedSlot) {}
        fn enqueue_equipment_change(&self, _slot: &EquipmentSlot, _stack: &ItemStack) {}
        fn award_experience(&self, _amount: i32) {}
        fn increment_stat(&self, _category: StatisticCategory, _stat_id: i32, _amount: i32) {}
        fn play_block_sound(&self, _sound: Sound, _pitch: f32) {}
    }

    #[test]
    fn crafter_menu_updates_noninteractive_preview_and_skips_disabled_slots() {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(rustc_hash::FxHashMap::default()),
        ));
        let player = DummyPlayer {
            inventory: player_inventory.clone(),
        };
        let grid: Arc<dyn RecipeInputInventory> = Arc::new(CraftingInventory::new(3, 3));
        let delegate: Arc<dyn PropertyDelegate> = Arc::new(DummyCrafterDelegate::new());

        let mut handler =
            CrafterScreenHandler::new(1, &player_inventory, grid.clone(), delegate, None, false);
        assert_eq!(handler.get_behaviour().slots.len(), 46);
        assert_eq!(handler.get_behaviour().properties.len(), 10);

        // Disable slot 0 and shift-click an oak log from the player's hotbar (slot 36).
        handler.set_slot_state(0, false);
        assert!(handler.is_slot_disabled(0));
        handler.get_behaviour().slots[36].set_stack(ItemStack::new(1, &Item::OAK_LOG));
        handler.on_slot_click(36, 0, SlotActionType::QuickMove, &player);

        assert!(grid.get_stack(0).is_empty());
        assert_eq!(grid.get_stack(1).item.id, Item::OAK_LOG.id);

        handler.send_content_updates();
        let preview = handler.get_behaviour().slots[CrafterScreenHandler::RESULT_SLOT].get_stack();
        assert_eq!(preview.item.id, Item::OAK_PLANKS.id);
        assert_eq!(preview.item_count, 4);

        // Clicking or shift-clicking the preview slot cannot take items or consume inputs.
        handler.on_slot_click(
            CrafterScreenHandler::RESULT_SLOT as i32,
            0,
            SlotActionType::Pickup,
            &player,
        );
        handler.on_slot_click(
            CrafterScreenHandler::RESULT_SLOT as i32,
            0,
            SlotActionType::QuickMove,
            &player,
        );
        assert_eq!(grid.get_stack(1).item.id, Item::OAK_LOG.id);
        assert!(
            handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap()
                .is_empty()
        );
    }
}
