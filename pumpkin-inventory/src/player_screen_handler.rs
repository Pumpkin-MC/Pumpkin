use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{equipment_slot::EquipmentSlot, inventory::Inventory, screen_handler::ScreenHandler};

pub struct PlayerScreenHandler {}

impl PlayerScreenHandler {
    const EQUIPMENT_SLOT_ORDER: [EquipmentSlot; 4] = [
        EquipmentSlot::HEAD,
        EquipmentSlot::CHEST,
        EquipmentSlot::LEGS,
        EquipmentSlot::FEET,
    ];
}
