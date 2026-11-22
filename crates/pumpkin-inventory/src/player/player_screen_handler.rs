//! Player inventory screen handler.
//!
//! This module handles the player's inventory screen (opened with E key).
//! It includes:
//! - The 2x2 crafting grid (inventory crafting)
//! - Armor slots (head, chest, legs, feet)
//! - Main inventory (27 slots)
//! - Hotbar (9 slots)
//! - Offhand slot
//!
//! # Slot Layout
//!
//! The player screen handler uses the following slot indices:
//! - 0: Crafting result
//! - 1-4: Crafting grid (2x2)
//! - 5-8: Armor slots (head, chest, legs, feet)
//! - 9-35: Main inventory
//! - 36-44: Hotbar
//! - 45: Offhand

use super::player_inventory::PlayerInventory;
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::crafting::crafting_screen_handler::CraftingScreenHandler;
use crate::crafting::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::inventory::Inventory;
use crate::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use crate::slot::{ArmorSlot, NormalSlot, Slot};
use pumpkin_data::data_component_impl::{EquipmentSlot, EquipmentType, EquippableImpl};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use std::any::Any;
use std::sync::Arc;

/// Screen handler for the player's inventory.
///
/// Manages the player's inventory UI including crafting, armor, and
/// the main inventory. This is the default screen shown when pressing E.
pub struct PlayerScreenHandler {
    /// Core screen handler behavior (slots, sync ID, listeners).
    behaviour: ScreenHandlerBehaviour,
    /// The 2x2 crafting grid inventory.
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl RecipeFinderScreenHandler for PlayerScreenHandler {}

impl CraftingScreenHandler<CraftingInventory> for PlayerScreenHandler {}

// TODO: Fully implement this
impl PlayerScreenHandler {
    /// Equipment slot order for armor display.
    const EQUIPMENT_SLOT_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];

    /// Checks if a slot index is in the hotbar.
    ///
    /// Hotbar slots are 36-44 in the protocol (0-indexed 36-44).
    #[must_use]
    pub fn is_in_hotbar(slot: u8) -> bool {
        (36..=45).contains(&slot)
    }

    /// Gets a slot by its index.
    pub fn get_slot(&self, slot: usize) -> Arc<dyn Slot> {
        self.behaviour.slots[slot].clone()
    }

    /// Creates a new player screen handler.
    ///
    /// # Arguments
    /// - `player_inventory` - The player's inventory
    /// - `window_type` - The window type (usually None for player inventory)
    /// - `sync_id` - The synchronization ID
    pub fn new(
        player_inventory: &Arc<PlayerInventory>,
        window_type: Option<WindowType>,
        sync_id: u8,
        provider: Option<Arc<dyn crate::crafting::recipe_provider::RecipeProvider>>,
    ) -> Self {
        let crafting_inventory: Arc<dyn RecipeInputInventory> =
            Arc::new(CraftingInventory::new(2, 2));

        let mut player_screen_handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, window_type),
            crafting_inventory: crafting_inventory.clone(),
        };

        player_screen_handler.add_recipe_slots(crafting_inventory, provider);

        // Add armor slots (head, chest, legs, feet)
        for i in 0..4 {
            player_screen_handler.add_slot(Arc::new(ArmorSlot::new(
                player_inventory.clone(),
                39 - i,
                Self::EQUIPMENT_SLOT_ORDER[i].clone(),
            )));
        }

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();

        // Add main inventory and hotbar
        player_screen_handler.add_player_slots(&player_inventory);

        // Offhand slot (index 40 in player inventory, 45 in screen handler)
        // TODO: onEquipStack callback for offhand
        player_screen_handler.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), 40)));

        player_screen_handler
    }
}

impl ScreenHandler for PlayerScreenHandler {
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
        //TODO: this.craftingResultInventory.clear();
        self.drop_inventory(player, self.crafting_inventory.clone());
    }

    /// Performs quick move (shift-click) for the given slot.
    ///
    /// The quick move logic depends on the source slot:
    /// - Crafting result (0) -> Player inventory (from end)
    /// - Crafting grid (1-4) -> Player inventory (from start)
    /// - Armor slots (5-8) -> Player inventory, unequips
    /// - Armor items -> Armor slots if empty
    /// - Offhand items -> Offhand slot if empty
    /// - Main inventory (9-35) -> Hotbar
    /// - Hotbar (36-44) -> Main inventory
    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        if slot.has_stack() && slot.can_take_items(player) {
            let mut slot_stack = slot.get_stack();
            let stack_prev = slot_stack.clone();

            let equipment_slot = slot_stack
                .get_data_component::<EquippableImpl>()
                .map_or(&EquipmentSlot::MAIN_HAND, |equippable| equippable.slot);

            // Quick move logic based on source slot
            let success = if slot_index == 0 {
                // From crafting result slot (0) -> Player Inventory (9-45, from end)
                self.insert_item(&mut slot_stack, 9, 45, true)
            } else if (1..5).contains(&slot_index) {
                // From craft ingredient slots (1-4) -> Player Inventory (9-45, from start)
                self.insert_item(&mut slot_stack, 9, 45, false)
            } else if (5..9).contains(&slot_index) {
                // From armour slots (5-8) -> Player Inventory (9-45, from start)
                let result = self.insert_item(&mut slot_stack, 9, 45, false);

                if result {
                    player.enqueue_equipment_change(equipment_slot, ItemStack::EMPTY);
                }
                result
            } else if equipment_slot.slot_type() == EquipmentType::HumanoidArmor
                && self
                    .get_slot((8 - equipment_slot.get_entity_slot_id()) as usize)
                    .get_cloned_stack()
                    .is_empty()
                && self
                    .get_slot((8 - equipment_slot.get_entity_slot_id()) as usize)
                    .can_insert(&slot_stack)
            {
                // Into empty armour slots (5-8)
                let index = 8 - equipment_slot.get_entity_slot_id();
                let result = self.insert_item(&mut slot_stack, index, index + 1, false);

                if result {
                    let equipped = self.get_slot(index as usize).get_cloned_stack();
                    player.enqueue_equipment_change(equipment_slot, &equipped);
                }
                result
            } else if matches!(equipment_slot, EquipmentSlot::OffHand(_))
                && slot_index != 45
                && self.get_slot(45).get_cloned_stack().is_empty()
            {
                // Into empty offhand slot (45)
                let index = 45;
                let result = self.insert_item(&mut slot_stack, index, index + 1, false);

                if result {
                    player.enqueue_equipment_change(&EquipmentSlot::OFF_HAND, &stack_prev);
                }
                result
            } else if slot_index == 45 {
                // From offhand (45) -> Player Inventory (9-44)
                let result = self.insert_item(&mut slot_stack, 9, 45, false);

                if result {
                    // The remainder stays in the offhand, an empty one clears it
                    let remaining = if slot_stack.is_empty() {
                        ItemStack::EMPTY
                    } else {
                        &slot_stack
                    };
                    player.enqueue_equipment_change(&EquipmentSlot::OFF_HAND, remaining);
                }
                result
            } else if (9..36).contains(&slot_index) {
                // From main inventory (9-35) -> Hotbar (36-44)
                self.insert_item(&mut slot_stack, 36, 45, false)
            } else if (36..45).contains(&slot_index) {
                // From hotbar (36-44) -> Main inventory (9-35)
                self.insert_item(&mut slot_stack, 9, 36, false)
            } else {
                // Fallback to moving into the player inventory area
                self.insert_item(&mut slot_stack, 9, 45, false)
            };

            if !success {
                return ItemStack::EMPTY.clone();
            }

            let stack = slot_stack.clone();

            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone());
            } else {
                slot.set_stack(stack.clone());
            }

            if stack.item_count == stack_prev.item_count {
                return ItemStack::EMPTY.clone();
            }

            let mut taken_stack = stack_prev.clone();
            taken_stack.set_count(stack_prev.item_count - stack.item_count);
            slot.on_take_item(player, &taken_stack);

            if slot_index == 0 {
                // From crafting result slot (0)
                // Notify the result slot to refill
                slot.on_quick_move_crafted(stack.clone(), stack_prev.clone());
                // For crafting result slot, drop any remaining items
                if !stack.is_empty() {
                    player.drop_item(stack, false);
                }
            }

            return stack_prev;
        }

        // Nothing changed
        ItemStack::EMPTY.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_equipment::EntityEquipment;
    use pumpkin_data::Enchantment;
    use pumpkin_data::data_component_impl::{EnchantmentsImpl, IDSet, IdOr};
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_data::sound::Sound;
    use pumpkin_data::statistic::StatisticCategory;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory, CSetSelectedSlot,
    };
    use pumpkin_protocol::java::server::play::SlotActionType;
    use std::borrow::Cow;
    use std::sync::Mutex;

    struct TestPlayer {
        inventory: Arc<PlayerInventory>,
        creative: bool,
    }

    impl InventoryPlayer for TestPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn drop_item(&self, _item: ItemStack, _retain_ownership: bool) {}
        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }
        fn has_infinite_materials(&self) -> bool {
            self.creative
        }
        fn is_creative(&self) -> bool {
            self.creative
        }
        fn experience_level(&self) -> i32 {
            0
        }
        fn add_experience_levels(&self, _levels: i32) {}
        fn enchantment_seed(&self) -> i32 {
            0
        }
        fn set_enchantment_seed(&self, _seed: i32) {}
        fn enqueue_inventory_packet(&self, _: &CSetContainerContent, _: Option<WindowType>) {}
        fn enqueue_slot_packet(&self, _: &CSetContainerSlot, _: Option<WindowType>, _: usize) {}
        fn enqueue_cursor_packet(&self, _: &CSetCursorItem) {}
        fn enqueue_property_packet(&self, _: &CSetContainerProperty, _: Option<WindowType>) {}
        fn enqueue_slot_set_packet(&self, _: &CSetPlayerInventory) {}
        fn enqueue_set_held_item_packet(&self, _: &CSetSelectedSlot) {}
        fn enqueue_equipment_change(&self, _: &EquipmentSlot, _: &ItemStack) {}
        fn award_experience(&self, _amount: i32) {}
        fn increment_stat(&self, _: StatisticCategory, _: i32, _: i32) {}
        fn play_block_sound(&self, _: Sound, _: f32) {}
    }

    fn make_handler(creative: bool) -> (PlayerScreenHandler, TestPlayer) {
        let inv = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(crate::build_equipment_slots()),
        ));
        let handler = PlayerScreenHandler::new(&inv, None, 0, None);
        let player = TestPlayer {
            inventory: inv,
            creative,
        };
        (handler, player)
    }

    #[test]
    fn binding_curse_blocks_non_creative_unequip_and_ctrl_q_hang() {
        let (mut handler, survival_player) = make_handler(false);
        let mut cursed_helmet = ItemStack::new(1, &Item::DIAMOND_HELMET);
        let binding = Enchantment::from_name("minecraft:binding_curse").unwrap();
        cursed_helmet.set_data_component(EnchantmentsImpl {
            enchantment: Cow::Owned(vec![(binding, 1)]),
        });
        handler.get_slot(5).set_stack(cursed_helmet);

        // Pickup, QuickMove, and Throw (Ctrl+Q) must all leave the cursed helmet in slot 5
        handler.on_slot_click(5, 0, SlotActionType::Pickup, &survival_player);
        assert_eq!(
            handler.get_slot(5).get_stack().item.id,
            Item::DIAMOND_HELMET.id
        );

        handler.on_slot_click(5, 0, SlotActionType::QuickMove, &survival_player);
        assert_eq!(
            handler.get_slot(5).get_stack().item.id,
            Item::DIAMOND_HELMET.id
        );

        handler.on_slot_click(5, 1, SlotActionType::Throw, &survival_player);
        assert_eq!(
            handler.get_slot(5).get_stack().item.id,
            Item::DIAMOND_HELMET.id
        );

        // Creative player CAN unequip it
        let creative_player = TestPlayer {
            inventory: survival_player.inventory,
            creative: true,
        };
        handler.on_slot_click(5, 0, SlotActionType::QuickMove, &creative_player);
        assert!(handler.get_slot(5).get_stack().is_empty());
    }

    #[test]
    fn allowed_entities_blocks_non_player_equippables_and_drag_clamps_max_stack() {
        let (mut handler, player) = make_handler(false);
        let mut wolf_helmet = ItemStack::new(1, &Item::STICK);
        wolf_helmet.set_data_component(EquippableImpl {
            slot: &EquipmentSlot::HEAD,
            equip_sound: IdOr::Id(Sound::ItemArmorEquipGeneric),
            asset_id: None,
            camera_overlay: None,
            allowed_entities: Some(IDSet::IDs(Cow::Borrowed(&[&EntityType::WOLF]))),
            dispensable: true,
            swappable: true,
            damage_on_hurt: true,
            equip_on_interact: false,
            can_be_sheared: false,
            shearing_sound: IdOr::Id(Sound::ItemShearsSnip),
        });
        handler.get_slot(9).set_stack(wolf_helmet);
        handler.on_slot_click(9, 0, SlotActionType::QuickMove, &player);
        assert!(handler.get_slot(5).get_stack().is_empty());

        handler
            .get_slot(10)
            .set_stack(ItemStack::new(1, &Item::DIAMOND_HELMET));
        handler.on_slot_click(10, 0, SlotActionType::QuickMove, &player);
        assert_eq!(
            handler.get_slot(5).get_stack().item.id,
            Item::DIAMOND_HELMET.id
        );

        // Left-dragging 20 ender pearls (max_stack_size = 16) across 1 empty slot vs 2 empty slots:
        // Put 20 pearls on cursor (cheated test stack) and drag across slots 10 and 11 (10 each <= 16),
        // then put 2 unstackable swords on cursor and drag across slots 12 and 13 -> 1 each!
        *handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            ItemStack::new(4, &Item::DIAMOND_SWORD);
        handler.on_slot_click(-999, 0, SlotActionType::QuickCraft, &player); // start left drag
        handler.on_slot_click(12, 1, SlotActionType::QuickCraft, &player);
        handler.on_slot_click(13, 1, SlotActionType::QuickCraft, &player);
        handler.on_slot_click(-999, 2, SlotActionType::QuickCraft, &player); // end left drag
        assert_eq!(handler.get_slot(12).get_stack().item_count, 1);
        assert_eq!(handler.get_slot(13).get_stack().item_count, 1);
    }
}
