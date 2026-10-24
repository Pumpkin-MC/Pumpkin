//! Plugin inventory and container management utilities.
//!
//! This module provides a unified API for inspecting, modifying, and interacting
//! with player inventories, ender chests, custom GUIs, and container block entities.
//!
//! # Examples
//!
//! ## Inspecting and Modifying a Player's Inventory
//! ```rust,ignore
//! use pumpkin_plugin_api::{Player, ItemStack};
//!
//! fn equip_player(player: &Player) {
//!     let inv = player.get_inventory();
//!     inv.set_helmet(Some(ItemStack::new("minecraft:diamond_helmet", 1)));
//!     inv.set_boots(Some(ItemStack::new("minecraft:diamond_boots", 1)));
//!     
//!     let storage = inv.as_inventory();
//!     storage.set_item(0, Some(ItemStack::new("minecraft:diamond_sword", 1)));
//! }
//! ```
//!
//! ## Interacting with Custom GUIs
//! ```rust,ignore
//! use pumpkin_plugin_api::gui::Gui;
//!
//! fn setup_gui(gui: &Gui) {
//!     let inv = gui.get_inventory();
//!     inv.clear();
//! }
//! ```

pub use crate::wit::pumpkin::plugin::inventory::{Inventory, PlayerInventory};
