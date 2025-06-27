use super::{Recipe, RecipeType};

#[derive(Debug, Clone)]
pub struct SmithingTrimRecipe {
    pub recipe_type: RecipeType,
    pub template: String,
    pub base: String,
    pub addition: String,
}

impl SmithingTrimRecipe {
    pub fn new(recipe_type: RecipeType, template: String, base: String, addition: String) -> Self {
        Self {
            recipe_type,
            template,
            base,
            addition,
        }
    }
}

impl Recipe for SmithingTrimRecipe {}
