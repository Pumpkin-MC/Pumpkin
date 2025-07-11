use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;

use crate::crafting::crafting_inventory::CraftingInventory;
use crate::crafting::crafting_screen_handler::CraftingScreenHandler;
use crate::crafting::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::equipment_slot::EquipmentSlot;
use crate::screen_handler::{InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour};
use crate::slot::{ArmorSlot, NormalSlot, Slot};

use super::player_inventory::PlayerInventory;

pub struct PlayerScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl RecipeFinderScreenHandler for PlayerScreenHandler {}

impl CraftingScreenHandler<CraftingInventory> for PlayerScreenHandler {}

// TODO: Fully implement this
impl PlayerScreenHandler {
    const EQUIPMENT_SLOT_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];

    pub fn is_in_hotbar(slot: u8) -> bool {
        (36..=45).contains(&slot)
    }

    pub async fn get_slot(&self, slot: usize) -> Arc<dyn Slot> {
        self.behaviour.slots[slot].clone()
    }

    pub async fn new(
        player_inventory: &Arc<PlayerInventory>,
        window_type: Option<WindowType>,
        sync_id: u8,
    ) -> Self {
        let crafting_inventory: Arc<dyn RecipeInputInventory> =
            Arc::new(CraftingInventory::new(2, 2));

        let mut player_screen_handler = PlayerScreenHandler {
            behaviour: ScreenHandlerBehaviour::new(sync_id, window_type),
            crafting_inventory: crafting_inventory.clone(),
        };

        player_screen_handler
            .add_recipe_slots(crafting_inventory)
            .await;

        for i in 0..4 {
            player_screen_handler.add_slot(Arc::new(ArmorSlot::new(
                player_inventory.clone(),
                39 - i,
                Self::EQUIPMENT_SLOT_ORDER[i].clone(),
            )));
        }

        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();

        player_screen_handler.add_player_slots(&player_inventory);

        // Offhand
        // TODO: public void setStack(ItemStack stack, ItemStack previousStack) { owner.onEquipStack(EquipmentSlot.OFFHAND, previousStack, stack);
        player_screen_handler.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), 40)));

        player_screen_handler
    }
}

#[async_trait]
impl ScreenHandler for PlayerScreenHandler {
    async fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player).await;
        //TODO: this.craftingResultInventory.clear();
        self.drop_inventory(player, self.crafting_inventory.clone())
            .await;
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }

    /// Do quick move (Shift + Click) for the given slot index.
    ///
    /// Returns the moved stack if successful, or `ItemStack::EMPTY` if nothing changed.
    async fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();

        // TODO: Equippable component

        if slot.has_stack().await {
            let slot_stack = slot.get_stack().await;
            let mut slot_stack = slot_stack.lock().await;
            let stack_prev = *slot_stack;

            #[allow(clippy::if_same_then_else)]
            if slot_index == 0 {
                // From crafting result slot
                if !self.insert_item(&mut slot_stack, 9, 45, true).await {
                    return ItemStack::EMPTY;
                }
            } else if (1..5).contains(&slot_index) {
                // From craft ingredient slots
                if !self.insert_item(&mut slot_stack, 9, 45, false).await {
                    return ItemStack::EMPTY;
                }
            } else if (5..9).contains(&slot_index) {
                // From armour slots
                if !self.insert_item(&mut slot_stack, 9, 45, false).await {
                    return ItemStack::EMPTY;
                }
            } else if (9..36).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 36, 45, false).await {
                    return ItemStack::EMPTY;
                }
            } else if (36..45).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 9, 36, false).await {
                    return ItemStack::EMPTY;
                }
            } else if !self.insert_item(&mut slot_stack, 9, 45, false).await {
                return ItemStack::EMPTY;
            }

            let stack = *slot_stack;
            drop(slot_stack); // release the lock before calling other methods
            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY, stack_prev).await;
            } else {
                slot.mark_dirty().await;
            }

            if stack.item_count == stack_prev.item_count {
                return ItemStack::EMPTY;
            }

            slot.on_take_item(player, &stack).await;

            if slot_index == 0 {
                // From crafting result slot
                // Notify the result slot to refill
                slot.on_quick_move_crafted(stack, stack_prev).await;
                // For crafting result slot, drop any remaining items
                if !stack.is_empty() {
                    player.drop_item(stack, false).await;
                }
            }

            return stack_prev;
        }

        // Nothing changed
        ItemStack::EMPTY
    }
}
