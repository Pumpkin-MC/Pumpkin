fn is_valid_for_slot(&self, stack: &ItemStack) -> bool {
    if stack.is_empty() {
        return true;
    }

    let item = stack.item;

    let matches_static_role = |template: &RecipeIngredientTypes,
                               base: &RecipeIngredientTypes,
                               addition: &RecipeIngredientTypes|
     -> bool {
        match self.slot_type {
            0 => template.match_item(item),
            1 => base.match_item(item),
            2 => addition.match_item(item),
            _ => false,
        }
    };

    if RECIPES_SMITHING_TRIM
        .iter()
        .any(|r| matches_static_role(&r.template, &r.base, &r.addition))
    {
        return true;
    }
    if RECIPES_SMITHING_TRANSFORM
        .iter()
        .any(|r| matches_static_role(&r.template, &r.base, &r.addition))
    {
        return true;
    }

    if let Some(provider) = &self.recipe_provider {
        let matches_dynamic_role = |template: &OwnedRecipeIngredient,
                                    base: &OwnedRecipeIngredient,
                                    addition: &OwnedRecipeIngredient|
         -> bool {
            match self.slot_type {
                0 => template.match_item(item),
                1 => base.match_item(item),
                2 => addition.match_item(item),
                _ => false,
            }
        };

        for dynamic in provider.get_dynamic_recipes() {
            let ok = match dynamic {
                DynamicRecipe::SmithingTransform(recipe) => {
                    matches_dynamic_role(&recipe.template, &recipe.base, &recipe.addition)
                }
                DynamicRecipe::SmithingTrim(recipe) => {
                    matches_dynamic_role(&recipe.template, &recipe.base, &recipe.addition)
                }
                _ => false,
            };

            if ok {
                return true;
            }
        }
    }

    false
}
