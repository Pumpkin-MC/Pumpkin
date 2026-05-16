//! Pumpkin inventory system.
//!
//! This crate provides the inventory management system for the Pumpkin Minecraft server.
//! It handles player inventories, container screens (like chests, furnaces, crafting tables),
//! slot interactions, item dragging, and inventory synchronization between server and client.
//!
//! # Core Concepts
//!
//! - [`Inventory`] - A trait representing any item storage (player inventory, chest, furnace, etc.)
//! - [`ScreenHandler`] - Manages the UI screen for a container, handling slot layout and interactions
//! - [`Slot`] - Represents a single slot in an inventory that can hold items
//! - [`PlayerInventory`] - The player's 36-slot main inventory plus equipment slots
//! - [`SyncHandler`] - Synchronizes inventory state between server and client
//!
//! # Module Structure
//!
//! - [`player`] - Player inventory and screen handler implementations
//! - [`crafting`] - Crafting table and inventory crafting mechanics
//! - [`furnace_like`] - Furnace, smoker, and blast furnace screen handlers
//! - [`brewing`] - Brewing stand handling
//! - [`slot`] - Slot trait and implementations (normal slots, armor slots)
//! - [`container_click`] - Mouse and keyboard click handling
//! - [`drag_handler`] - Item dragging across multiple slots
//! - [`sync_handler`] - Client-server inventory synchronization
//! - [`window_property`] - Container UI properties (furnace progress, enchantment levels, etc.)
//!
//! [`Inventory`]: pumpkin_world::inventory::Inventory
//! [`ScreenHandler`]: screen_handler::ScreenHandler
//! [`Slot`]: slot::Slot
//! [`PlayerInventory`]: PlayerInventory
//! [`SyncHandler`]: sync_handler::SyncHandler

pub mod anvil;
pub mod beacon_screen_handler;
pub mod brewing;
pub mod container_click;
pub mod crafting;
pub mod double;
pub mod drag_handler;
pub mod enchanting;
pub mod entity_equipment;
mod error;
pub mod furnace_like;
pub mod generic_container_screen_handler;
pub mod gui_builder;
pub mod merchant;
pub mod player;
pub mod screen_handler;
pub mod slot;
pub mod stonecutter_screen_handler;
pub mod sync_handler;
pub mod window_property;

use std::collections::HashMap;

pub use error::InventoryError;
use pumpkin_data::data_component_impl::EquipmentSlot;

use crate::player::player_inventory::PlayerInventory;

/// Builds a map of slot indices to equipment slots for the player's inventory.
///
/// This creates the mapping between UI slot indices and equipment slots
/// (head, chest, legs, feet, off-hand) used by the player screen handler.
///
/// # Returns
/// A `HashMap` where keys are slot indices and values are the corresponding [`EquipmentSlot`]s.
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
