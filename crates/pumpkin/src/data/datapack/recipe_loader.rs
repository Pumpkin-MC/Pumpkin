use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
    OwnedRecipeIngredient, OwnedRecipeResult,
};
use serde_json::Value;

pub fn parse_recipe(namespace: &str, name: &str, json_str: &str) -> Option<DynamicRecipe> {
    let value: Value = serde_json::from_str(json_str).ok()?;
    let recipe_id = format!("{namespace}:{name}");
    let raw_type = value.get("type")?.as_str()?;
    let recipe_type = raw_type.strip_prefix("minecraft:").unwrap_or(raw_type);

    match recipe_type {
        "crafting_shaped" => parse_shaped(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_shapeless" => parse_shapeless(recipe_id, &value).map(DynamicRecipe::Crafting),
        "smelting" => parse_cooking(recipe_id, &value, 200)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(c))),
        "blasting" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(c))),
        "smoking" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(c))),
        "campfire_cooking" => parse_cooking(recipe_id, &value, 600)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::CampfireCooking(c))),
        _ => None,
    }
}

fn normalize_id(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

fn parse_ingredient(value: &Value) -> Option<OwnedRecipeIngredient> {
    match value {
        Value::String(s) => Some(OwnedRecipeIngredient::Simple(normalize_id(s))),
        Value::Object(map) => map
            .get("tag")
            .and_then(Value::as_str)
            .map(|tag| OwnedRecipeIngredient::Tagged(normalize_id(tag)))
            .or_else(|| {
                map.get("item")
                    .and_then(Value::as_str)
                    .map(|item| OwnedRecipeIngredient::Simple(normalize_id(item)))
            }),
        Value::Array(arr) => {
            let mut ids = Vec::new();
            for item in arr {
                if let Some(s) = item.as_str() {
                    ids.push(normalize_id(s));
                } else if let Some(obj_item) = item.get("item").and_then(Value::as_str) {
                    ids.push(normalize_id(obj_item));
                }
            }
            if ids.is_empty() {
                None
            } else {
                Some(OwnedRecipeIngredient::OneOf(ids))
            }
        }
        _ => None,
    }
}

fn parse_result(value: &Value) -> Option<OwnedRecipeResult> {
    match value {
        Value::String(s) => Some(OwnedRecipeResult {
            item_id: normalize_id(s),
            count: 1,
        }),
        Value::Object(map) => {
            let id = map
                .get("id")
                .or_else(|| map.get("item"))
                .and_then(Value::as_str)?;
            let count = map
                .get("count")
                .and_then(Value::as_u64)
                .unwrap_or(1)
                .min(99) as u8;
            Some(OwnedRecipeResult {
                item_id: normalize_id(id),
                count,
            })
        }
        _ => None,
    }
}

fn parse_category(value: Option<&Value>) -> RecipeCategoryTypes {
    let Some(val) = value.and_then(Value::as_str) else {
        return RecipeCategoryTypes::Misc;
    };
    match val.to_ascii_lowercase().as_str() {
        "equipment" => RecipeCategoryTypes::Equipment,
        "building" => RecipeCategoryTypes::Building,
        "redstone" | "restone" => RecipeCategoryTypes::Restone,
        "food" => RecipeCategoryTypes::Food,
        "blocks" => RecipeCategoryTypes::Blocks,
        _ => RecipeCategoryTypes::Misc,
    }
}

fn parse_shaped(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let pattern = value
        .get("pattern")?
        .as_array()?
        .iter()
        .map(|v| v.as_str().map(ToString::to_string))
        .collect::<Option<Vec<_>>>()?;

    let key_map = value.get("key")?.as_object()?;
    let mut key = Vec::new();
    for (k, v) in key_map {
        let ch = k.chars().next()?;
        let ingredient = parse_ingredient(v)?;
        key.push((ch, ingredient));
    }

    let result = parse_result(value.get("result")?)?;
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let show_notification = value
        .get("show_notification")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification,
        key,
        pattern,
        result,
    })
}

fn parse_shapeless(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let ingredients_arr = value.get("ingredients")?.as_array()?;
    let mut ingredients = Vec::new();
    for ing_val in ingredients_arr {
        ingredients.push(parse_ingredient(ing_val)?);
    }

    let result = parse_result(value.get("result")?)?;
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients,
        result,
    })
}

fn parse_cooking(
    recipe_id: String,
    value: &Value,
    default_cooking_time: i32,
) -> Option<OwnedCookingRecipe> {
    let ingredient = parse_ingredient(value.get("ingredient")?)?;
    let result = parse_result(value.get("result")?)?;
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let cooking_time = value
        .get("cookingtime")
        .and_then(Value::as_i64)
        .map_or(default_cooking_time, |t| t as i32);
    let experience = value
        .get("experience")
        .and_then(Value::as_f64)
        .map_or(0.0, |e| e as f32);

    Some(OwnedCookingRecipe {
        recipe_id,
        category,
        group,
        ingredient,
        cooking_time,
        experience,
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shaped_recipe() {
        let json = r##"{
            "type": "minecraft:crafting_shaped",
            "group": "sword",
            "pattern": ["X", "X", "#"],
            "key": {
                "#": "minecraft:stick",
                "X": "minecraft:diamond"
            },
            "result": {
                "id": "minecraft:diamond_sword",
                "count": 1
            }
        }"##;

        let parsed = parse_recipe("test", "diamond_sword", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shaped {
            recipe_id,
            result,
            pattern,
            ..
        })) = parsed
        {
            assert_eq!(recipe_id.as_deref(), Some("test:diamond_sword"));
            assert_eq!(result.item_id, "minecraft:diamond_sword");
            assert_eq!(pattern.len(), 3);
        } else {
            panic!("Expected shaped recipe");
        }
    }

    #[test]
    fn parse_shapeless_recipe() {
        let json = r#"{
            "type": "minecraft:crafting_shapeless",
            "ingredients": ["minecraft:oak_log"],
            "result": {
                "id": "minecraft:oak_planks",
                "count": 4
            }
        }"#;

        let parsed = parse_recipe("minecraft", "oak_planks_from_oak_log", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Crafting(OwnedCraftingRecipe::Shapeless {
            recipe_id,
            result,
            ingredients,
            ..
        })) = parsed
        {
            assert_eq!(
                recipe_id.as_deref(),
                Some("minecraft:oak_planks_from_oak_log")
            );
            assert_eq!(result.item_id, "minecraft:oak_planks");
            assert_eq!(result.count, 4);
            assert_eq!(ingredients.len(), 1);
        } else {
            panic!("Expected shapeless recipe");
        }
    }

    #[test]
    fn parse_smelting_recipe() {
        let json = r#"{
            "type": "minecraft:smelting",
            "ingredient": "minecraft:raw_iron",
            "result": {
                "id": "minecraft:iron_ingot"
            },
            "experience": 0.7,
            "cookingtime": 200
        }"#;

        let parsed = parse_recipe("minecraft", "iron_ingot_from_smelting_raw_iron", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(cooking))) = parsed {
            assert_eq!(
                cooking.recipe_id,
                "minecraft:iron_ingot_from_smelting_raw_iron"
            );
            assert_eq!(cooking.result.item_id, "minecraft:iron_ingot");
            assert_eq!(cooking.cooking_time, 200);
        } else {
            panic!("Expected smelting recipe");
        }
    }
}
