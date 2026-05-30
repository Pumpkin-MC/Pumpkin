use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::recipe::{
    CookingRecipe as WitCookingRecipe, CookingType as WitCookingType, Host as RecipeHost,
    HostRecipeManager, Ingredient as WitIngredient, RecipeManager as WitRecipeManager,
    ShapedRecipe as WitShapedRecipe, ShapelessRecipe as WitShapelessRecipe,
};
use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use wasmtime::component::Resource;

impl RecipeHost for PluginHostState {}

impl HostRecipeManager for PluginHostState {
    async fn register_shaped(
        &mut self,
        _res: Resource<WitRecipeManager>,
        _id: String,
        recipe: WitShapedRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.get_item_stack(&recipe.output)?;
        let result_stack = result_stack.lock().await;

        let owned_recipe = OwnedCraftingRecipe::Shaped {
            category: RecipeCategoryTypes::Misc, // TODO: Allow specifying category
            group: recipe.group,
            show_notification: true,
            key: recipe
                .key
                .into_iter()
                .map(|(k, ing)| (k.chars().next().unwrap_or(' '), to_owned_ingredient(ing)))
                .collect(),
            pattern: recipe.pattern,
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server
            .recipe_manager
            .add_recipe(DynamicRecipe::Crafting(owned_recipe))
            .await;
        Ok(())
    }

    async fn register_shapeless(
        &mut self,
        _res: Resource<WitRecipeManager>,
        _id: String,
        recipe: WitShapelessRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.get_item_stack(&recipe.output)?;
        let result_stack = result_stack.lock().await;

        let owned_recipe = OwnedCraftingRecipe::Shapeless {
            category: RecipeCategoryTypes::Misc,
            group: recipe.group,
            ingredients: recipe
                .ingredients
                .into_iter()
                .map(to_owned_ingredient)
                .collect(),
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server
            .recipe_manager
            .add_recipe(DynamicRecipe::Crafting(owned_recipe))
            .await;
        Ok(())
    }

    async fn register_cooking(
        &mut self,
        _res: Resource<WitRecipeManager>,
        id: String,
        station_type: WitCookingType,
        recipe: WitCookingRecipe,
    ) -> wasmtime::Result<()> {
        let result_stack = self.get_item_stack(&recipe.output)?;
        let result_stack = result_stack.lock().await;

        let owned_cooking = OwnedCookingRecipe {
            recipe_id: id,
            category: RecipeCategoryTypes::Misc,
            group: recipe.group,
            ingredient: to_owned_ingredient(recipe.ingredient),
            cooking_time: recipe.cooking_time as i32,
            experience: recipe.experience,
            result: OwnedRecipeResult {
                item_id: result_stack.item.registry_key.to_string(),
                count: result_stack.item_count,
            },
        };

        let dynamic_recipe = match station_type {
            WitCookingType::Smelting => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(owned_cooking))
            }
            WitCookingType::Blasting => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(owned_cooking))
            }
            WitCookingType::Smoking => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(owned_cooking))
            }
            WitCookingType::Campfire => {
                DynamicRecipe::Cooking(OwnedCookingRecipeType::CampfireCooking(owned_cooking))
            }
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server.recipe_manager.add_recipe(dynamic_recipe).await;
        Ok(())
    }

    async fn drop(&mut self, _rep: Resource<WitRecipeManager>) -> wasmtime::Result<()> {
        Ok(())
    }
}

fn to_owned_ingredient(ing: WitIngredient) -> OwnedRecipeIngredient {
    match ing {
        WitIngredient::Item(id) => OwnedRecipeIngredient::Simple(id),
        WitIngredient::Tag(tag) => OwnedRecipeIngredient::Tagged(tag),
    }
}
