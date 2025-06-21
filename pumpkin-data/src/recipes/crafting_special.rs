use super::{Recipe, RecipeType};

#[derive(Debug, Clone)]
pub struct CraftingSpecial {
    pub recipe_type: RecipeType,
    pub category: String,
}

impl CraftingSpecial {
    pub fn new(recipe_type: RecipeType, category: String) -> Self {
        Self {
            recipe_type,
            category,
        }
    }
}

impl Recipe for CraftingSpecial {}
