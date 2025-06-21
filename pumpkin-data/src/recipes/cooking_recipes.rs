use super::{Ingredient, Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct CookingRecipe {
    pub recipe_type: RecipeType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub ingredient: Ingredient,
    pub cookingtime: i32,
    pub result: RecipeResult,
    pub experience: f32,
}

impl CookingRecipe {
    pub fn new(
        recipe_type: RecipeType,
        category: Option<String>,
        group: Option<String>,
        ingredient: Ingredient,
        cookingtime: i32,
        result: RecipeResult,
        experience: f32,
    ) -> Self {
        Self {
            recipe_type,
            category,
            group,
            ingredient,
            cookingtime,
            result,
            experience,
        }
    }
}

impl Recipe for CookingRecipe {}
