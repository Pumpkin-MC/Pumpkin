use super::{Recipe, RecipeType};

#[derive(Debug, Clone)]
pub struct CraftingDecoratedPotRecipe {
    pub recipe_type: RecipeType,
    pub category: String,
}

impl CraftingDecoratedPotRecipe {
    pub fn new(recipe_type: RecipeType, category: String) -> Self {
        Self {
            recipe_type,
            category,
        }
    }
}

impl Recipe for CraftingDecoratedPotRecipe {}
