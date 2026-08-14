use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    BlockStateImpl, CustomModelDataImpl, CustomNameImpl, DamageImpl, DataComponentImpl,
    EnchantmentsImpl, ItemModelImpl, MapIdImpl, MaxDamageImpl, MaxStackSizeImpl,
    StoredEnchantmentsImpl, UnbreakableImpl,
};
use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use pumpkin_util::text::TextComponent;
use serde::Deserialize;
use std::borrow::Cow;

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
            let result = parse_result(raw.result.as_ref());

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
            let result = parse_result(raw.result.as_ref());

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

fn parse_result(val: Option<&serde_json::Value>) -> OwnedRecipeResult {
    match val {
        Some(serde_json::Value::String(s)) => OwnedRecipeResult {
            item_id: s.clone(),
            count: 1,
            components: Vec::new(),
        },
        Some(serde_json::Value::Object(obj)) => {
            // 26.x uses `id`; older packs use `item`.
            let item_id = obj
                .get("id")
                .or_else(|| obj.get("item"))
                .and_then(|v| v.as_str())
                .unwrap_or("minecraft:air")
                .to_string();
            let count = obj
                .get("count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(1)
                .clamp(1, 99) as u8;
            let components = parse_components(obj.get("components"));
            OwnedRecipeResult {
                item_id,
                count,
                components,
            }
        }
        _ => OwnedRecipeResult {
            item_id: "minecraft:air".to_string(),
            count: 1,
            components: Vec::new(),
        },
    }
}

/// Parse the `components` object of a recipe result into typed data components.
/// Unsupported or malformed components are skipped rather than failing the recipe.
fn parse_components(
    val: Option<&serde_json::Value>,
) -> Vec<(DataComponent, Box<dyn DataComponentImpl>)> {
    let Some(obj) = val.and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    let mut components = Vec::new();
    for (name, value) in obj {
        let Some(component) = parse_data_component(name, value) else {
            tracing::warn!("Skipping unsupported recipe result component `{name}`");
            continue;
        };
        components.push(component);
    }
    components
}

fn parse_data_component(
    name: &str,
    value: &serde_json::Value,
) -> Option<(DataComponent, Box<dyn DataComponentImpl>)> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    match name {
        "max_stack_size" => Some((
            DataComponent::MaxStackSize,
            MaxStackSizeImpl {
                size: value.as_u64()? as u8,
            }
            .to_dyn(),
        )),
        "max_damage" => Some((
            DataComponent::MaxDamage,
            MaxDamageImpl {
                max_damage: value.as_i64()? as i32,
            }
            .to_dyn(),
        )),
        "damage" => Some((
            DataComponent::Damage,
            DamageImpl {
                damage: value.as_i64()? as i32,
            }
            .to_dyn(),
        )),
        "unbreakable" if value.is_boolean() || value.is_object() => {
            Some((DataComponent::Unbreakable, UnbreakableImpl.to_dyn()))
        }
        "item_model" => Some((
            DataComponent::ItemModel,
            ItemModelImpl {
                id: Cow::Owned(value.as_str()?.to_string()),
            }
            .to_dyn(),
        )),
        "custom_name" => Some((
            DataComponent::CustomName,
            CustomNameImpl {
                name: serde_json::from_value::<TextComponent>(value.clone()).ok()?,
            }
            .to_dyn(),
        )),
        "map_id" => Some((
            DataComponent::MapId,
            MapIdImpl {
                id: value.as_i64()? as i32,
            }
            .to_dyn(),
        )),
        "block_state" => Some((
            DataComponent::BlockState,
            BlockStateImpl {
                properties: Cow::Owned(parse_block_state_properties(value)),
            }
            .to_dyn(),
        )),
        "custom_model_data" => Some((
            DataComponent::CustomModelData,
            parse_custom_model_data(value)?.to_dyn(),
        )),
        "enchantments" => Some((
            DataComponent::Enchantments,
            EnchantmentsImpl {
                enchantment: Cow::Owned(parse_enchantments(value)?),
            }
            .to_dyn(),
        )),
        "stored_enchantments" => Some((
            DataComponent::StoredEnchantments,
            StoredEnchantmentsImpl {
                enchantment: Cow::Owned(parse_enchantments(value)?),
            }
            .to_dyn(),
        )),
        _ => None,
    }
}

fn parse_block_state_properties(
    value: &serde_json::Value,
) -> Vec<(Cow<'static, str>, Cow<'static, str>)> {
    value
        .as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(key, v)| {
                    v.as_str()
                        .map(|s| (Cow::Owned(key.clone()), Cow::Owned(s.to_string())))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_custom_model_data(value: &serde_json::Value) -> Option<CustomModelDataImpl> {
    let obj = value.as_object()?;
    let floats = obj
        .get("floats")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_f64().map(|f| f as f32))
                .collect()
        })
        .unwrap_or_default();
    let flags = obj
        .get("flags")
        .and_then(serde_json::Value::as_array)
        .map(|arr| arr.iter().filter_map(serde_json::Value::as_bool).collect())
        .unwrap_or_default();
    let strings = obj
        .get("strings")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let colors = obj
        .get("colors")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_i64().map(|i| i as i32))
                .collect()
        })
        .unwrap_or_default();
    Some(CustomModelDataImpl {
        floats,
        flags,
        strings,
        colors,
    })
}

fn parse_enchantments(value: &serde_json::Value) -> Option<Vec<(&'static Enchantment, i32)>> {
    let obj = value.as_object()?;
    let mut out = Vec::with_capacity(obj.len());
    for (id, level) in obj {
        let enchantment = Enchantment::from_name(id)?;
        let level = level.as_i64().unwrap_or(1);
        out.push((enchantment, level as i32));
    }
    Some(out)
}

fn make_owned_cooking(raw: &RawRecipeJson) -> OwnedCookingRecipe {
    let ingredient = raw.ingredient.as_ref().map_or(
        OwnedRecipeIngredient::Simple("minecraft:air".to_string()),
        parse_ingredient,
    );
    let result = parse_result(raw.result.as_ref());

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
