use pumpkin_data::recipes::RecipeCategoryTypes;
use serde::Serialize;

use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;

#[derive(Clone, Debug, Serialize)]
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
                let name = format!("minecraft:{}", item.registry_key);
                name == *id
            }
            Self::Tagged(tag) => item
                .is_tagged_with(tag)
                .expect("Crafting recipe used invalid tag"),
            Self::OneOf(ids) => {
                let name = format!("minecraft:{}", item.registry_key);
                ids.contains(&name)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OwnedRecipeResult {
    pub item_id: String,
    pub count: u8,
    // TODO: Add components/enchantments if needed for the display result
}

#[derive(Clone, Debug, Serialize)]
pub enum OwnedCraftingRecipe {
    Shaped {
        category: RecipeCategoryTypes,
        group: Option<String>,
        show_notification: bool,
        key: Vec<(char, OwnedRecipeIngredient)>,
        pattern: Vec<String>,
        result: OwnedRecipeResult,
    },
    Shapeless {
        category: RecipeCategoryTypes,
        group: Option<String>,
        ingredients: Vec<OwnedRecipeIngredient>,
        result: OwnedRecipeResult,
    },
}

#[derive(Clone, Debug, Serialize)]
pub struct OwnedCookingRecipe {
    pub recipe_id: String,
    pub category: RecipeCategoryTypes,
    pub group: Option<String>,
    pub ingredient: OwnedRecipeIngredient,
    pub cooking_time: i32,
    pub experience: f32,
    pub result: OwnedRecipeResult,
}

#[derive(Clone, Debug, Serialize)]
pub enum OwnedCookingRecipeType {
    Blasting(OwnedCookingRecipe),
    Smelting(OwnedCookingRecipe),
    Smoking(OwnedCookingRecipe),
    CampfireCooking(OwnedCookingRecipe),
}

#[derive(Clone, Debug, Serialize)]
pub enum DynamicRecipe {
    Crafting(OwnedCraftingRecipe),
    Cooking(OwnedCookingRecipeType),
}
