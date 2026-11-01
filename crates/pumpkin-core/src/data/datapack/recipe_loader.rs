use pumpkin_data::recipes::RecipeCategoryTypes;
use pumpkin_protocol::codec::recipe::{
    DynamicRecipe, OwnedBrewingRecipe, OwnedCookingRecipe, OwnedCookingRecipeType,
    OwnedCraftingRecipe, OwnedRecipeIngredient, OwnedRecipeResult, OwnedSmithingTransformRecipe,
    OwnedSmithingTrimRecipe, OwnedSpecialCraftingRecipe, OwnedStonecutterRecipe,
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
        "crafting_transmute" => parse_transmute(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_dye" => parse_dye(recipe_id, &value).map(DynamicRecipe::Crafting),
        "crafting_decorated_pot" => {
            parse_decorated_pot(recipe_id, &value).map(DynamicRecipe::Crafting)
        }
        "crafting_imbue" => parse_imbue(recipe_id, &value).map(DynamicRecipe::Crafting),
        "stonecutting" => parse_stonecutting(recipe_id, &value).map(DynamicRecipe::Stonecutting),
        "smithing_transform" => {
            parse_smithing_transform(recipe_id, &value).map(DynamicRecipe::SmithingTransform)
        }
        "smithing_trim" => parse_smithing_trim(recipe_id, &value).map(DynamicRecipe::SmithingTrim),
        s if s.starts_with("crafting_special_") => {
            let category = parse_category(value.get("category"));
            Some(DynamicRecipe::SpecialCrafting(OwnedSpecialCraftingRecipe {
                recipe_id,
                category,
            }))
        }
        "smelting" => parse_cooking(recipe_id, &value, 200)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smelting(c))),
        "blasting" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Blasting(c))),
        "smoking" => parse_cooking(recipe_id, &value, 100)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::Smoking(c))),
        "campfire_cooking" => parse_cooking(recipe_id, &value, 600)
            .map(|c| DynamicRecipe::Cooking(OwnedCookingRecipeType::CampfireCooking(c))),
        "brewing" => parse_brewing(recipe_id, &value).map(DynamicRecipe::Brewing),
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

fn parse_brewing(recipe_id: String, value: &Value) -> Option<OwnedBrewingRecipe> {
    let input = value.get("input")?;
    let input_item = normalize_id(input.get("item")?.as_str()?);
    let input_potion = input
        .get("potion_contents")
        .and_then(|pc| pc.get("potions").or_else(|| pc.get("potion")))
        .and_then(Value::as_str)
        .map(normalize_id);

    let reagent = normalize_id(value.get("reagent")?.get("item")?.as_str()?);

    let output = value.get("output")?;
    let output_item = normalize_id(output.get("id").or_else(|| output.get("item"))?.as_str()?);
    let output_potion = output
        .get("components")
        .and_then(|c| c.get("minecraft:potion_contents"))
        .and_then(|pc| pc.get("potion"))
        .and_then(Value::as_str)
        .map(normalize_id);

    Some(OwnedBrewingRecipe {
        recipe_id,
        input_item,
        input_potion,
        reagent,
        output_item,
        output_potion,
    })
}

fn parse_transmute(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let input = parse_ingredient(value.get("input")?)?;
    let material = parse_ingredient(value.get("material")?)?;
    let result = parse_result(value.get("result")?).unwrap_or(OwnedRecipeResult {
        item_id: "minecraft:filled_map".to_string(),
        count: 1,
    });
    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients: vec![input, material],
        result,
    })
}

fn parse_dye(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let target = parse_ingredient(value.get("target")?)?;
    let dye = parse_ingredient(value.get("dye")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shapeless {
        recipe_id: Some(recipe_id),
        category,
        group,
        ingredients: vec![target, dye],
        result,
    })
}

fn parse_decorated_pot(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let back = parse_ingredient(value.get("back")?)?;
    let left = parse_ingredient(value.get("left")?)?;
    let right = parse_ingredient(value.get("right")?)?;
    let front = parse_ingredient(value.get("front")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification: true,
        key: vec![('B', back), ('L', left), ('R', right), ('F', front)],
        pattern: vec![" B ".to_string(), "L R".to_string(), " F ".to_string()],
        result,
    })
}

fn parse_imbue(recipe_id: String, value: &Value) -> Option<OwnedCraftingRecipe> {
    let category = parse_category(value.get("category"));
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let material = parse_ingredient(value.get("material")?)?;
    let source = parse_ingredient(value.get("source")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedCraftingRecipe::Shaped {
        recipe_id: Some(recipe_id),
        category,
        group,
        show_notification: true,
        key: vec![('#', material), ('X', source)],
        pattern: vec!["###".to_string(), "#X#".to_string(), "###".to_string()],
        result,
    })
}

fn parse_stonecutting(recipe_id: String, value: &Value) -> Option<OwnedStonecutterRecipe> {
    let group = value
        .get("group")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let ingredient = parse_ingredient(value.get("ingredient")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedStonecutterRecipe {
        recipe_id,
        group,
        ingredient,
        result,
    })
}

fn parse_smithing_transform(
    recipe_id: String,
    value: &Value,
) -> Option<OwnedSmithingTransformRecipe> {
    let template = parse_ingredient(value.get("template")?)?;
    let base = parse_ingredient(value.get("base")?)?;
    let addition = parse_ingredient(value.get("addition")?)?;
    let result = parse_result(value.get("result")?)?;
    Some(OwnedSmithingTransformRecipe {
        recipe_id,
        template,
        base,
        addition,
        result,
    })
}

fn parse_smithing_trim(recipe_id: String, value: &Value) -> Option<OwnedSmithingTrimRecipe> {
    let template = parse_ingredient(value.get("template")?)?;
    let base = parse_ingredient(value.get("base")?)?;
    let addition = parse_ingredient(value.get("addition")?)?;
    let pattern = value.get("pattern")?.as_str()?.to_string();
    Some(OwnedSmithingTrimRecipe {
        recipe_id,
        template,
        base,
        addition,
        pattern: normalize_id(&pattern),
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

    #[test]
    fn parse_brewing_recipe() {
        let json = r#"{
            "type": "minecraft:brewing",
            "input": {
                "item": "minecraft:potion",
                "potion_contents": {
                    "potions": "minecraft:water"
                }
            },
            "output": {
                "components": {
                    "minecraft:potion_contents": {
                        "potion": "minecraft:awkward"
                    }
                },
                "id": "minecraft:potion"
            },
            "reagent": {
                "item": "minecraft:nether_wart"
            }
        }"#;

        let parsed = parse_recipe("minecraft", "brewing/potion_water_nether_wart", json);
        assert!(parsed.is_some());
        if let Some(DynamicRecipe::Brewing(brewing)) = parsed {
            assert_eq!(
                brewing.recipe_id,
                "minecraft:brewing/potion_water_nether_wart"
            );
            assert_eq!(brewing.input_item, "minecraft:potion");
            assert_eq!(brewing.input_potion.as_deref(), Some("minecraft:water"));
            assert_eq!(brewing.reagent, "minecraft:nether_wart");
            assert_eq!(brewing.output_item, "minecraft:potion");
            assert_eq!(brewing.output_potion.as_deref(), Some("minecraft:awkward"));
        } else {
            panic!("Expected brewing recipe");
        }
    }
}
