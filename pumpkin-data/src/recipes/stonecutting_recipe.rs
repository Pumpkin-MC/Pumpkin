use super::{Ingredient, Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct StoneCuttingRecipe {
    pub recipe_type: RecipeType,
    pub ingredient: Ingredient,
    pub result: RecipeResult,
}

impl StoneCuttingRecipe {
    pub fn new(recipe_type: RecipeType, ingredient: Ingredient, result: RecipeResult) -> Self {
        Self {
            recipe_type,
            ingredient,
            result,
        }
    }
}

impl Recipe for StoneCuttingRecipe {}
