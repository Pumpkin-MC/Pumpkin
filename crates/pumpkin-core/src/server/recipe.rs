use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
pub use pumpkin_protocol::codec::recipe::DynamicRecipe;

pub struct RecipeManager {
    dynamic_recipes: RwLock<Vec<DynamicRecipe>>,
    /// Bumped on every change, so a sender can tell that the recipes changed while it was sending.
    generation: AtomicU64,
}

impl Default for RecipeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dynamic_recipes: RwLock::new(Vec::new()),
            generation: AtomicU64::new(0),
        }
    }

    pub fn add_recipe(&self, recipe: DynamicRecipe) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.push(recipe);
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn add_recipes(&self, new_recipes: impl IntoIterator<Item = DynamicRecipe>) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.extend(new_recipes);
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn set_recipes(&self, new_recipes: Vec<DynamicRecipe>) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *recipes = new_recipes;
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn clear(&self) {
        let mut recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        recipes.clear();
        self.generation.fetch_add(1, Ordering::Release);
    }

    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn get_dynamic_recipes_internal(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl RecipeProvider for RecipeManager {
    fn get_dynamic_recipes(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}
