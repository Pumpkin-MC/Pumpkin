use super::{Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct CraftingTransmuteRecipe {
    pub recipe_type: RecipeType,
    pub category: Option<String>,
    pub group: Option<String>,
    pub input: String,
    pub material: String,
    pub result: RecipeResult,
}

impl CraftingTransmuteRecipe {
    pub fn new(
        recipe_type: RecipeType,
        category: Option<String>,
        group: Option<String>,
        input: String,
        material: String,
        result: RecipeResult,
    ) -> Self {
        Self {
            recipe_type,
            category,
            group,
            input,
            material,
            result,
        }
    }
}

impl Recipe for CraftingTransmuteRecipe {}
