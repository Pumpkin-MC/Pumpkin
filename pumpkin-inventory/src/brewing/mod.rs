//! Brewing module.
//!
//! This module handles the brewing stand mechanics:
//! - [`BrewingStandScreenHandler`] - Screen handler for the brewing stand UI
//!
//! The brewing stand allows players to brew potions by combining water bottles
//! with various ingredients.

pub mod brewing_screen_handler;

pub use brewing_screen_handler::create_brewing;
