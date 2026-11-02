use std::any::Any;
use std::sync::Arc;
use std::sync::RwLock;

use pumpkin_data::{item_stack::ItemStack, screen::WindowType};
use pumpkin_inventory::inventory::SimpleInventory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use pumpkin_inventory::slot::{NormalSlot, Slot};
use pumpkin_inventory::{Clearable, Inventory};
use pumpkin_util::text::TextComponent;

pub struct PluginGui {
    pub window_type: WindowType,
    pub title: TextComponent,
    pub inventory: Arc<PluginInventory>,
    pub allow_grab_items: bool,
    pub allow_put_items: bool,
}

impl PluginGui {
    /// Returns the number of plugin-owned slots in the screen.
    #[must_use]
    pub const fn inventory_size(window_type: WindowType) -> usize {
        match window_type {
            WindowType::Generic9x1 | WindowType::Generic3x3 | WindowType::Crafter3x3 => 9,
            WindowType::Generic9x2 => 18,
            WindowType::Generic9x3 | WindowType::ShulkerBox => 27,
            WindowType::Generic9x4 => 36,
            WindowType::Generic9x5 => 45,
            WindowType::Generic9x6 => 54,
            WindowType::Anvil
            | WindowType::BlastFurnace
            | WindowType::Furnace
            | WindowType::Grindstone
            | WindowType::Merchant
            | WindowType::Smoker
            | WindowType::CartographyTable => 3,
            WindowType::Beacon | WindowType::Lectern => 1,
            WindowType::BrewingStand | WindowType::Hopper => 5,
            WindowType::Crafting => 10,
            WindowType::Enchantment | WindowType::Stonecutter => 2,
            WindowType::Loom | WindowType::Smithing => 4,
        }
    }
}

pub struct PluginInventory {
    pub slots: RwLock<Vec<ItemStack>>,
}

impl PluginInventory {
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            slots: RwLock::new(vec![ItemStack::EMPTY.clone(); size]),
        }
    }
}

impl Clearable for PluginInventory {
    fn clear(&self) {
        let mut slots = self
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slots.fill_with(|| ItemStack::EMPTY.clone());
    }
}

impl Inventory for PluginInventory {
    fn size(&self) -> usize {
        self.slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    fn is_empty(&self) -> bool {
        let slots = self
            .slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slots.iter().all(ItemStack::is_empty)
    }

    fn get_stack(&self, slot: usize) -> ItemStack {
        let slots = self
            .slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        slots
            .get(slot)
            .cloned()
            .unwrap_or_else(|| ItemStack::EMPTY.clone())
    }

    fn remove_stack(&self, slot: usize) -> ItemStack {
        let mut slots = self
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < slots.len() {
            std::mem::replace(&mut slots[slot], ItemStack::EMPTY.clone())
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn remove_stack_specific(&self, slot: usize, amount: u8) -> ItemStack {
        let mut slots = self
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < slots.len() && !slots[slot].is_empty() && amount > 0 {
            slots[slot].split(amount)
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    fn set_stack(&self, slot: usize, stack: ItemStack) {
        let mut slots = self
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot < slots.len() {
            slots[slot] = stack;
        }
    }

    fn on_open(&self) {}

    fn on_close(&self) {}

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
        player_inventory: &Arc<PlayerInventory>,
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
        if window_type != WindowType::Lectern {
            let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
            handler.add_player_slots(&player_inventory);
        }
        if window_type == WindowType::Crafter3x3 {
            // CrafterMenu puts its noninteractive preview after the player inventory.
            handler.add_slot(Arc::new(PluginPreviewSlot(NormalSlot::new(
                Arc::new(SimpleInventory::new(1)),
                0,
            ))));
        }

        handler
    }
}

struct PluginPreviewSlot(NormalSlot);

impl Slot for PluginPreviewSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.0.get_inventory()
    }

    fn get_index(&self) -> usize {
        self.0.get_index()
    }

    fn set_id(&self, index: usize) {
        self.0.set_id(index);
    }

    fn mark_dirty(&self) {
        self.0.mark_dirty();
    }

    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }

    fn can_take_items(&self, _player: &dyn InventoryPlayer) -> bool {
        false
    }
}

impl ScreenHandler for PluginScreenHandler {
    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.inventory.on_close();
    }

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

    fn quick_move(&mut self, _player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            stack_left = slot_stack.clone();
            let container_slots = self.get_behaviour().container_slots as i32;

            if slot_index < container_slots {
                if !self.get_behaviour().allow_grab_items {
                    return ItemStack::EMPTY.clone();
                }
                if !self.insert_item(
                    &mut slot_stack,
                    container_slots,
                    (container_slots + PlayerInventory::MAIN_SIZE as i32)
                        .min(self.get_behaviour().slots.len() as i32),
                    true,
                ) {
                    return ItemStack::EMPTY.clone();
                }
            } else {
                if !self.get_behaviour().allow_put_items {
                    return ItemStack::EMPTY.clone();
                }
                if !self.insert_item(&mut slot_stack, 0, container_slots, false) {
                    return ItemStack::EMPTY.clone();
                }
            }

            if slot_stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(slot_stack);
            }
        }

        stack_left
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use pumpkin_data::{
        data_component_impl::{BundleContentsImpl, EquipmentSlot},
        item::Item,
        sound::Sound,
        statistic::StatisticCategory,
    };
    use pumpkin_inventory::entity_equipment::EntityEquipment;
    use pumpkin_protocol::java::{
        client::play::{
            CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
            CSetPlayerInventory, CSetSelectedSlot,
        },
        server::play::SlotActionType,
    };

    use super::*;

    struct TestPlayer(Arc<PlayerInventory>);

    impl InventoryPlayer for TestPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) {}
        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.0.clone()
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
        fn enqueue_property_packet(&self, _packet: &CSetContainerProperty) {}
        fn enqueue_slot_set_packet(&self, _packet: &CSetPlayerInventory) {}
        fn enqueue_set_held_item_packet(&self, _packet: &CSetSelectedSlot) {}
        fn enqueue_equipment_change(&self, _slot: &EquipmentSlot, _stack: &ItemStack) {}
        fn award_experience(&self, _amount: i32) {}
        fn increment_stat(&self, _category: StatisticCategory, _stat_id: i32, _amount: i32) {}
        fn play_block_sound(&self, _sound: Sound, _pitch: f32) {}
    }

    fn handler(screen: WindowType, grab: bool, put: bool) -> (TestPlayer, PluginScreenHandler) {
        let inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(rustc_hash::FxHashMap::default()),
        ));
        let plugin_inventory = Arc::new(PluginInventory::new(PluginGui::inventory_size(screen)));
        let handler = PluginScreenHandler::new(1, screen, &inventory, &plugin_inventory, grab, put);
        (TestPlayer(inventory), handler)
    }

    fn set_cursor(handler: &PluginScreenHandler, stack: ItemStack) {
        *handler
            .behaviour
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
    }

    fn cursor(handler: &PluginScreenHandler) -> ItemStack {
        handler
            .behaviour
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[test]
    fn deposit_only_blocks_exchanges_but_accepts_matching_stacks() {
        let (player, mut handler) = handler(WindowType::Generic9x1, false, true);
        handler
            .inventory
            .set_stack(0, ItemStack::new(8, &Item::DIAMOND));
        for button in [0, 1] {
            set_cursor(&handler, ItemStack::new(3, &Item::COBBLESTONE));
            handler.on_slot_click(0, button, SlotActionType::Pickup, &player);
            assert_eq!(handler.inventory.get_stack(0).item.id, Item::DIAMOND.id);
            assert_eq!(handler.inventory.get_stack(0).item_count, 8);
            assert_eq!(cursor(&handler).item.id, Item::COBBLESTONE.id);
            assert_eq!(cursor(&handler).item_count, 3);
        }
        set_cursor(&handler, ItemStack::new(3, &Item::DIAMOND));
        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);
        assert_eq!(handler.inventory.get_stack(0).item_count, 11);
        assert!(cursor(&handler).is_empty());
    }

    #[test]
    fn cursor_bundle_cannot_remove_deposit_only_items() {
        let (player, mut handler) = handler(WindowType::Generic9x1, false, true);
        handler
            .inventory
            .set_stack(0, ItemStack::new(8, &Item::DIAMOND));
        let mut bundle = ItemStack::new(1, &Item::BUNDLE);
        bundle.set_data_component(BundleContentsImpl { items: Vec::new() });
        set_cursor(&handler, bundle);
        handler.on_slot_click(0, 1, SlotActionType::Pickup, &player);
        assert_eq!(handler.inventory.get_stack(0).item_count, 8);
        assert_eq!(cursor(&handler).item.id, Item::BUNDLE.id);
        assert!(
            cursor(&handler)
                .get_data_component::<BundleContentsImpl>()
                .is_some_and(|contents| contents.items.is_empty())
        );
    }

    #[test]
    fn dragging_into_own_inventory_skips_locked_container_slots() {
        let (player, mut handler) = handler(WindowType::Generic9x1, false, false);
        set_cursor(&handler, ItemStack::new(6, &Item::COBBLESTONE));
        handler.on_slot_click(-999, 0, SlotActionType::QuickCraft, &player);
        for slot in [0, 9, 10] {
            handler.on_slot_click(slot, 1, SlotActionType::QuickCraft, &player);
        }
        handler.on_slot_click(-999, 2, SlotActionType::QuickCraft, &player);
        assert!(handler.inventory.is_empty());
        assert_eq!(player.0.get_stack(9).item_count, 3);
        assert_eq!(player.0.get_stack(10).item_count, 3);
        assert!(cursor(&handler).is_empty());
    }

    #[test]
    fn collecting_matching_items_skips_locked_container_slots() {
        let (player, mut handler) = handler(WindowType::Generic9x1, false, false);
        handler
            .inventory
            .set_stack(0, ItemStack::new(11, &Item::DIAMOND));
        player.0.set_stack(9, ItemStack::new(2, &Item::DIAMOND));
        player.0.set_stack(10, ItemStack::new(3, &Item::DIAMOND));
        set_cursor(&handler, ItemStack::new(1, &Item::DIAMOND));
        handler.on_slot_click(11, 0, SlotActionType::PickupAll, &player);
        assert_eq!(handler.inventory.get_stack(0).item_count, 11);
        assert!(player.0.get_stack(9).is_empty());
        assert!(player.0.get_stack(10).is_empty());
        assert_eq!(cursor(&handler).item_count, 6);
    }

    #[test]
    fn one_row_and_furnace_clicks_use_client_inventory_offsets() {
        let (player, mut chest) = handler(WindowType::Generic9x1, true, true);
        chest
            .inventory
            .set_stack(8, ItemStack::new(2, &Item::EMERALD));
        player.0.set_stack(9, ItemStack::new(3, &Item::DIAMOND));
        chest.on_slot_click(8, 0, SlotActionType::Pickup, &player);
        assert_eq!(cursor(&chest).item.id, Item::EMERALD.id);
        assert_eq!(player.0.get_stack(9).item_count, 3);
        set_cursor(&chest, ItemStack::EMPTY.clone());
        chest.on_slot_click(9, 0, SlotActionType::Pickup, &player);
        assert_eq!(cursor(&chest).item.id, Item::DIAMOND.id);
        assert!(player.0.get_stack(9).is_empty());

        let (player, mut furnace) = handler(WindowType::Furnace, true, true);
        player.0.set_stack(9, ItemStack::new(4, &Item::DIAMOND));
        player.0.set_stack(8, ItemStack::new(5, &Item::EMERALD));
        furnace.on_slot_click(3, 0, SlotActionType::Pickup, &player);
        assert_eq!(cursor(&furnace).item.id, Item::DIAMOND.id);
        assert!(player.0.get_stack(9).is_empty());
        set_cursor(&furnace, ItemStack::EMPTY.clone());
        furnace.on_slot_click(38, 0, SlotActionType::Pickup, &player);
        assert_eq!(cursor(&furnace).item.id, Item::EMERALD.id);
        assert!(player.0.get_stack(8).is_empty());
    }

    #[test]
    fn crafter_preview_cannot_receive_items_or_shift_transfer() {
        let (player, mut handler) = handler(WindowType::Crafter3x3, true, true);
        assert!(handler.is_slot_valid(45));
        assert!(!handler.is_slot_valid(46));
        set_cursor(&handler, ItemStack::new(4, &Item::COBBLESTONE));
        handler.on_slot_click(45, 0, SlotActionType::Pickup, &player);
        assert!(handler.behaviour.slots[45].get_stack().is_empty());
        assert_eq!(cursor(&handler).item_count, 4);
        set_cursor(&handler, ItemStack::EMPTY.clone());
        handler
            .inventory
            .set_stack(0, ItemStack::new(3, &Item::DIAMOND));
        handler.on_slot_click(0, 0, SlotActionType::QuickMove, &player);
        assert!(handler.inventory.get_stack(0).is_empty());
        assert_eq!(player.0.get_stack(8).item_count, 3);
        assert!(handler.behaviour.slots[45].get_stack().is_empty());
    }

    #[test]
    fn lectern_has_no_player_slots_and_shift_click_is_safe() {
        let (player, mut handler) = handler(WindowType::Lectern, true, true);
        assert!(handler.is_slot_valid(0));
        assert!(!handler.is_slot_valid(1));
        player.0.set_stack(9, ItemStack::new(2, &Item::DIAMOND));
        handler
            .inventory
            .set_stack(0, ItemStack::new(1, &Item::BOOK));
        handler.on_slot_click(0, 0, SlotActionType::QuickMove, &player);
        assert_eq!(handler.inventory.get_stack(0).item.id, Item::BOOK.id);
        assert_eq!(player.0.get_stack(9).item_count, 2);
    }
}
