use pumpkin_data::{item::Item, recipes::RecipeIngredientTypes};

#[test]
fn simple_ingredients_require_the_exact_minecraft_name() {
    for name in [
        "minecraft:stone",
        "stone",
        "other:stone",
        "minecraft:minecraft:stone",
        "minecraft:",
        "minecraft:stone_bricks",
        "minecraft:Stone",
    ] {
        let ingredient = RecipeIngredientTypes::Simple(name);
        assert_eq!(
            ingredient.match_item(&Item::STONE),
            name == "minecraft:stone",
            "{name}"
        );
    }
}

#[test]
fn alternatives_match_only_exact_names() {
    let ingredient =
        RecipeIngredientTypes::OneOf(&["other:stone", "minecraft:dirt", "minecraft:stone"]);
    assert!(ingredient.match_item(&Item::STONE));
    assert!(ingredient.match_item(&Item::DIRT));
    assert!(!ingredient.match_item(&Item::DIAMOND));
    assert!(!RecipeIngredientTypes::OneOf(&[]).match_item(&Item::STONE));
    assert!(!RecipeIngredientTypes::OneOf(&["stone", "other:stone"]).match_item(&Item::STONE));
}

#[test]
fn ingredient_matching_agrees_with_formatted_names_for_every_item() {
    let ingredients = [
        RecipeIngredientTypes::Simple("minecraft:stone"),
        RecipeIngredientTypes::Simple("minecraft:iron_ore"),
        RecipeIngredientTypes::Simple("stone"),
        RecipeIngredientTypes::Simple("other:stone"),
        RecipeIngredientTypes::OneOf(&[
            "minecraft:stone",
            "minecraft:diamond",
            "minecraft:iron_ore",
        ]),
        RecipeIngredientTypes::OneOf(&[]),
    ];
    for item in (0..=u16::MAX).filter_map(Item::from_id) {
        let name = format!("minecraft:{}", item.registry_key);
        for ingredient in &ingredients {
            let expected = match ingredient {
                RecipeIngredientTypes::Simple(key) => name == *key,
                RecipeIngredientTypes::OneOf(keys) => keys.contains(&name.as_str()),
                RecipeIngredientTypes::Tagged(_) => continue,
            };
            assert_eq!(
                ingredient.match_item(item),
                expected,
                "{ingredient:?}: {name}"
            );
        }
    }
}
