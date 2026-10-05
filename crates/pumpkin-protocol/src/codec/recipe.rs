use pumpkin_data::recipes::RecipeCategoryTypes;

use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;

#[derive(Clone, Debug)]
pub enum OwnedRecipeIngredient {
    Simple(String),
    Tagged(String),
    OneOf(Vec<String>),
}

impl OwnedRecipeIngredient {
    #[must_use]
    pub fn match_item(&self, item: &Item) -> bool {
        match self {
            Self::Simple(id) => {
                let name = item.resource_location();
                *name == *id
            }
            Self::Tagged(tag) => item.is_tagged_with(tag).unwrap_or(false),
            Self::OneOf(ids) => {
                let name = item.resource_location();
                ids.iter().any(|id| *id == *name)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct OwnedRecipeResult {
    pub item_id: String,
    pub count: u8,
    // TODO: Add components/enchantments if needed for the display result
}

#[derive(Clone, Debug)]
pub enum OwnedCraftingRecipe {
    Shaped {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        show_notification: bool,
        key: Vec<(char, OwnedRecipeIngredient)>,
        pattern: Vec<String>,
        result: OwnedRecipeResult,
    },
    Shapeless {
        recipe_id: Option<String>,
        category: RecipeCategoryTypes,
        group: Option<String>,
        ingredients: Vec<OwnedRecipeIngredient>,
        result: OwnedRecipeResult,
    },
}

#[derive(Clone, Debug)]
pub struct OwnedCookingRecipe {
    pub recipe_id: String,
    pub category: RecipeCategoryTypes,
    pub group: Option<String>,
    pub ingredient: OwnedRecipeIngredient,
    pub cooking_time: i32,
    pub experience: f32,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug)]
pub enum OwnedCookingRecipeType {
    Blasting(OwnedCookingRecipe),
    Smelting(OwnedCookingRecipe),
    Smoking(OwnedCookingRecipe),
    CampfireCooking(OwnedCookingRecipe),
}

#[derive(Clone, Debug)]
pub struct OwnedBrewingRecipe {
    pub recipe_id: String,
    pub input_item: String,
    pub input_potion: Option<String>,
    pub reagent: String,
    pub output_item: String,
    pub output_potion: Option<String>,
}

#[derive(Clone, Debug)]
pub enum DynamicRecipe {
    Crafting(OwnedCraftingRecipe),
    Cooking(OwnedCookingRecipeType),
    Brewing(OwnedBrewingRecipe),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::dynamic_tag::register_tag;
    use pumpkin_data::item_registry::ItemRegistration;
    use pumpkin_data::tag::RegistryKey;

    #[test]
    fn ingredients_match_dynamic_items_by_key_and_tag() {
        let gem = Item::register_dynamic(ItemRegistration {
            key: "test_recipe:gem".to_string(),
            components: Vec::new(),
            max_stack_size: None,
        })
        .unwrap();
        register_tag(
            RegistryKey::Item,
            "test_recipe:gems",
            ["test_recipe:gem".to_string()],
        );

        assert!(OwnedRecipeIngredient::Simple("test_recipe:gem".to_string()).match_item(gem));
        assert!(
            !OwnedRecipeIngredient::Simple("test_recipe:gem".to_string())
                .match_item(&Item::DIAMOND)
        );
        assert!(
            OwnedRecipeIngredient::Simple("minecraft:diamond".to_string())
                .match_item(&Item::DIAMOND)
        );
        assert!(
            OwnedRecipeIngredient::OneOf(vec![
                "minecraft:diamond".to_string(),
                "test_recipe:gem".to_string()
            ])
            .match_item(gem)
        );
        assert!(OwnedRecipeIngredient::Tagged("test_recipe:gems".to_string()).match_item(gem));
        assert!(
            !OwnedRecipeIngredient::Tagged("test_recipe:gems".to_string()).match_item(&Item::COAL)
        );
    }
}
