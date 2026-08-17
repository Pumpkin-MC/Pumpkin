use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
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
                let name = format!("minecraft:{}", item.registry_key);
                name == *id
            }
            Self::Tagged(tag) => item.is_tagged_with(tag).unwrap_or(false),
            Self::OneOf(ids) => {
                let name = format!("minecraft:{}", item.registry_key);
                ids.contains(&name)
            }
        }
    }
}

#[derive(Clone)]
pub struct OwnedRecipeResult {
    pub item_id: String,
    pub count: u8,
    /// Result data components parsed from the recipe's `components` field.
    pub components: Vec<(DataComponent, Box<dyn DataComponentImpl>)>,
}

impl std::fmt::Debug for OwnedRecipeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let component_names: Vec<&str> =
            self.components.iter().map(|(id, _)| id.to_name()).collect();
        f.debug_struct("OwnedRecipeResult")
            .field("item_id", &self.item_id)
            .field("count", &self.count)
            .field("components", &component_names)
            .finish()
    }
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
pub enum DynamicRecipe {
    Crafting(OwnedCraftingRecipe),
    Cooking(OwnedCookingRecipeType),
    // TODO(datapack parity): Add SmithingTransform, SmithingTrim, Stonecutting variants
    // and their protocol serialization. These require new Owned* structs.
}
