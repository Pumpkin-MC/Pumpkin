use std::collections::HashMap;

use super::{Ingredient, Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct CraftingShapelessRecipe {
    pub recipe_type: RecipeType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredients: Vec<Ingredient>,
    pub result: RecipeResult,
}

impl CraftingShapelessRecipe {
    pub fn new(
        recipe_type: RecipeType,
        category: Option<String>,
        group: Option<String>,
        ingredients: Vec<Ingredient>,
        result: RecipeResult,
    ) -> Self {
        Self {
            recipe_type,
            category,
            group,
            ingredients,
            result,
        }
    }
}

impl Recipe for CraftingShapelessRecipe {}
