use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    crafting::{CraftingScreenHandler, RecipeFinderScreenHandler},
    equipment_slot::EquipmentSlot,
    inventory::Inventory,
    player_inventory::PlayerInventory,
    screen_handler::ScreenHandler,
};

pub struct PlayerScreenHandler {}

impl PlayerScreenHandler {
    const EQUIPMENT_SLOT_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];

    //pub fn new(player_inventory: Arc<Mutex<PlayerInventory>>) -> Self {

    //}
}

impl RecipeFinderScreenHandler for PlayerScreenHandler {}
