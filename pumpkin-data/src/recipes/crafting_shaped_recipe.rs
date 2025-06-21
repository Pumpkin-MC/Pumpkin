use std::collections::HashMap;

use super::{Ingredient, Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct CraftingShapedRecipe {
    pub recipe_type: RecipeType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub show_notification: Option<bool>,
    pub pattern: Vec<String>,
    pub key: HashMap<String, Ingredient>,
    pub result: RecipeResult,
}

impl CraftingShapedRecipe {
    pub fn new(
        recipe_type: RecipeType,
        category: Option<String>,
        group: Option<String>,
        show_notification: Option<bool>,
        pattern: Vec<String>,
        key: HashMap<String, Ingredient>,
        result: RecipeResult,
    ) -> Self {
        Self {
            recipe_type,
            category,
            group,
            show_notification,
            pattern,
            key,
            result,
        }
    }
}

impl Recipe for CraftingShapedRecipe {}
