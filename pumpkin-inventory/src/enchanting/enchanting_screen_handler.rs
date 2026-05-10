use std::any::Any;
use std::sync::Arc;

use pumpkin_data::Enchantment;
use pumpkin_data::data_component_impl::EnchantableImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::tag::{Enchantment as EnchantmentTag, Taggable};
use pumpkin_util::random::RandomImpl;
use pumpkin_world::inventory::Inventory;

use crate::{
    player::player_inventory::PlayerInventory,
    screen_handler::{
        InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour,
        ScreenHandlerFuture, offer_or_drop_stack,
    },
    slot::NormalSlot,
    window_property::{EnchantmentTable, WindowProperty},
};

pub struct EnchantingTableScreenHandler {
    pub inventory: Arc<dyn Inventory>,
    behaviour: ScreenHandlerBehaviour,
    pub level_requirements: [i32; 3],
    pub enchantment_id: [i32; 3],
    pub enchantment_level: [i32; 3],
    pub enchantment_seed: i32,
    pub bookshelf_count: i32,
}

impl EnchantingTableScreenHandler {
    pub fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        inventory: &Arc<dyn Inventory>,
        enchantment_seed: i32,
        bookshelf_count: i32,
    ) -> Self {
        let mut handler = Self {
            inventory: inventory.clone(),
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Enchantment)),
            level_requirements: [0; 3],
            enchantment_id: [-1; 3],
            enchantment_level: [-1; 3],
            enchantment_seed,
            bookshelf_count,
        };

        // Enchanting slots: 0 is item, 1 is lapis
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 0)));
        handler.add_slot(Arc::new(NormalSlot::new(inventory.clone(), 1)));

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        handler.add_player_slots(&player_inventory);

        handler
    }

    pub async fn update_enchantments(&mut self, _player: &dyn InventoryPlayer) {
        let item = self.inventory.get_stack(0).await;
        let item = item.lock().await;

        if item.is_empty() || item.has_enchantments() {
            for i in 0..3 {
                self.level_requirements[i] = 0;
                self.enchantment_id[i] = -1;
                self.enchantment_level[i] = -1;
            }
        } else {
            let enchantability = item
                .get_data_component::<EnchantableImpl>()
                .map_or(0, |e| e.value);

            if enchantability <= 0 {
                for i in 0..3 {
                    self.level_requirements[i] = 0;
                    self.enchantment_id[i] = -1;
                    self.enchantment_level[i] = -1;
                }
            } else {
                let mut random = pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(
                    self.enchantment_seed as u64,
                );

                for i in 0..3 {
                    let level = self.calculate_level_requirement(&mut random, i, enchantability);
                    self.level_requirements[i] = level;
                }

                for i in 0..3 {
                    if self.level_requirements[i] > 0 {
                        let enchantments = Self::get_enchantment_list(
                            &mut random,
                            &item,
                            i,
                            self.level_requirements[i],
                        );
                        if let Some(first) = enchantments.first() {
                            self.enchantment_id[i] = first.0.id as i32;
                            self.enchantment_level[i] = first.1;
                        } else {
                            self.enchantment_id[i] = -1;
                            self.enchantment_level[i] = -1;
                        }
                    } else {
                        self.enchantment_id[i] = -1;
                        self.enchantment_level[i] = -1;
                    }
                }
            }
        }
        self.send_property_updates().await;
    }

    fn calculate_level_requirement(
        &self,
        random: &mut pumpkin_util::random::xoroshiro128::Xoroshiro,
        slot: usize,
        _enchantability: i32,
    ) -> i32 {
        let b = self.bookshelf_count;
        let level = random.next_bounded_i32(8) + 1 + (b >> 1) + random.next_bounded_i32(b + 1);

        match slot {
            0 => (level / 3).max(1),
            1 => (level * 2 / 3 + 7).max(1),
            2 => level.max(b * 2).max(1),
            _ => 0,
        }
    }

    fn get_enchantment_list(
        random: &mut pumpkin_util::random::xoroshiro128::Xoroshiro,
        item: &ItemStack,
        _slot: usize,
        level: i32,
    ) -> Vec<(&'static Enchantment, i32)> {
        let enchantability = item
            .get_data_component::<EnchantableImpl>()
            .map_or(0, |e| e.value);
        let mut enchant_level = level
            + 1
            + random.next_bounded_i32(enchantability / 4 + 1)
            + random.next_bounded_i32(enchantability / 4 + 1);
        let bonus = (random.next_f32() + random.next_f32() - 1.0) * 0.15;
        enchant_level = (enchant_level as f32 * (1.0 + bonus)).round() as i32;
        enchant_level = enchant_level.max(1);

        let mut available = Vec::new();
        for enchant in Enchantment::all() {
            if enchant.has_tag(&EnchantmentTag::MINECRAFT_IN_ENCHANTING_TABLE)
                && enchant.can_enchant(item.item)
            {
                for l in (1..=enchant.max_level).rev() {
                    if enchant_level >= enchant.min_cost.calculate(l)
                        && enchant_level <= enchant.max_cost.calculate(l)
                    {
                        available.push((*enchant, l));
                        break;
                    }
                }
            }
        }

        if available.is_empty() {
            return Vec::new();
        }

        let total_weight: i32 = available.iter().map(|(e, _)| e.weight).sum();
        if total_weight <= 0 {
            return Vec::new();
        }

        let mut weight = random.next_bounded_i32(total_weight);
        let mut selected = None;
        for (e, l) in &available {
            weight -= e.weight;
            if weight < 0 {
                selected = Some((*e, *l));
                break;
            }
        }

        let mut result = Vec::new();
        if let Some(s) = selected {
            result.push(s);

            // Add more?
            let mut current_level = enchant_level;
            while random.next_bounded_i32(50) <= (current_level + 1) / 2 {
                available.retain(|(e, _)| {
                    for (se, _) in &result {
                        if !e.are_compatible(se) {
                            return false;
                        }
                    }
                    true
                });

                if available.is_empty() {
                    break;
                }

                let total_weight: i32 = available.iter().map(|(e, _)| e.weight).sum();
                let mut weight = random.next_bounded_i32(total_weight);
                for (e, l) in &available {
                    weight -= e.weight;
                    if weight < 0 {
                        result.push((*e, *l));
                        break;
                    }
                }
                current_level /= 2;
            }
        }

        result
    }

    async fn send_property_updates(&self) {
        if let Some(sync_handler) = self.behaviour.sync_handler.as_ref() {
            for i in 0..3 {
                let (id, val) = WindowProperty::new(
                    EnchantmentTable::LevelRequirement { slot: i as u8 },
                    self.level_requirements[i] as i16,
                )
                .into_tuple();
                sync_handler
                    .update_property(&self.behaviour, id as i32, val as i32)
                    .await;

                let (id, val) = WindowProperty::new(
                    EnchantmentTable::EnchantmentId { slot: i as u8 },
                    self.enchantment_id[i] as i16,
                )
                .into_tuple();
                sync_handler
                    .update_property(&self.behaviour, id as i32, val as i32)
                    .await;

                let (id, val) = WindowProperty::new(
                    EnchantmentTable::EnchantmentLevel { slot: i as u8 },
                    self.enchantment_level[i] as i16,
                )
                .into_tuple();
                sync_handler
                    .update_property(&self.behaviour, id as i32, val as i32)
                    .await;
            }

            let (id, val) = WindowProperty::new(
                EnchantmentTable::EnchantmentSeed,
                (self.enchantment_seed & 0xFFFF) as i16,
            )
            .into_tuple();
            sync_handler
                .update_property(&self.behaviour, id as i32, val as i32)
                .await;
        }
    }
}

impl ScreenHandler for EnchantingTableScreenHandler {
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

    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.inventory.on_close().await;
            // Return items to player
            for i in 0..2 {
                let stack = self.inventory.remove_stack(i).await;
                if !stack.is_empty() {
                    offer_or_drop_stack(player, stack).await;
                }
            }
        })
    }

    fn on_button_click<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        id: i32,
    ) -> ScreenHandlerFuture<'a, bool> {
        Box::pin(async move {
            if !(0..3).contains(&id) {
                return false;
            }

            let level_req = self.level_requirements[id as usize];
            if player.experience_level() < level_req && !player.is_creative() {
                return false;
            }

            let lapis_slot = self.inventory.get_stack(1).await;
            let mut lapis_stack = lapis_slot.lock().await;
            let lapis_cost = (id + 1) as u8;

            if !player.is_creative()
                && (lapis_stack.is_empty() || lapis_stack.item_count < lapis_cost)
            {
                return false;
            }

            // Perform enchantment
            let item_slot = self.inventory.get_stack(0).await;
            let mut item_stack = item_slot.lock().await;

            if item_stack.is_empty() || item_stack.has_enchantments() {
                return false;
            }

            let mut random = pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(
                self.enchantment_seed as u64,
            );
            let enchantments =
                Self::get_enchantment_list(&mut random, &item_stack, id as usize, level_req);

            if enchantments.is_empty() {
                return false;
            }

            if !player.is_creative() {
                player.add_experience_levels(-(id + 1)).await;
                lapis_stack.decrement(lapis_cost);
            }

            for (enchant, level) in enchantments {
                item_stack.add_enchantment(enchant, level as u16);
            }

            // CRITICAL FIX: Drop locks *before* calling `update_enchantments`
            // Otherwise, update_enchantments will try to lock slot 0 again and deadlock!
            drop(item_stack);
            drop(lapis_stack);

            // Update seed
            player.set_enchantment_seed(rand::random()).await;
            self.enchantment_seed = player.enchantment_seed();

            self.update_enchantments(player).await;
            self.send_content_updates().await;

            true
        })
    }

    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer, // FIX: Changed _player to player
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let mut stack_left = ItemStack::EMPTY.clone();
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let slot_stack_guard = slot_stack_lock.lock().await;
                stack_left = slot_stack_guard.clone();
                drop(slot_stack_guard);

                let mut slot_stack_mut = slot_stack_lock.lock().await;

                if slot_index < 2 {
                    // From enchanting to player
                    if !self
                        .insert_item(
                            &mut slot_stack_mut,
                            2,
                            self.get_behaviour().slots.len() as i32,
                            true,
                        )
                        .await
                    {
                        return ItemStack::EMPTY.clone();
                    }
                } else {
                    // From player to enchanting
                    // Lapis check
                    if slot_stack_mut.item == &Item::LAPIS_LAZULI {
                        if !self.insert_item(&mut slot_stack_mut, 1, 2, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack_mut, 0, 1, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                }

                if slot_stack_mut.is_empty() {
                    drop(slot_stack_mut);
                    slot.set_stack(ItemStack::EMPTY.clone()).await;
                } else {
                    drop(slot_stack_mut);
                    slot.mark_dirty().await;
                }

                // CRITICAL FIX: Ensure the client is notified when shift-clicking items into the slots
                self.update_enchantments(player).await;
            }

            stack_left
        })
    }

    fn on_slot_click<'a>(
        &'a mut self,
        slot_index: i32,
        button: i32,
        action_type: pumpkin_protocol::java::server::play::SlotActionType,
        player: &'a dyn InventoryPlayer,
    ) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.internal_on_slot_click(slot_index, button, action_type, player)
                .await;
            if slot_index == 0 || slot_index == 1 {
                self.update_enchantments(player).await;
            }
        })
    }
}
