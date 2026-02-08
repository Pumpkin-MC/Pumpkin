pub mod anvil;
pub mod beacon;
pub mod brewing_stand;
pub mod cartography_table;
pub mod container_click;
pub mod crafting;
pub mod double;
pub mod drag_handler;
pub mod enchanting_table;
pub mod entity_equipment;
mod error;
pub mod furnace_like;
pub mod generic_container_screen_handler;
pub mod grindstone;
pub mod lectern;
pub mod loom;
pub mod player;
pub mod screen_handler;
pub mod slot;
pub mod smithing;
pub mod stonecutter;
pub mod sync_handler;
pub mod window_property;

use std::collections::HashMap;

pub use error::InventoryError;
use pumpkin_data::data_component_impl::EquipmentSlot;

use crate::player::player_inventory::PlayerInventory;

#[must_use]
pub fn build_equipment_slots() -> HashMap<usize, EquipmentSlot> {
    let mut equipment_slots = HashMap::new();
    equipment_slots.insert(
        EquipmentSlot::FEET.get_offset_entity_slot_id(PlayerInventory::MAIN_SIZE as i32) as usize,
        EquipmentSlot::FEET,
    );
    equipment_slots.insert(
        EquipmentSlot::LEGS.get_offset_entity_slot_id(PlayerInventory::MAIN_SIZE as i32) as usize,
        EquipmentSlot::LEGS,
    );
    equipment_slots.insert(
        EquipmentSlot::CHEST.get_offset_entity_slot_id(PlayerInventory::MAIN_SIZE as i32) as usize,
        EquipmentSlot::CHEST,
    );
    equipment_slots.insert(
        EquipmentSlot::HEAD.get_offset_entity_slot_id(PlayerInventory::MAIN_SIZE as i32) as usize,
        EquipmentSlot::HEAD,
    );

    equipment_slots.insert(PlayerInventory::OFF_HAND_SLOT, EquipmentSlot::OFF_HAND);
    equipment_slots.insert(PlayerInventory::OFF_HAND_SLOT, EquipmentSlot::OFF_HAND);
    equipment_slots
}
