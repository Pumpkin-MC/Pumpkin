use std::{
    any::Any,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

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
    screen_handler::{
        DefaultScreenHandlerBehaviour, ScreenHandler, ScreenHandlerListener, ScreenProperty,
    },
    slot::{ArmorSlot, NormalSlot, Slot},
    sync_handler::{AlwaysInSyncTrackedSlot, SyncHandler, TrackedSlot},
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

    fn is_in_hotbar(slot: u8) -> bool {
        slot >= 36 && slot < 45 || slot == 45
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
            Arc::new(Mutex::new(CraftingInventory {
                width: 2,
                height: 2,
            }));

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
    fn window_type(&self) -> WindowType {
        unreachable!()
    }

    fn size(&self) -> usize {
        self.behaviour.slots.len()
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
}
