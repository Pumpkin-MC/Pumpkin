use std::any::Any;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{EnchantmentsImpl, MaxDamageImpl, StoredEnchantmentsImpl};
use pumpkin_data::enchantment::Enchantment;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::Enchantment as EnchantmentTag;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::java::server::play::SlotActionType;
use pumpkin_world::inventory::Inventory;

use crate::{
    anvil::anvil_screen_handler::calculate_increased_repair_cost,
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, offer_or_drop_stack},
    slot::Slot,
};

pub struct GrindstoneInputSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl GrindstoneInputSlot {
    #[must_use]
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for GrindstoneInputSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }

    fn get_index(&self) -> usize {
        self.index
    }

    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }

    fn can_insert(&self, stack: &ItemStack) -> bool {
        stack.is_damageable() || has_any_enchantments(stack)
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
}

pub struct GrindstoneResultSlot {
    pub inventory: Arc<dyn Inventory>,
    pub index: usize,
    pub id: AtomicU8,
}

impl GrindstoneResultSlot {
    #[must_use]
    pub fn new(inventory: Arc<dyn Inventory>, index: usize) -> Self {
        Self {
            inventory,
            index,
            id: AtomicU8::new(0),
        }
    }
}

impl Slot for GrindstoneResultSlot {
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

    fn on_take_item(&self, player: &dyn InventoryPlayer, _stack: &ItemStack) {
        let input = self
            .inventory
            .get_stack(GrindstoneScreenHandler::INPUT_SLOT);
        let additional = self
            .inventory
            .get_stack(GrindstoneScreenHandler::ADDITIONAL_SLOT);
        let xp_amount = GrindstoneScreenHandler::calculate_experience_amount(&input, &additional);

        player.use_grindstone(xp_amount);

        self.inventory.set_stack(
            GrindstoneScreenHandler::INPUT_SLOT,
            ItemStack::EMPTY.clone(),
        );
        self.inventory.set_stack(
            GrindstoneScreenHandler::ADDITIONAL_SLOT,
            ItemStack::EMPTY.clone(),
        );
        self.mark_dirty();
    }

    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
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

#[must_use]
pub fn has_any_enchantments(stack: &ItemStack) -> bool {
    !get_enchantments_for_crafting(stack).is_empty()
}

pub fn set_enchantments(result: &mut ItemStack, enchantments: Vec<(&'static Enchantment, i32)>) {
    if result.is_empty() {
        return;
    }
    if enchantments.is_empty() {
        result.patch.retain(|(id, _)| {
            *id != DataComponent::Enchantments && *id != DataComponent::StoredEnchantments
        });
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

pub struct GrindstoneScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
}

impl GrindstoneScreenHandler {
    pub const MAX_NAME_LENGTH: usize = 35;
    pub const INPUT_SLOT: usize = 0;
    pub const ADDITIONAL_SLOT: usize = 1;
    pub const RESULT_SLOT: usize = 2;
    pub const INV_SLOT_START: i32 = 3;
    pub const INV_SLOT_END: i32 = 30;
    pub const USE_ROW_SLOT_START: i32 = 30;
    pub const USE_ROW_SLOT_END: i32 = 39;

    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: Arc<dyn Inventory>,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Grindstone)),
        };

        // Slot 0: Input slot
        handler.add_slot(Arc::new(GrindstoneInputSlot::new(
            inventory.clone(),
            Self::INPUT_SLOT,
        )));
        // Slot 1: Additional slot
        handler.add_slot(Arc::new(GrindstoneInputSlot::new(
            inventory.clone(),
            Self::ADDITIONAL_SLOT,
        )));
        // Slot 2: Result slot
        handler.add_slot(Arc::new(GrindstoneResultSlot::new(
            inventory,
            Self::RESULT_SLOT,
        )));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub fn create_result(&mut self) {
        let input = self.inventory.get_stack(Self::INPUT_SLOT);
        let additional = self.inventory.get_stack(Self::ADDITIONAL_SLOT);
        let result = Self::compute_result(&input, &additional);
        self.inventory.set_stack(Self::RESULT_SLOT, result);
    }

    #[must_use]
    pub fn compute_result(input: &ItemStack, additional: &ItemStack) -> ItemStack {
        let has_an_item = !input.is_empty() || !additional.is_empty();
        if !has_an_item {
            return ItemStack::EMPTY.clone();
        }

        if input.item_count <= 1 && additional.item_count <= 1 {
            let has_both_items = !input.is_empty() && !additional.is_empty();
            if has_both_items {
                Self::merge_items(input, additional)
            } else {
                let item = if input.is_empty() { additional } else { input };
                if has_any_enchantments(item) {
                    Self::remove_non_curses_from(item.clone())
                } else {
                    ItemStack::EMPTY.clone()
                }
            }
        } else {
            ItemStack::EMPTY.clone()
        }
    }

    #[must_use]
    pub fn merge_items(input: &ItemStack, additional: &ItemStack) -> ItemStack {
        if input.item != additional.item {
            return ItemStack::EMPTY.clone();
        }

        let max_damage1 = input.get_max_damage().unwrap_or(0);
        let max_damage2 = additional.get_max_damage().unwrap_or(0);
        let durability = max_damage1.max(max_damage2);
        let remaining1 = max_damage1 - input.get_damage();
        let remaining2 = max_damage2 - additional.get_damage();
        let remaining = remaining1 + remaining2 + durability * 5 / 100;
        let count = if input.is_damageable() {
            1
        } else {
            if input.get_max_stack_size() < 2 || !input.are_equal(additional) {
                return ItemStack::EMPTY.clone();
            }
            2
        };

        let mut new_item = input.copy_with_count(count);
        if new_item.is_damageable() {
            new_item.set_data_component(MaxDamageImpl {
                max_damage: durability,
            });
            new_item.set_damage((durability - remaining).max(0));
        }

        Self::merge_enchants_from(&mut new_item, additional);
        Self::remove_non_curses_from(new_item)
    }

    pub fn merge_enchants_from(target: &mut ItemStack, source: &ItemStack) {
        let mut target_enchantments = get_enchantments_for_crafting(target);
        let source_enchantments = get_enchantments_for_crafting(source);

        for (enchant, level) in source_enchantments {
            let is_curse = enchant.has_tag(&EnchantmentTag::MINECRAFT_CURSE);
            if let Some(target_entry) = target_enchantments.iter_mut().find(|(e, _)| *e == enchant)
            {
                if !is_curse || target_entry.1 == 0 {
                    target_entry.1 = target_entry.1.max(level);
                }
            } else {
                target_enchantments.push((enchant, level));
            }
        }

        set_enchantments(target, target_enchantments);
    }

    #[must_use]
    pub fn remove_non_curses_from(mut item: ItemStack) -> ItemStack {
        let enchantments = get_enchantments_for_crafting(&item);
        let curses: Vec<(&'static Enchantment, i32)> = enchantments
            .into_iter()
            .filter(|(ench, _)| ench.has_tag(&EnchantmentTag::MINECRAFT_CURSE))
            .collect();

        let curses_count = curses.len();

        if item.item == &Item::ENCHANTED_BOOK && curses_count == 0 {
            item.item = &Item::BOOK;
        }

        set_enchantments(&mut item, curses);

        let mut repair_cost = 0;
        for _ in 0..curses_count {
            repair_cost = calculate_increased_repair_cost(repair_cost);
        }

        item.set_repair_cost(repair_cost);
        item
    }

    #[must_use]
    pub fn calculate_experience_amount(input: &ItemStack, additional: &ItemStack) -> i32 {
        let mut amount = 0;
        amount += Self::get_experience_from_item(input);
        amount += Self::get_experience_from_item(additional);

        if amount > 0 {
            let half_amount = ((f64::from(amount)) / 2.0).ceil() as i32;
            let random_bonus = if half_amount > 0 {
                (rand::random::<u32>() as usize % half_amount as usize) as i32
            } else {
                0
            };
            half_amount + random_bonus
        } else {
            0
        }
    }

    #[must_use]
    pub fn get_experience_from_item(item: &ItemStack) -> i32 {
        let mut amount = 0;
        let enchantments = get_enchantments_for_crafting(item);

        for (enchant, lvl) in enchantments {
            if !enchant.has_tag(&EnchantmentTag::MINECRAFT_CURSE) {
                amount += enchant.min_cost.calculate(lvl);
            }
        }

        amount
    }
}

impl ScreenHandler for GrindstoneScreenHandler {
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
        for i in 0..2 {
            let stack = self.inventory.remove_stack(i);
            if !stack.is_empty() {
                offer_or_drop_stack(player, stack);
            }
        }
        self.inventory
            .set_stack(Self::RESULT_SLOT, ItemStack::EMPTY.clone());
    }

    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let mut stack_left = ItemStack::EMPTY.clone();
        let slot = self.get_behaviour().slots.get(slot_index as usize).cloned();

        if let Some(slot) = slot
            && slot.has_stack()
        {
            let mut slot_stack = slot.get_stack();
            stack_left = slot_stack.clone();

            let input = self.inventory.get_stack(Self::INPUT_SLOT);
            let additional = self.inventory.get_stack(Self::ADDITIONAL_SLOT);

            if slot_index as usize == Self::RESULT_SLOT {
                if !self.insert_item(
                    &mut slot_stack,
                    Self::INV_SLOT_START,
                    Self::USE_ROW_SLOT_END,
                    true,
                ) {
                    return ItemStack::EMPTY.clone();
                }

                slot.on_quick_move_crafted(slot_stack.clone(), stack_left.clone());
                slot.on_take_item(player, &stack_left);
                self.create_result();
                self.send_content_updates();
                return stack_left;
            } else if slot_index < Self::INV_SLOT_START {
                // Input slots (0, 1): from grindstone to player inventory (3..39, reversed = false)
                if !self.insert_item(
                    &mut slot_stack,
                    Self::INV_SLOT_START,
                    Self::USE_ROW_SLOT_END,
                    false,
                ) {
                    return ItemStack::EMPTY.clone();
                }
            } else {
                // Player inventory (3..39)
                if !input.is_empty() && !additional.is_empty() {
                    if slot_index < Self::USE_ROW_SLOT_START {
                        if !self.insert_item(
                            &mut slot_stack,
                            Self::USE_ROW_SLOT_START,
                            Self::USE_ROW_SLOT_END,
                            false,
                        ) {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(
                        &mut slot_stack,
                        Self::INV_SLOT_START,
                        Self::INV_SLOT_END,
                        false,
                    ) {
                        return ItemStack::EMPTY.clone();
                    }
                } else if !self.insert_item(
                    &mut slot_stack,
                    Self::INPUT_SLOT as i32,
                    Self::RESULT_SLOT as i32,
                    false,
                ) {
                    return ItemStack::EMPTY.clone();
                }
            }

            if slot_stack.item_count == stack_left.item_count {
                return ItemStack::EMPTY.clone();
            }

            if slot_stack.is_empty() {
                slot.set_stack(ItemStack::EMPTY.clone());
            } else {
                slot.set_stack(slot_stack.clone());
            }

            slot.on_take_item(player, &slot_stack);
            slot.mark_dirty();
            self.create_result();
            self.send_content_updates();
        }

        stack_left
    }

    fn on_slot_click(
        &mut self,
        slot_index: i32,
        button: i32,
        action_type: SlotActionType,
        player: &dyn InventoryPlayer,
    ) {
        self.internal_on_slot_click(slot_index, button, action_type, player);
        if (slot_index as usize) <= Self::RESULT_SLOT {
            self.create_result();
            self.send_content_updates();
        }
    }
}
