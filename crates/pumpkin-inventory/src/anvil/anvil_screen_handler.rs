use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering};

use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{EnchantmentsImpl, StoredEnchantmentsImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, offer_or_drop_stack},
    slot::{NormalSlot, Slot},
    window_property::{Anvil, WindowProperty},
};

pub struct AnvilResultSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
    pub cost: Arc<AtomicI32>,
    pub repair_item_count_cost: Arc<AtomicI32>,
    pub only_renaming: Arc<AtomicBool>,
}

impl AnvilResultSlot {
    #[must_use]
    pub fn new(
        inventory: Arc<dyn Inventory>,
        index: usize,
        cost: Arc<AtomicI32>,
        repair_item_count_cost: Arc<AtomicI32>,
        only_renaming: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
            cost,
            repair_item_count_cost,
            only_renaming,
        }
    }
}

impl Slot for AnvilResultSlot {
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
        false
    }

    fn can_take_items(&self, player: &dyn InventoryPlayer) -> bool {
        let cost = self.cost.load(Ordering::Relaxed);
        (player.has_infinite_materials() || player.experience_level() >= cost) && cost > 0
    }

    fn on_take_item(&self, player: &dyn InventoryPlayer, _stack: &ItemStack) {
        let cost = self.cost.load(Ordering::Relaxed);
        if !player.has_infinite_materials() {
            player.add_experience_levels(-cost);
        }

        let repair_item_count_cost = self.repair_item_count_cost.load(Ordering::Relaxed);
        if repair_item_count_cost > 0 {
            let mut addition = self.inventory.get_stack(1);
            if !addition.is_empty() && addition.item_count > repair_item_count_cost as u8 {
                addition.decrement(repair_item_count_cost as u8);
                self.inventory.set_stack(1, addition);
            } else {
                self.inventory.set_stack(1, ItemStack::EMPTY.clone());
            }
        } else if !self.only_renaming.load(Ordering::Relaxed) {
            self.inventory.set_stack(1, ItemStack::EMPTY.clone());
        }

        self.cost.store(0, Ordering::Relaxed);
        self.inventory.set_stack(0, ItemStack::EMPTY.clone());
        player.use_anvil();
        self.mark_dirty();
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

pub struct AnvilScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    pub item_name: Option<String>,
    pub repair_cost: Arc<AtomicI32>,
    pub repair_item_count_cost: Arc<AtomicI32>,
    pub only_renaming: Arc<AtomicBool>,
}

#[must_use]
pub fn validate_name(name: &str) -> Option<String> {
    let filtered: String = name.chars().filter(|&c| c >= ' ' && c != '\x7F').collect();
    (filtered.chars().count() <= 50).then_some(filtered)
}

#[must_use]
pub const fn calculate_increased_repair_cost(base_cost: i32) -> i32 {
    let doubled = (base_cost as i64) * 2 + 1;
    if doubled > i32::MAX as i64 {
        i32::MAX
    } else {
        doubled as i32
    }
}

#[must_use]
pub fn get_component_type(stack: &ItemStack) -> DataComponent {
    if stack.item == &Item::ENCHANTED_BOOK {
        DataComponent::StoredEnchantments
    } else {
        DataComponent::Enchantments
    }
}

#[must_use]
pub fn can_store_enchantments(stack: &ItemStack) -> bool {
    if stack.is_empty() {
        return false;
    }
    stack.has_data_component(get_component_type(stack))
}

#[must_use]
pub fn get_enchantments_for_crafting(stack: &ItemStack) -> Vec<(&'static Enchantment, i32)> {
    if stack.is_empty() {
        return Vec::new();
    }
    if stack.item == &Item::ENCHANTED_BOOK {
        if let Some(stored) = stack.get_data_component::<StoredEnchantmentsImpl>() {
            return stored
                .enchantment
                .iter()
                .map(|(e, lvl)| (*e, *lvl))
                .collect();
        }
    } else if let Some(ench) = stack.get_data_component::<EnchantmentsImpl>() {
        return ench.enchantment.iter().map(|(e, lvl)| (*e, *lvl)).collect();
    }
    Vec::new()
}

pub fn set_enchantments(result: &mut ItemStack, enchantments: Vec<(&'static Enchantment, i32)>) {
    if result.is_empty() {
        return;
    }
    if result.item == &Item::ENCHANTED_BOOK
        || result
            .get_data_component::<StoredEnchantmentsImpl>()
            .is_some()
    {
        let stored = StoredEnchantmentsImpl {
            enchantment: Cow::Owned(enchantments),
        };
        result.set_data_component(stored);
    } else {
        let ench = EnchantmentsImpl {
            enchantment: Cow::Owned(enchantments),
        };
        result.set_data_component(ench);
    }
}

impl AnvilScreenHandler {
    #[expect(clippy::needless_pass_by_value)]
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let cost = Arc::new(AtomicI32::new(0));
        let repair_item_count_cost = Arc::new(AtomicI32::new(0));
        let only_renaming = Arc::new(AtomicBool::new(false));

        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Anvil)),
            item_name: None,
            repair_cost: cost.clone(),
            repair_item_count_cost: repair_item_count_cost.clone(),
            only_renaming: only_renaming.clone(),
        };

        // Slot 0: Input slot
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        // Slot 1: Additional slot
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));
        // Slot 2: Result slot
        handler.add_slot(Arc::new(AnvilResultSlot::new(
            inventory.clone(),
            2,
            cost,
            repair_item_count_cost,
            only_renaming,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub fn set_item_name(&mut self, name: &str, has_infinite_materials: bool) -> bool {
        let validated_name = validate_name(name);
        if let Some(validated_name) = validated_name
            && self.item_name.as_deref() != Some(&validated_name)
        {
            self.item_name = Some(validated_name.clone());
            let result_stack = self.inventory.get_stack(2);
            if !result_stack.is_empty() {
                let mut updated_stack = result_stack;
                if validated_name.trim().is_empty() {
                    updated_stack.remove_custom_name();
                } else {
                    updated_stack.set_custom_name(validated_name);
                }
                self.inventory.set_stack(2, updated_stack);
            }
            self.create_result(has_infinite_materials);
            return true;
        }
        false
    }

    pub fn update_item_name(&mut self, name: &str) {
        self.set_item_name(name, false);
    }

    pub fn update_result_slot(&mut self) {
        self.create_result(false);
    }

    pub fn set_repair_cost(&mut self, cost: i32) {
        self.repair_cost.store(cost, Ordering::Relaxed);
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            let (property_id, property_value) =
                WindowProperty::new(Anvil::RepairCost, cost.clamp(0, i16::MAX as i32) as i16)
                    .into_tuple();
            sync_handler.update_property(
                &self.behaviour,
                property_id as i32,
                property_value as i32,
            );
        }
    }

    #[must_use]
    pub fn get_cost(&self) -> i32 {
        self.repair_cost.load(Ordering::Relaxed)
    }

    #[allow(clippy::too_many_lines)]
    pub fn create_result(&mut self, has_infinite_materials: bool) {
        let input = self.inventory.get_stack(0);
        self.only_renaming.store(false, Ordering::Relaxed);
        self.set_repair_cost(1);
        let mut price: i32 = 0;
        let mut tax: i64 = 0;
        let mut naming_cost: i32 = 0;

        if !input.is_empty() && can_store_enchantments(&input) {
            let mut result = input.clone();
            let addition = self.inventory.get_stack(1);
            let mut enchantments = get_enchantments_for_crafting(&result);
            tax += input.get_repair_cost() as i64 + addition.get_repair_cost() as i64;
            self.repair_item_count_cost.store(0, Ordering::Relaxed);

            if !addition.is_empty() {
                let using_book = addition.item == &Item::ENCHANTED_BOOK
                    || addition
                        .get_data_component::<StoredEnchantmentsImpl>()
                        .is_some_and(|s| !s.enchantment.is_empty());

                if result.is_damageable() && input.is_valid_repair_item(&addition) {
                    let max_damage = result.get_max_damage().unwrap_or(0);
                    let mut repair_amount = result.get_damage().min(max_damage / 4);
                    if repair_amount <= 0 {
                        self.inventory.set_stack(2, ItemStack::EMPTY.clone());
                        self.set_repair_cost(0);
                        return;
                    }

                    let mut count = 0;
                    while repair_amount > 0 && count < addition.item_count as i32 {
                        let result_damage = result.get_damage() - repair_amount;
                        result.set_damage(result_damage);
                        price += 1;
                        repair_amount = result.get_damage().min(max_damage / 4);
                        count += 1;
                    }

                    self.repair_item_count_cost.store(count, Ordering::Relaxed);
                } else {
                    if !using_book && (result.item != addition.item || !result.is_damageable()) {
                        self.inventory.set_stack(2, ItemStack::EMPTY.clone());
                        self.set_repair_cost(0);
                        return;
                    }

                    if result.is_damageable() && !using_book {
                        let max_damage = result.get_max_damage().unwrap_or(0);
                        let remaining1 = max_damage - input.get_damage();
                        let remaining2 =
                            addition.get_max_damage().unwrap_or(0) - addition.get_damage();
                        let additional = remaining2 + max_damage * 12 / 100;
                        let remaining = remaining1 + additional;
                        let mut result_damage = max_damage - remaining;
                        if result_damage < 0 {
                            result_damage = 0;
                        }

                        if result_damage < result.get_damage() {
                            result.set_damage(result_damage);
                            price += 2;
                        }
                    }

                    let additional_enchantments = get_enchantments_for_crafting(&addition);
                    let mut is_any_enchantment_compatible = false;
                    let mut is_any_enchantment_not_compatible = false;

                    for (enchantment, entry_level) in additional_enchantments {
                        let current = enchantments
                            .iter()
                            .find(|(e, _)| *e == enchantment)
                            .map_or(0, |(_, lvl)| *lvl);
                        let mut level = if current == entry_level {
                            entry_level + 1
                        } else {
                            entry_level.max(current)
                        };

                        let mut compatible =
                            if has_infinite_materials || input.item == &Item::ENCHANTED_BOOK {
                                true
                            } else {
                                enchantment.can_enchant(input.item)
                            };

                        for (other, _) in &enchantments {
                            if *other != enchantment && !other.are_compatible(enchantment) {
                                compatible = false;
                                price += 1;
                            }
                        }

                        if compatible {
                            is_any_enchantment_compatible = true;
                            if level > enchantment.max_level {
                                level = enchantment.max_level;
                            }

                            if let Some(entry) =
                                enchantments.iter_mut().find(|(e, _)| *e == enchantment)
                            {
                                entry.1 = level;
                            } else {
                                enchantments.push((enchantment, level));
                            }

                            let mut fee = enchantment.anvil_cost as i32;
                            if using_book {
                                fee = 1.max(fee / 2);
                            }

                            price += fee * level;
                            if input.item_count > 1 {
                                price = 40;
                            }
                        } else {
                            is_any_enchantment_not_compatible = true;
                        }
                    }

                    if is_any_enchantment_not_compatible && !is_any_enchantment_compatible {
                        self.inventory.set_stack(2, ItemStack::EMPTY.clone());
                        self.set_repair_cost(0);
                        return;
                    }
                }
            }

            let input_hover_name = input.get_hover_name();
            if let Some(item_name) = &self.item_name
                && !item_name.trim().is_empty()
            {
                if item_name != &input_hover_name {
                    naming_cost = 1;
                    price += naming_cost;
                    result.set_custom_name(item_name.clone());
                }
            } else if input.has_custom_name() {
                naming_cost = 1;
                price += naming_cost;
                result.remove_custom_name();
            }

            let final_price = if price <= 0 {
                0
            } else {
                (tax + price as i64).clamp(0, i32::MAX as i64) as i32
            };
            self.set_repair_cost(final_price);
            if price <= 0 {
                result = ItemStack::EMPTY.clone();
            }

            if naming_cost == price && naming_cost > 0 {
                if self.repair_cost.load(Ordering::Relaxed) >= 40 {
                    self.set_repair_cost(39);
                }
                self.only_renaming.store(true, Ordering::Relaxed);
            }

            if self.repair_cost.load(Ordering::Relaxed) >= 40 && !has_infinite_materials {
                result = ItemStack::EMPTY.clone();
            }

            if !result.is_empty() {
                let mut base_cost = result.get_repair_cost();
                if base_cost < addition.get_repair_cost() {
                    base_cost = addition.get_repair_cost();
                }

                if naming_cost != price || naming_cost == 0 {
                    base_cost = calculate_increased_repair_cost(base_cost);
                }

                result.set_repair_cost(base_cost);
                set_enchantments(&mut result, enchantments);
            }

            self.inventory.set_stack(2, result);
            self.send_content_updates();
        } else {
            self.inventory.set_stack(2, ItemStack::EMPTY.clone());
            self.set_repair_cost(0);
        }
    }
}

impl ScreenHandler for AnvilScreenHandler {
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
        self.inventory.on_close();
        // Drop inputs from anvil
        for i in 0..2 {
            let stack = self.inventory.remove_stack(i);
            if !stack.is_empty() {
                offer_or_drop_stack(player, stack);
            }
        }
        self.inventory.set_stack(2, ItemStack::EMPTY.clone());
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            stack_left = slot_stack.clone();

            match slot_index.cmp(&2) {
                std::cmp::Ordering::Equal => {
                    // Result slot: from anvil to player
                    if !slot.can_take_items(player) {
                        return ItemStack::EMPTY.clone();
                    }
                    if !self.insert_item(&mut slot_stack, 3, 39, true) {
                        return ItemStack::EMPTY.clone();
                    }
                    slot.on_quick_move_crafted(slot_stack.clone(), stack_left.clone());
                    slot.on_take_item(player, &stack_left);
                    self.create_result(player.has_infinite_materials());
                    self.send_content_updates();
                    return stack_left;
                }
                std::cmp::Ordering::Less => {
                    // Input slots (0, 1): from anvil to player
                    if !self.insert_item(&mut slot_stack, 3, 39, false) {
                        return ItemStack::EMPTY.clone();
                    }
                }
                std::cmp::Ordering::Greater => {
                    // From player to anvil input slots (0..2)
                    if !self.insert_item(&mut slot_stack, 0, 2, false) {
                        return ItemStack::EMPTY.clone();
                    }
                }
            }

            if slot_stack.item_count == stack_left.item_count {
                return ItemStack::EMPTY.clone();
            }

            slot.set_stack_prev(slot_stack.clone(), stack_left.clone());
            slot.on_take_item(player, &slot_stack);
            slot.mark_dirty();
            self.create_result(player.has_infinite_materials());
            self.send_content_updates();
        }

        stack_left
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: pumpkin_protocol::java::server::play::SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        if slot_index == 0 || slot_index == 1 || slot_index == 2 {
            self.create_result(player.has_infinite_materials());
            self.send_content_updates();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_equipment::EntityEquipment;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory, CSetSelectedSlot,
    };
    use pumpkin_world::inventory::SimpleInventory;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicI32;

    struct DummyPlayer {
        pub exp_level: AtomicI32,
        pub creative: bool,
        pub inventory: Arc<PlayerInventory>,
    }

    impl DummyPlayer {
        fn new(exp_level: i32, creative: bool) -> Self {
            Self {
                exp_level: AtomicI32::new(exp_level),
                creative,
                inventory: Arc::new(PlayerInventory::new(
                    Arc::new(Mutex::new(EntityEquipment::new())),
                    Arc::new(rustc_hash::FxHashMap::default()),
                )),
            }
        }
    }

    impl InventoryPlayer for DummyPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) {}

        fn is_creative(&self) -> bool {
            self.creative
        }

        fn has_infinite_materials(&self) -> bool {
            self.creative
        }

        fn experience_level(&self) -> i32 {
            self.exp_level.load(Ordering::Relaxed)
        }

        fn add_experience_levels(&self, levels: i32) {
            self.exp_level.fetch_add(levels, Ordering::Relaxed);
        }

        fn enchantment_seed(&self) -> i32 {
            0
        }

        fn set_enchantment_seed(&self, _seed: i32) {}

        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }

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

        fn enqueue_equipment_change(
            &self,
            _slot: &pumpkin_data::data_component_impl::EquipmentSlot,
            _stack: &ItemStack,
        ) {
        }

        fn award_experience(&self, _amount: i32) {}

        fn increment_stat(
            &self,
            _category: pumpkin_data::statistic::StatisticCategory,
            _stat_id: i32,
            _amount: i32,
        ) {
        }
    }

    #[test]
    fn rename_item() {
        let player = DummyPlayer::new(10, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        anvil_inv.set_stack(0, sword);

        handler.set_item_name("Excalibur", false);

        assert_eq!(handler.get_cost(), 1);
        let result = anvil_inv.get_stack(2);
        assert_eq!(result.get_hover_name(), "Excalibur");
        assert_eq!(result.get_repair_cost(), 0); // Only renaming does not increase repair cost
    }

    #[test]
    fn material_repair() {
        let player = DummyPlayer::new(10, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let mut damaged_sword = ItemStack::new(1, &Item::IRON_SWORD);
        let max_damage = damaged_sword.get_max_damage().unwrap();
        // 250 / 4 = 62 per ingot. 125 damage requires 3 ingots (62 + 62 + 1).
        damaged_sword.set_damage(max_damage / 2);
        anvil_inv.set_stack(0, damaged_sword);

        let ingots = ItemStack::new(4, &Item::IRON_INGOT);
        anvil_inv.set_stack(1, ingots);

        handler.create_result(false);

        assert_eq!(handler.get_cost(), 3);
        let result = anvil_inv.get_stack(2);
        assert_eq!(result.get_damage(), 0);
        assert_eq!(result.get_repair_cost(), 1); // calculateIncreasedRepairCost(0) = 1
        assert_eq!(handler.repair_item_count_cost.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn tool_combine_durability() {
        let player = DummyPlayer::new(10, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let mut sword1 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let max_damage = sword1.get_max_damage().unwrap();
        sword1.set_damage(500);
        anvil_inv.set_stack(0, sword1);

        let mut sword2 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword2.set_damage(500);
        anvil_inv.set_stack(1, sword2);

        handler.create_result(false);

        assert_eq!(handler.get_cost(), 2);
        let result = anvil_inv.get_stack(2);
        // Durability combine: (max - 500) + (max - 500) + max * 12 / 100
        let expected_remaining = (max_damage - 500) + (max_damage - 500) + max_damage * 12 / 100;
        let expected_damage = (max_damage - expected_remaining).max(0);
        assert_eq!(result.get_damage(), expected_damage);
        assert_eq!(result.get_repair_cost(), 1);
    }

    #[test]
    fn enchanted_book_combine() {
        let player = DummyPlayer::new(10, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        anvil_inv.set_stack(0, sword);

        let mut book = ItemStack::new(1, &Item::ENCHANTED_BOOK);
        let stored = StoredEnchantmentsImpl {
            enchantment: Cow::Owned(vec![(&Enchantment::SHARPNESS, 3)]),
        };
        book.set_data_component(stored);
        anvil_inv.set_stack(1, book);

        handler.create_result(false);

        // Sharpness anvil cost = 1. Fee using book = max(1, 1/2) = 1. Price = 1 * 3 = 3.
        assert_eq!(handler.get_cost(), 3);
        let result = anvil_inv.get_stack(2);
        let enchs = get_enchantments_for_crafting(&result);
        assert_eq!(enchs.len(), 1);
        assert_eq!(enchs[0].0.id, Enchantment::SHARPNESS.id);
        assert_eq!(enchs[0].1, 3);
        assert_eq!(result.get_repair_cost(), 1);
    }

    #[test]
    fn enchantment_level_upgrade() {
        let player = DummyPlayer::new(10, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let mut sword1 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let ench1 = EnchantmentsImpl {
            enchantment: Cow::Owned(vec![(&Enchantment::SHARPNESS, 3)]),
        };
        sword1.set_data_component(ench1);
        anvil_inv.set_stack(0, sword1);

        let mut sword2 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        let ench2 = EnchantmentsImpl {
            enchantment: Cow::Owned(vec![(&Enchantment::SHARPNESS, 3)]),
        };
        sword2.set_data_component(ench2);
        anvil_inv.set_stack(1, sword2);

        handler.create_result(false);

        let result = anvil_inv.get_stack(2);
        let enchs = get_enchantments_for_crafting(&result);
        assert_eq!(enchs.len(), 1);
        assert_eq!(enchs[0].0.id, Enchantment::SHARPNESS.id);
        assert_eq!(enchs[0].1, 4);
    }

    #[test]
    fn too_expensive_limit() {
        let player = DummyPlayer::new(50, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let mut sword1 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword1.set_repair_cost(35);
        sword1.set_damage(500);
        anvil_inv.set_stack(0, sword1);

        let mut sword2 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword2.set_repair_cost(35);
        sword2.set_damage(500);
        anvil_inv.set_stack(1, sword2);

        handler.create_result(false);

        // Tax is 35 + 35 = 70. Combining damaged tools adds 2. Total = 72 >= 40.
        assert_eq!(handler.get_cost(), 72);
        assert!(anvil_inv.get_stack(2).is_empty());

        // But in creative mode, result is available
        handler.create_result(true);
        assert!(!anvil_inv.get_stack(2).is_empty());
    }

    #[test]
    fn rename_only_caps_at_39() {
        let player = DummyPlayer::new(50, false);
        let anvil_inv = Arc::new(SimpleInventory::new(3));
        let mut handler = AnvilScreenHandler::new(1, &player.inventory, anvil_inv.clone());

        let mut sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        sword.set_repair_cost(45);
        anvil_inv.set_stack(0, sword);

        handler.set_item_name("Legendary", false);

        // Renaming only is capped at 39 even with high repair cost
        assert_eq!(handler.get_cost(), 39);
        assert!(!anvil_inv.get_stack(2).is_empty());
    }
}
