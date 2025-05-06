pub mod container_click;
pub mod crafting;
pub mod drag_handler;
pub mod entity_equipment;
pub mod equipment_slot;
mod error;
pub mod generic_container_screen_handler;
pub mod player;
pub mod screen_handler;
pub mod slot;
pub mod sync_handler;
pub mod window_property;

use std::sync::Arc;

pub use error::InventoryError;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;
