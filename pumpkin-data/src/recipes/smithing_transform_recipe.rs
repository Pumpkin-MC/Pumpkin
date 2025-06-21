use super::{Recipe, RecipeResult, RecipeType};

#[derive(Debug, Clone)]
pub struct SmithingTransformRecipe {
    pub recipe_type: RecipeType,
    pub template: String,
    pub base: String,
    pub addition: String,
    pub result: RecipeResult,
}

impl SmithingTransformRecipe {
    pub fn new(
        recipe_type: RecipeType,
        template: String,
        base: String,
        addition: String,
        result: RecipeResult,
    ) -> Self {
        Self {
            recipe_type,
            template,
            base,
            addition,
            result,
        }
    }
}

impl Recipe for SmithingTransformRecipe {}
