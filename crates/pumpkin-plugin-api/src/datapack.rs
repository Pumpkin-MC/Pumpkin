//! Plugin datapack management and querying utilities.
//!
//! This module provides interfaces to inspect, query, enable, disable, and reload
//! datapacks on the server.
//!
//! # Examples
//!
//! ## Listing Datapacks
//! ```rust,ignore
//! use pumpkin_plugin_api::Server;
//!
//! fn log_datapacks(server: &Server) {
//!     let manager = server.get_datapack_manager();
//!     for pack in manager.list_all_packs() {
//!         println!("Datapack {}: enabled = {}", pack.name, pack.is_enabled);
//!     }
//! }
//! ```
//!
//! ## Enabling and Reloading Datapacks
//! ```rust,ignore
//! use pumpkin_plugin_api::{Server, datapack::EnablePosition};
//!
//! fn enable_custom_pack(server: &Server) -> Result<(), String> {
//!     let manager = server.get_datapack_manager();
//!     manager.enable_pack("my_custom_pack", EnablePosition::Last)?;
//!     Ok(())
//! }
//! ```

pub use crate::wit::pumpkin::plugin::datapack::{DatapackInfo, DatapackManager, EnablePosition};
