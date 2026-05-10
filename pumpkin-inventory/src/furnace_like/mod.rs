//! Furnace-like containers module.
//!
//! This module handles screen handlers for furnace-like blocks:
//! - Furnace
//! - Smoker
//! - Blast Furnace
//!
//! These containers share the same slot layout and behavior:
//! - Slot 0: Input (item to smelt/cook)
//! - Slot 1: Fuel (coal, charcoal, etc.)
//! - Slot 2: Output (smelted/cooked result)
//!
//! # Properties
//!
/// Furnace-like containers track four properties:
/// - Property 0: Fire icon (fuel remaining)
/// - Property 1: Maximum fuel burn time
/// - Property 2: Progress arrow (smelting progress)
/// - Property 3: Maximum progress (typically 200 ticks)
pub mod furnace_like_screen_handler;
pub mod furnace_like_slot;
