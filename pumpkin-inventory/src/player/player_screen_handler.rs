use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use pumpkin_data::screen::WindowType;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::{
    crafting::{
        crafting_inventory::CraftingInventory,
        crafting_screen_handler::CraftingScreenHandler,
        recipies::{RecipeFinderScreenHandler, RecipeInputInventory},
    },
    equipment_slot::EquipmentSlot,
    inventory::Inventory,
    screen_handler::{DefaultScreenHandlerBehaviour, InventoryPlayer, ScreenHandler},
    slot::{ArmorSlot, NormalSlot, Slot},
};

use super::player_inventory::PlayerInventory;

pub struct PlayerScreenHandler {
    behaviour: DefaultScreenHandlerBehaviour,
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
        (36..45).contains(&slot) || slot == 45
    }

    pub async fn get_slot(&self, slot: usize) -> Arc<dyn Slot> {
        self.behaviour.slots[slot].clone()
    }

    pub async fn set_previous_tracked_slot(&mut self, slot: usize, stack: ItemStack) {
        self.behaviour.previous_tracked_stacks[slot] = stack;
    }

    pub async fn new(
        player_inventory: &Arc<Mutex<PlayerInventory>>,
        window_type: Option<WindowType>,
        sync_id: u8,
    ) -> Self {
        let mut player_screen_handler = PlayerScreenHandler {
            behaviour: DefaultScreenHandlerBehaviour::new(sync_id, window_type),
        };

        let crafting_inventory: Arc<Mutex<dyn RecipeInputInventory>> =
            Arc::new(Mutex::new(CraftingInventory::new(2, 2)));

        player_screen_handler
            .add_result_slot(&crafting_inventory)
            .await;

        player_screen_handler
            .add_input_slots(&crafting_inventory)
            .await;

        for i in 0..4 {
            player_screen_handler.add_slot(Arc::new(ArmorSlot::new(
                player_inventory.clone(),
                39 - i,
                Self::EQUIPMENT_SLOT_ORDER[i].clone(),
            )));
        }

        let player_inventory: Arc<Mutex<dyn Inventory>> = player_inventory.clone();

        player_screen_handler.add_player_slots(&player_inventory);

        // Offhand
        // TODO: public void setStack(ItemStack stack, ItemStack previousStack) { owner.onEquipStack(EquipmentSlot.OFFHAND, previousStack, stack);
        player_screen_handler.add_slot(Arc::new(NormalSlot::new(player_inventory.clone(), 40)));

        player_screen_handler
    }
}

#[async_trait]
impl ScreenHandler for PlayerScreenHandler {
    fn window_type(&self) -> Option<WindowType> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn sync_id(&self) -> u8 {
        self.behaviour.sync_id
    }

    fn get_behaviour(&self) -> &DefaultScreenHandlerBehaviour {
        &self.behaviour
    }

    fn get_behaviour_mut(&mut self) -> &mut DefaultScreenHandlerBehaviour {
        &mut self.behaviour
    }

    async fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        todo!()
    }
}
