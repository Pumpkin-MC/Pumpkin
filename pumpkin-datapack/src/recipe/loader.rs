use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use serde::Deserialize;

use crate::resource::ResourceManager;

/// Load all recipes from enabled datapacks.
pub fn load_recipes(
    manager: &dyn ResourceManager,
) -> Result<Vec<DynamicRecipe>, crate::DatapackError> {
    let mut recipes = Vec::new();

    // Discover recipe JSONs from resource manager
    let namespaces = manager.get_namespaces();
    for ns in &namespaces {
        let paths = crate::resource::list_resources_multi(manager, ns, &["recipe", "recipes"]);
        for path in &paths {
            if !std::path::Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let Some(data) = manager.get_resource(ns, path) else {
                continue;
            };
            match parse_recipe_json(&data) {
                Ok(Some(recipe)) => recipes.push(recipe),
                Ok(None) => {} // unknown type, skip
                Err(e) => {
                    tracing::warn!("Failed to parse recipe {ns}:{path}: {e}");
                }
            }
        }
    }

    Ok(recipes)
}

#[derive(Debug, Deserialize)]
struct RawRecipeJson {
    #[serde(rename = "type")]
    recipe_type: String,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    key: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default)]
    pattern: Option<serde_json::Value>,
    #[serde(default)]
    ingredients: Option<serde_json::Value>,
    #[serde(default)]
    ingredient: Option<serde_json::Value>,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    cookingtime: Option<i32>,
    #[serde(default)]
    experience: Option<f32>,
    #[serde(default)]
    show_notification: Option<bool>,
}

/// Parse a single recipe JSON into a `DynamicRecipe`.
fn parse_recipe_json(data: &[u8]) -> Result<Option<DynamicRecipe>, serde_json::Error> {
    let raw: RawRecipeJson = serde_json::from_slice(data)?;

    let recipe_type = raw
        .recipe_type
        .strip_prefix("minecraft:")
        .unwrap_or(&raw.recipe_type);

    Ok(match recipe_type {
        "crafting_shaped" => {
            let category = parse_category(raw.category.as_deref());
            let group = raw.group;
            let show_notification = raw.show_notification.unwrap_or(true);
            let key = parse_key(raw.key.unwrap_or_default());
            let pattern = raw
                .pattern
                .and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|s| s.as_str().map(String::from))
                            .collect::<Vec<String>>()
                    })
                })
                .unwrap_or_default();
            let result = parse_result(raw.result)?;

            Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shaped {
                category,
                group,
                show_notification,
                key,
                pattern,
                result,
            }))
        }
        "crafting_shapeless" => {
            let category = parse_category(raw.category.as_deref());
            let group = raw.group;
            let ingredients = parse_ingredients(raw.ingredients);
            let result = parse_result(raw.result)?;

            Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shapeless {
                category,
                group,
                ingredients,
                result,
            }))
        }
        "smelting" => Some(DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(
            make_owned_cooking(&raw),
        ))),
        "blasting" => Some(DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(
            make_owned_cooking(&raw),
        ))),
        "smoking" => Some(DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(
            make_owned_cooking(&raw),
        ))),
        "campfire_cooking" => Some(DynamicRecipe::Cooking(
            OwnedCookingRecipeType::CampfireCooking(make_owned_cooking(&raw)),
        )),
        "stonecutting" | "smithing_transform" | "smithing_trim" => {
            // TODO - Placeholder: parse but don't send as recipe for now
            None
        }
        _ => None,
    })
}

fn parse_category(cat: Option<&str>) -> RecipeCategoryTypes {
    match cat {
        Some("building") => RecipeCategoryTypes::Building,
        Some("redstone") => RecipeCategoryTypes::Restone,
        Some("equipment" | "combat") => RecipeCategoryTypes::Equipment,
        Some("food") => RecipeCategoryTypes::Food,
        _ => RecipeCategoryTypes::Misc,
    }
}

fn parse_key(
    map: serde_json::Map<String, serde_json::Value>,
) -> Vec<(char, OwnedRecipeIngredient)> {
    let mut keys = Vec::new();
    for (k, v) in map {
        if let Some(c) = k.chars().next() {
            keys.push((c, parse_ingredient(&v)));
        }
    }
    keys
}

fn parse_ingredients(val: Option<serde_json::Value>) -> Vec<OwnedRecipeIngredient> {
    let mut ingredients = Vec::new();
    let Some(val) = val else {
        return ingredients;
    };
    match val {
        serde_json::Value::Array(arr) => {
            for v in arr {
                ingredients.push(parse_ingredient(&v));
            }
        }
        _ => {
            ingredients.push(parse_ingredient(&val));
        }
    }
    ingredients
}

#[allow(clippy::option_if_let_else)]
fn parse_ingredient(val: &serde_json::Value) -> OwnedRecipeIngredient {
    if let Some(s) = val.as_str() {
        if let Some(tag) = s.strip_prefix('#') {
            OwnedRecipeIngredient::Tagged(tag.to_string())
        } else {
            OwnedRecipeIngredient::Simple(s.to_string())
        }
    } else if let Some(obj) = val.as_object() {
        if let Some(item) = obj.get("item").and_then(|v| v.as_str()) {
            OwnedRecipeIngredient::Simple(item.to_string())
        } else if let Some(tag) = obj.get("tag").and_then(|v| v.as_str()) {
            OwnedRecipeIngredient::Tagged(tag.to_string())
        } else {
            OwnedRecipeIngredient::Simple("minecraft:air".to_string())
        }
    } else if let Some(arr) = val.as_array() {
        let items: Vec<String> = arr
            .iter()
            .filter_map(|v| {
                v.as_str()
                    .or_else(|| v.get("item").and_then(|x| x.as_str()))
                    .map(String::from)
            })
            .collect();
        OwnedRecipeIngredient::OneOf(items)
    } else {
        OwnedRecipeIngredient::Simple("minecraft:air".to_string())
    }
}

#[allow(clippy::unnecessary_wraps, clippy::option_if_let_else)]
fn parse_result(val: Option<serde_json::Value>) -> Result<OwnedRecipeResult, serde_json::Error> {
    let Some(val) = val else {
        return Ok(OwnedRecipeResult {
            item_id: "minecraft:air".to_string(),
            count: 1,
        });
    };

    if let Some(s) = val.as_str() {
        Ok(OwnedRecipeResult {
            item_id: s.to_string(),
            count: 1,
        })
    } else if let Some(obj) = val.as_object() {
        let item_id = obj
            .get("item")
            .and_then(|v| v.as_str())
            .unwrap_or("minecraft:air")
            .to_string();
        let count = obj
            .get("count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(1) as u8;
        Ok(OwnedRecipeResult { item_id, count })
    } else {
        Ok(OwnedRecipeResult {
            item_id: "minecraft:air".to_string(),
            count: 1,
        })
    }
}

fn make_owned_cooking(raw: &RawRecipeJson) -> OwnedCookingRecipe {
    let ingredient = raw.ingredient.as_ref().map_or(
        OwnedRecipeIngredient::Simple("minecraft:air".to_string()),
        parse_ingredient,
    );
    let result = raw
        .result
        .as_ref()
        .and_then(|r| {
            r.as_str()
                .map(|s| OwnedRecipeResult {
                    item_id: s.to_string(),
                    count: 1,
                })
                .or_else(|| {
                    r.as_object().map(|o| OwnedRecipeResult {
                        item_id: o
                            .get("item")
                            .and_then(|v| v.as_str())
                            .unwrap_or("minecraft:air")
                            .to_string(),
                        count: o
                            .get("count")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(1) as u8,
                    })
                })
        })
        .unwrap_or_else(|| OwnedRecipeResult {
            item_id: "minecraft:air".to_string(),
            count: 1,
        });

    OwnedCookingRecipe {
        recipe_id: String::new(),
        category: parse_category(raw.category.as_deref()),
        group: raw.group.clone(),
        ingredient,
        cooking_time: raw.cookingtime.unwrap_or(200),
        experience: raw.experience.unwrap_or(0.0),
        result,
    }
}
