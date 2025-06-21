pub mod cooking_recipes;
pub mod crafting_decorated_pot_recipe;
pub mod crafting_shaped_recipe;
pub mod crafting_shapeless_recipe;
pub mod crafting_special;
pub mod crafting_transmute_recipe;
pub mod smithing_transform_recipe;
pub mod smithing_trim_recipe;
pub mod stonecutting_recipe;

use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    str::FromStr,
    sync::{Arc, OnceLock},
    vec,
};

use cooking_recipes::CookingRecipe;
use crafting_decorated_pot_recipe::CraftingDecoratedPotRecipe;
use crafting_shaped_recipe::CraftingShapedRecipe;
use crafting_shapeless_recipe::CraftingShapelessRecipe;
use crafting_special::CraftingSpecial;
use crafting_transmute_recipe::CraftingTransmuteRecipe;
use serde::Deserialize;
use smithing_transform_recipe::SmithingTransformRecipe;
use smithing_trim_recipe::SmithingTrimRecipe;
use stonecutting_recipe::StoneCuttingRecipe;

use crate::item::Item;

#[derive(Deserialize, Debug, Clone)]
pub struct RecipeResult {
    #[serde(rename = "id")]
    pub key: String,
    pub count: Option<i32>,

    //TODO: The potion_content
    //Like:
    //"components": {
    // "minecraft:suspicious_stew_effects": [
    //   {
    //     "id": "minecraft:nausea",
    //     "duration": 140
    //   }
    // ]
    #[serde(skip)]
    pub components: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Ingredient {
    Single(String),
    Multi(Vec<String>),
}

impl Ingredient {
    fn strip_minecraft_prefix(i: &Ingredient) -> Ingredient {
        match i {
            Ingredient::Single(ingredient) => match ingredient.strip_prefix("minecraft:") {
                Some(s) => Ingredient::Single(s.to_string()),
                None => Ingredient::Single(ingredient.to_string()),
            },
            Ingredient::Multi(ingredients) => Ingredient::Multi(
                ingredients
                    .iter()
                    .map(|s| match s.strip_prefix("minecraft:") {
                        Some(s) => s.to_string(),
                        None => s.to_string(),
                    })
                    .collect::<Vec<String>>(),
            ),
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
struct CommonFields {
    category: Option<String>,
    group: Option<String>,
    #[serde(rename = "show_notification")]
    show_notification: Option<bool>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
struct CookingFields {
    ingredient: Option<Ingredient>,
    cookingtime: Option<i32>,
    experience: Option<f32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
struct CraftingFields {
    pattern: Option<Ingredient>,
    key: Option<HashMap<String, Ingredient>>,
    ingredients: Option<Vec<Ingredient>>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
struct SmithingFields {
    template: Option<String>,
    base: Option<String>,
    addition: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
struct TransmuteFields {
    input: Option<String>,
    material: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct AllRecipe {
    #[serde(rename = "type")]
    type_: String,

    #[serde(flatten)]
    common: CommonFields,

    #[serde(flatten)]
    cooking: CookingFields,

    #[serde(flatten)]
    crafting: CraftingFields,

    #[serde(flatten)]
    smithing: SmithingFields,

    #[serde(flatten)]
    tranmute: TransmuteFields,

    #[serde(rename = "result")]
    result: Option<RecipeResult>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RecipeType {
    Blasting,
    CampfireCooking,
    CraftingShaped,
    CraftingShapeless,
    CraftingTransmute,
    CraftingSpecial(CraftingSpecialRecipeType),
    CraftingDecoratedPot,
    Smelting,
    SmithingTransform,
    SmithingTrim,
    Smoking,
    Stonecutting,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CraftingSpecialRecipeType {
    ArmorDye,
    BannerDuplicate,
    BookCloning,
    FireworkRocket,
    FireworkStar,
    FireworkStarFade,
    MapCloning,
    MapExtending,
    RepairItem,
    ShieldDecoration,
    TippedArrow,
}

impl FromStr for RecipeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blasting" => Ok(RecipeType::Blasting),
            "campfire_cooking" => Ok(RecipeType::CampfireCooking),
            "crafting_shaped" => Ok(RecipeType::CraftingShaped),
            "crafting_shapeless" => Ok(RecipeType::CraftingShapeless),
            "crafting_transmute" => Ok(RecipeType::CraftingTransmute),
            "crafting_decorated_pot" => Ok(RecipeType::CraftingDecoratedPot),
            "smelting" => Ok(RecipeType::Smelting),
            "smithing_transform" => Ok(RecipeType::SmithingTransform),
            "smithing_trim" => Ok(RecipeType::SmithingTrim),
            "smoking" => Ok(RecipeType::Smoking),
            "stonecutting" => Ok(RecipeType::Stonecutting),

            s if s.starts_with("crafting_special_") => {
                let special_type = s.trim_start_matches("crafting_special_");
                CraftingSpecialRecipeType::from_str(special_type)
                    .map(RecipeType::CraftingSpecial)
                    .map_err(|_| format!("Unknown CraftingSpecialRecipeType: {}", s))
            }

            _ => Err(format!("Unknown RecipeType: {}", s)),
        }
    }
}

impl FromStr for CraftingSpecialRecipeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "armordye" => Ok(CraftingSpecialRecipeType::ArmorDye),
            "bannerduplicate" => Ok(CraftingSpecialRecipeType::BannerDuplicate),
            "bookcloning" => Ok(CraftingSpecialRecipeType::BookCloning),
            "firework_rocket" => Ok(CraftingSpecialRecipeType::FireworkRocket),
            "firework_star" => Ok(CraftingSpecialRecipeType::FireworkStar),
            "firework_star_fade" => Ok(CraftingSpecialRecipeType::FireworkStarFade),
            "mapcloning" => Ok(CraftingSpecialRecipeType::MapCloning),
            "mapextending" => Ok(CraftingSpecialRecipeType::MapExtending),
            "repairitem" => Ok(CraftingSpecialRecipeType::RepairItem),
            "shielddecoration" => Ok(CraftingSpecialRecipeType::ShieldDecoration),
            "tippedarrow" => Ok(CraftingSpecialRecipeType::TippedArrow),
            _ => Err(format!("Unknown CraftingSpecialRecipeType: {}", s)),
        }
    }
}

const RECIPES_JSON: &str = include_str!("../../../assets/recipes.json");
#[derive(Debug)]
pub struct RecipeRegistry {
    //Output : Recipe
    pub cooking_recipes: HashMap<String, Vec<CookingRecipe>>,
    pub crafting_shaped_recipes: HashMap<String, Vec<CraftingShapedRecipe>>,
    pub crafting_shapeless_recipes: HashMap<String, Vec<CraftingShapelessRecipe>>,
    pub smithing_transform_recipes: HashMap<String, Vec<SmithingTransformRecipe>>,
    pub stone_cutting_recipes: HashMap<String, Vec<StoneCuttingRecipe>>,
    pub crafting_transmute_recipes: HashMap<String, Vec<CraftingTransmuteRecipe>>,

    //No Output
    pub crafting_decorated_pot_recipes: Vec<CraftingDecoratedPotRecipe>,
    pub crafting_special_recipes: Vec<CraftingSpecial>,
    pub smithing_trim_recipes: Vec<SmithingTrimRecipe>,
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        let mut all_recipe: Vec<AllRecipe> =
            serde_json::from_str(RECIPES_JSON).expect("Failed to parse recipes");

        //strip minecraft: prefix
        for recipe in all_recipe.iter_mut() {
            let crafting_ingredients = recipe.crafting.ingredients.clone().map(|ingredient_list| {
                ingredient_list
                    .iter()
                    .map(Ingredient::strip_minecraft_prefix)
                    .collect::<Vec<Ingredient>>()
            });
            recipe.crafting.ingredients = crafting_ingredients;

            if let Some(ref mut cooking_ingredient) = recipe.cooking.ingredient {
                *cooking_ingredient = Ingredient::strip_minecraft_prefix(cooking_ingredient);
            }

            if let Some(ref mut key) = recipe.crafting.key {
                for val in key.values_mut() {
                    *val = Ingredient::strip_minecraft_prefix(val);
                }
            }

            if let Some(ref mut result) = recipe.result {
                if let Some(key) = result.key.strip_prefix("minecraft:") {
                    result.key = key.to_string();
                }
            }
        }

        let mut cooking_recipes: HashMap<String, Vec<CookingRecipe>> = HashMap::new();
        let mut crafting_shaped_recipes: HashMap<String, Vec<CraftingShapedRecipe>> =
            HashMap::new();
        let mut crafting_shapeless_recipes: HashMap<String, Vec<CraftingShapelessRecipe>> =
            HashMap::new();
        let mut smithing_transform_recipes: HashMap<String, Vec<SmithingTransformRecipe>> =
            HashMap::new();
        let mut stone_cutting_recipes: HashMap<String, Vec<StoneCuttingRecipe>> = HashMap::new();
        let mut crafting_transmute_recipes: HashMap<String, Vec<CraftingTransmuteRecipe>> =
            HashMap::new();

        let mut crafting_special_recipes = vec![];
        let mut smithing_trim_recipes = vec![];
        let mut crafting_decorated_pot_recipes = vec![];

        let mut add_recipe_with_output =
            |type_: &str, recipe_key: String, recipe: AllRecipe| match type_ {
                "blasting" | "campfire_cooking" | "smelting" | "smoking" => {
                    let cooking_recipe = CookingRecipe::new(
                        RecipeType::from_str(type_).unwrap(),
                        recipe.common.category,
                        recipe.common.group,
                        recipe.cooking.ingredient.unwrap(),
                        recipe.cooking.cookingtime.unwrap(),
                        recipe.result.unwrap(),
                        recipe.cooking.experience.unwrap(),
                    );

                    Self::default_insert(&mut cooking_recipes, recipe_key, cooking_recipe);
                }
                "crafting_shaped" => {
                    let pattern = match recipe.crafting.pattern.unwrap() {
                        Ingredient::Single(s) => {
                            panic!("crafting_shaped pattern: {s}")
                        }
                        Ingredient::Multi(pattern) => pattern,
                    };
                    let crafting_shaped_recipe = CraftingShapedRecipe::new(
                        RecipeType::CraftingShaped,
                        recipe.common.category,
                        recipe.common.group,
                        recipe.common.show_notification,
                        pattern,
                        recipe.crafting.key.unwrap(),
                        recipe.result.unwrap(),
                    );

                    Self::default_insert(
                        &mut crafting_shaped_recipes,
                        recipe_key,
                        crafting_shaped_recipe,
                    );
                }
                "crafting_shapeless" => {
                    let crafting_shapeless_recipe = CraftingShapelessRecipe::new(
                        RecipeType::CraftingShapeless,
                        recipe.common.category,
                        recipe.common.group,
                        recipe.crafting.ingredients.unwrap(),
                        recipe.result.unwrap(),
                    );
                    Self::default_insert(
                        &mut crafting_shapeless_recipes,
                        recipe_key,
                        crafting_shapeless_recipe,
                    );
                }
                "crafting_transmute" => {
                    let crafting_transmute_recipe = CraftingTransmuteRecipe::new(
                        RecipeType::CraftingTransmute,
                        recipe.common.category,
                        recipe.common.group,
                        recipe.tranmute.input.unwrap(),
                        recipe.tranmute.material.unwrap(),
                        recipe.result.unwrap(),
                    );
                    Self::default_insert(
                        &mut crafting_transmute_recipes,
                        recipe_key,
                        crafting_transmute_recipe,
                    );
                }
                "smithing_transform" => {
                    let smithing_transform_recipe = SmithingTransformRecipe::new(
                        RecipeType::SmithingTransform,
                        recipe.smithing.template.unwrap(),
                        recipe.smithing.base.unwrap(),
                        recipe.smithing.addition.unwrap(),
                        recipe.result.unwrap(),
                    );
                    Self::default_insert(
                        &mut smithing_transform_recipes,
                        recipe_key,
                        smithing_transform_recipe,
                    );
                }
                "stonecutting" => {
                    let stonecutting_recipe = StoneCuttingRecipe::new(
                        RecipeType::Stonecutting,
                        recipe.cooking.ingredient.unwrap(),
                        recipe.result.unwrap(),
                    );
                    Self::default_insert(
                        &mut stone_cutting_recipes,
                        recipe_key,
                        stonecutting_recipe,
                    );
                }
                _ => {}
            };

        for recipe in all_recipe {
            let type_ = recipe.type_.strip_prefix("minecraft:").unwrap();

            if let Some(recipe_result) = recipe.result.clone() {
                let recipe_key = recipe_result.key;
                add_recipe_with_output(type_, recipe_key, recipe.clone());
            } else {
                match type_ {
                    "crafting_decorated_pot" => {
                        let crafting_decorated_pot_recipe = CraftingDecoratedPotRecipe::new(
                            RecipeType::CraftingDecoratedPot,
                            recipe.common.category.unwrap(),
                        );
                        crafting_decorated_pot_recipes.push(crafting_decorated_pot_recipe);
                    }
                    "smithing_trim" => {
                        let smithing_trim_recipe = SmithingTrimRecipe::new(
                            RecipeType::SmithingTrim,
                            recipe.smithing.template.unwrap(),
                            recipe.smithing.base.unwrap(),
                            recipe.smithing.addition.unwrap(),
                        );
                        smithing_trim_recipes.push(smithing_trim_recipe);
                    }
                    type_ if type_.contains("crafting_special") => {
                        let crafting_special_recipe = CraftingSpecial::new(
                            RecipeType::from_str(type_).unwrap(),
                            recipe.common.category.unwrap(),
                        );
                        crafting_special_recipes.push(crafting_special_recipe);
                    }
                    _ => {}
                }
            }
        }

        Self {
            cooking_recipes,
            crafting_shaped_recipes,
            crafting_shapeless_recipes,
            crafting_decorated_pot_recipes,
            smithing_transform_recipes,
            smithing_trim_recipes,
            stone_cutting_recipes,
            crafting_special_recipes,
            crafting_transmute_recipes,
        }
    }
}

impl RecipeRegistry {
    fn default_insert<T: Recipe>(recipe_map: &mut HashMap<String, Vec<T>>, key: String, recipe: T) {
        recipe_map.entry(key).or_default().push(recipe);
    }

    #[must_use]
    pub fn get_cooking_recipe_with_output(
        &self,
        item: &Item,
        recipe_type: RecipeType,
    ) -> Option<&CookingRecipe> {
        if let Some(recipe_list) = self.cooking_recipes.get(item.registry_key) {
            if let Some(recipe) = recipe_list
                .iter()
                .find(|recipe| recipe.recipe_type == recipe_type)
            {
                return Some(recipe);
            };
        };
        None
    }

    #[must_use]
    pub fn get_cooking_recipe_with_ingredient(
        &self,
        item: &Item,
        recipe_type: RecipeType,
    ) -> Option<&CookingRecipe> {
        //get all recipe list
        for recipe_list in self.cooking_recipes.values() {
            let recipe = recipe_list.iter().find(|recipe| {
                recipe.recipe_type == recipe_type
                    && match recipe.ingredient.clone() {
                        Ingredient::Single(ingredient) => ingredient == item.registry_key,
                        Ingredient::Multi(ingredients) => {
                            ingredients.contains(&item.registry_key.to_string())
                        }
                    }
            });
            match recipe {
                Some(recipe) => return Some(recipe),
                None => continue,
            }
        }

        None
    }
}

static RECIPE_REGISTRY: OnceLock<Arc<RecipeRegistry>> = OnceLock::new();

pub fn init_recipe_registry() -> Arc<RecipeRegistry> {
    RECIPE_REGISTRY
        .get_or_init(|| Arc::new(RecipeRegistry::default()))
        .clone()
}

pub fn get_recipe_registry() -> Arc<RecipeRegistry> {
    RECIPE_REGISTRY
        .get()
        .expect("RecipeRegistry not initialized")
        .clone()
}

pub trait Recipe {}
