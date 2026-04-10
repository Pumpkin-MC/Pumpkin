//! Recipe-related types and traits.
//!
//! This module defines the interfaces for recipe handling in crafting systems.
//! It provides traits for screen handlers that can find recipes and inventories
//! that can serve as recipe input.
//!
//! # Recipe System
//!
//! The recipe system involves:
//! - [`RecipeFinderScreenHandler`] - Screen handlers that can find matching recipes
//! - [`RecipeInputInventory`] - Inventories that provide crafting input
//! - [`RecipeMatcher`] - Helper for matching items to recipes
//! - [`RecipeFinder`] - Helper for finding recipes

use pumpkin_world::inventory::Inventory;

/// Helper struct for matching recipe ingredients.
// RecipeMatcher.java
pub struct RecipeMatcher;

/// Helper struct for finding recipes.
// RecipeFinder.java
pub struct RecipeFinder;

/// Trait for screen handlers that can find crafting recipes.
///
/// Screen handlers implementing this trait can search for recipes
/// that match the current input inventory state.
// AbstractRecipeScreenHandle.java
pub trait RecipeFinderScreenHandler {}

/// Trait for inventories that serve as recipe input.
///
/// Crafting grids implement this trait to provide their dimensions
/// and item access for recipe matching.
pub trait RecipeInputInventory: Inventory {
    /// Gets the width of the crafting grid.
    fn get_width(&self) -> usize;

    /// Gets the height of the crafting grid.
    fn get_height(&self) -> usize;

    // TODO: Additional methods for recipe input handling
    // fn get_held_stacks(), Get a lock on the inventory instead
    // createRecipeInput
    // createPositionedRecipeInput
}
