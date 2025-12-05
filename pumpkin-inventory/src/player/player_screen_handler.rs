use super::player_inventory::PlayerInventory;
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::crafting::crafting_screen_handler::CraftingScreenHandler;
use crate::crafting::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
};
use crate::slot::{ArmorSlot, NormalSlot, Slot};
use pumpkin_data::data_component_impl::{EquipmentSlot, EquipmentType, EquippableImpl};
use pumpkin_data::screen::WindowType;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use std::any::Any;
use std::sync::Arc;

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

impl ScreenHandler for PlayerScreenHandler {
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            //TODO: this.craftingResultInventory.clear();
            self.drop_inventory(player, self.crafting_inventory.clone())
                .await;
        })
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
    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();

            // TODO: Equippable component

            if slot.has_stack().await {
                let slot_stack_lock = slot.get_stack().await;
                let mut slot_stack = slot_stack_lock.lock().await;
                let stack_prev = slot_stack.clone();

                let equipment_slot = slot_stack
                    .get_data_component::<EquippableImpl>()
                    .map_or(&EquipmentSlot::MAIN_HAND, |equippable| equippable.slot);

                // Quick move logic
                #[allow(clippy::if_same_then_else)]
                let success = if slot_index == 0 {
                    // From crafting result slot (0) -> Player Inventory (9-45, from end)
                    self.insert_item(&mut slot_stack, 9, 45, true).await
                } else if (1..5).contains(&slot_index) {
                    // From craft ingredient slots (1-4) -> Player Inventory (9-45, from start)
                    self.insert_item(&mut slot_stack, 9, 45, false).await
                } else if (5..9).contains(&slot_index) {
                    // From armour slots (5-8) -> Player Inventory (9-45, from start)
                    let result = self.insert_item(&mut slot_stack, 9, 45, false).await;

                    if result {
                        player
                            .enqueue_equipment_change(equipment_slot, ItemStack::EMPTY)
                            .await;
                    }
                    result
                } else if equipment_slot.slot_type() == EquipmentType::HumanoidArmor
                    && self
                        .get_slot((8 - equipment_slot.get_entity_slot_id()) as usize)
                        .await
                        .get_cloned_stack()
                        .await
                        .is_empty()
                {
                    // Into empty armour slots (5-8)
                    let index = 8 - equipment_slot.get_entity_slot_id();
                    let result = self
                        .insert_item(&mut slot_stack, index, index + 1, false)
                        .await;

                    if result {
                        player
                            .enqueue_equipment_change(equipment_slot, &stack_prev)
                            .await;
                    }
                    result
                } else if matches!(equipment_slot, EquipmentSlot::OffHand(_))
                    && slot_index != 45
                    && self.get_slot(45).await.get_cloned_stack().await.is_empty()
                {
                    // Into empty offhand slot (45)
                    let index = 45;
                    self.insert_item(&mut slot_stack, index, index + 1, false)
                        .await
                } else if (9..36).contains(&slot_index) {
                    // From main inventory (9-35) -> Hotbar (36-44)
                    self.insert_item(&mut slot_stack, 36, 45, false).await
                } else if (36..45).contains(&slot_index) {
                    // From hotbar (36-44) -> Main inventory (9-35)
                    self.insert_item(&mut slot_stack, 9, 36, false).await
                } else {
                    // Fallback to moving into the player inventory area
                    self.insert_item(&mut slot_stack, 9, 45, false).await
                };

                if !success {
                    return ItemStack::EMPTY.clone();
                }

                let stack = slot_stack.clone();
                drop(slot_stack); // release the lock before calling other methods

                if stack.is_empty() {
                    slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone())
                        .await;
                } else {
                    slot.mark_dirty().await;
                }

                if stack.item_count == stack_prev.item_count {
                    return ItemStack::EMPTY.clone();
                }

                slot.on_take_item(player, &stack).await;

                if slot_index == 0 {
                    // From crafting result slot (0)
                    // Notify the result slot to refill
                    slot.on_quick_move_crafted(stack.clone(), stack_prev.clone())
                        .await;
                    // For crafting result slot, drop any remaining items
                    if !stack.is_empty() {
                        player.drop_item(stack, false).await;
                    }
                }

                return stack_prev;
            }

            // Nothing changed
            ItemStack::EMPTY.clone()
        })
    }
}
