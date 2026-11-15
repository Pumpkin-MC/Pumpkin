use pumpkin_data::item::{Item, JavaToBedrockItemMapping};
use pumpkin_protocol::{
    bedrock::{
        client::{
            BedrockRecipe, BedrockShapedRecipe, BedrockShapelessRecipe, ItemDescriptorCount,
            RecipeUnlockRequirement,
        },
        network_item::NetworkItemDescriptor,
    },
    codec::{
        recipe::{
            DynamicRecipe, OwnedCookingRecipe, OwnedCookingRecipeType, OwnedCraftingRecipe,
            OwnedRecipeIngredient, OwnedRecipeResult, OwnedStonecutterRecipe,
        },
        var_int::VarInt,
        var_uint::VarUInt,
    },
};
use uuid::Uuid;

// Item tags present natively in Bedrock 1.26.60. Java-only tags are expanded into item descriptors.
const NATIVE_BEDROCK_TAGS: &[&str] = &[
    "minecraft:coals",
    "minecraft:crimson_stems",
    "minecraft:egg",
    "minecraft:logs",
    "minecraft:logs_that_burn",
    "minecraft:mangrove_logs",
    "minecraft:metal_nuggets",
    "minecraft:mushrooms_for_stew",
    "minecraft:planks",
    "minecraft:soul_fire_base_blocks",
    "minecraft:stone_crafting_materials",
    "minecraft:stone_tool_materials",
    "minecraft:trim_materials",
    "minecraft:trim_templates",
    "minecraft:trimmable_armors",
    "minecraft:warped_stems",
    "minecraft:wooden_slabs",
    "minecraft:wool",
];

/// Builds dynamic Bedrock recipe entries registered at runtime via [`crate::server::RecipeManager`].
///
/// Vanilla Bedrock recipes are pre-encoded in [`pumpkin_data::bedrock_recipe`].
pub fn crafting_data(recipe_manager: &crate::server::RecipeManager) -> Vec<BedrockRecipe> {
    let dynamic = recipe_manager.get_dynamic_recipes_internal();
    if dynamic.is_empty() {
        return Vec::new();
    }

    let mut recipes = Vec::new();
    let mut network_id = pumpkin_data::bedrock_recipe::NEXT_RECIPE_NETWORK_ID;
    for recipe in dynamic {
        match recipe {
            DynamicRecipe::Crafting(crafting) => {
                append_dynamic_crafting_recipe(&mut recipes, &crafting, &mut network_id);
            }
            DynamicRecipe::Stonecutting(stonecutting) => {
                append_dynamic_stonecutter_recipe(&mut recipes, &stonecutting, &mut network_id);
            }
            DynamicRecipe::Cooking(cooking) => {
                append_dynamic_cooking_recipe(&mut recipes, &cooking, &mut network_id);
            }
            DynamicRecipe::Brewing(_)
            | DynamicRecipe::SmithingTransform(_)
            | DynamicRecipe::SmithingTrim(_)
            | DynamicRecipe::SpecialCrafting(_) => {}
        }
    }
    recipes
}

/// Resolves a Bedrock stonecutter `recipe_network_id` to `(bedrock_item_id, bedrock_aux_value)`.
pub fn stonecutter_output_for_recipe_id(
    recipe_manager: Option<&crate::server::RecipeManager>,
    recipe_id: u32,
) -> Option<(i16, u32)> {
    let static_recipes = pumpkin_data::bedrock_recipe::STONECUTTER_RECIPES;
    if let Ok(index) = static_recipes.binary_search_by_key(&recipe_id, |entry| entry.network_id) {
        let entry = static_recipes[index];
        return Some((entry.output_item_id, entry.output_aux_value));
    }

    if recipe_id >= pumpkin_data::bedrock_recipe::NEXT_RECIPE_NETWORK_ID
        && let Some(manager) = recipe_manager
    {
        for recipe in crafting_data(manager) {
            if let BedrockRecipe::Shapeless(shapeless) = recipe
                && shapeless.block == "stonecutter"
                && shapeless.recipe_network_id.0 == recipe_id
                && let Some(out) = shapeless.output.first()
                && let Ok(item_id) = i16::try_from(out.id.0)
            {
                return Some((item_id, out.aux_value.0));
            }
        }
    }

    None
}

fn item_descriptor(identifier: &str) -> Option<ItemDescriptorCount> {
    let item = Item::from_registry_key(identifier)?;
    let mapping = JavaToBedrockItemMapping::from_java_item_id(item.id)?;
    Some(ItemDescriptorCount::item(
        mapping.bedrock_item.registry_key.to_string(),
        mapping.bedrock_data.cast_signed(),
    ))
}

fn push_unique(
    mut options: Vec<ItemDescriptorCount>,
    option: ItemDescriptorCount,
) -> Vec<ItemDescriptorCount> {
    if !options.contains(&option) {
        options.push(option);
    }
    options
}

/// Expands the per-slot option lists of a recipe into the variants Bedrock
/// needs, one entry per variant holding the chosen descriptor of every slot.
///
/// Slots sharing an identical option list must resolve to the same choice (the
/// two planks of a stick recipe, for example), so identical lists collapse into
/// a single dimension. Slots with different lists are independent, so every
/// combination of their choices is a valid variant.
fn ingredient_variants(options: &[Vec<ItemDescriptorCount>]) -> Vec<Vec<ItemDescriptorCount>> {
    if options.iter().any(Vec::is_empty) {
        return Vec::new();
    }

    let mut dimensions: Vec<(&[ItemDescriptorCount], Vec<usize>)> = Vec::new();
    for (slot, list) in options.iter().enumerate() {
        let list = list.as_slice();
        match dimensions.iter_mut().find(|(options, _)| *options == list) {
            Some((_, slots)) => slots.push(slot),
            None => dimensions.push((list, vec![slot])),
        }
    }

    let mut variants = vec![vec![ItemDescriptorCount::empty(); options.len()]];
    for (list, slots) in &dimensions {
        let mut combined = Vec::with_capacity(variants.len() * list.len());
        for variant in &variants {
            for choice in *list {
                let mut variant = variant.clone();
                for &slot in slots {
                    variant[slot] = choice.clone();
                }
                combined.push(variant);
            }
        }
        variants = combined;
    }

    variants
}

fn owned_ingredient_options(ingredient: &OwnedRecipeIngredient) -> Vec<ItemDescriptorCount> {
    match ingredient {
        OwnedRecipeIngredient::Simple(identifier) => {
            item_descriptor(identifier).into_iter().collect()
        }
        OwnedRecipeIngredient::Tagged(tag) => {
            let tag = tag.strip_prefix('#').unwrap_or(tag);
            if NATIVE_BEDROCK_TAGS.contains(&tag) {
                return vec![ItemDescriptorCount::tag(tag.to_string())];
            }

            pumpkin_data::tag::get_tag_ids(pumpkin_data::tag::RegistryKey::Item, tag)
                .into_iter()
                .flatten()
                .filter_map(|&id| Item::from_id(id))
                .filter_map(|item| item_descriptor(item.registry_key))
                .fold(Vec::new(), push_unique)
        }
        OwnedRecipeIngredient::OneOf(identifiers) => identifiers
            .iter()
            .filter_map(|identifier| item_descriptor(identifier))
            .fold(Vec::new(), push_unique),
    }
}

fn owned_output_descriptor(result: &OwnedRecipeResult) -> Option<NetworkItemDescriptor> {
    let item = Item::from_registry_key(&result.item_id)?;
    let mapping = JavaToBedrockItemMapping::from_java_item_id(item.id)?;
    Some(NetworkItemDescriptor {
        id: VarInt::from(mapping.bedrock_item.id),
        stack_size: u16::from(result.count),
        aux_value: VarUInt(mapping.bedrock_data),
        block_runtime_id: VarInt::from(mapping.bedrock_block_state),
        nbt_data: pumpkin_nbt::Nbt::default(),
        place_on_blocks: Vec::new(),
        destroy_blocks: Vec::new(),
        shield_blocking_tick: 0,
    })
}

fn append_dynamic_crafting_recipe(
    recipes: &mut Vec<BedrockRecipe>,
    recipe: &OwnedCraftingRecipe,
    network_id: &mut u32,
) {
    match recipe {
        OwnedCraftingRecipe::Shaped {
            recipe_id,
            key,
            pattern,
            result,
            ..
        } => {
            let Ok(height) = i32::try_from(pattern.len()) else {
                return;
            };
            let Ok(width) = i32::try_from(pattern.iter().map(String::len).max().unwrap_or(0))
            else {
                return;
            };
            let Ok(width_usize) = usize::try_from(width) else {
                return;
            };
            let mut options = Vec::new();
            for row in pattern {
                for column in 0..width_usize {
                    let symbol = row.chars().nth(column).unwrap_or(' ');
                    options.push(if symbol == ' ' {
                        vec![ItemDescriptorCount::empty()]
                    } else {
                        key.iter()
                            .find(|(k, _)| *k == symbol)
                            .map_or_else(Vec::new, |(_, ingredient)| {
                                owned_ingredient_options(ingredient)
                            })
                    });
                }
            }

            let Some(output) = owned_output_descriptor(result) else {
                return;
            };
            for input in ingredient_variants(&options) {
                let id_str = recipe_id
                    .clone()
                    .unwrap_or_else(|| format!("pumpkin:dynamic_{network_id}"));
                recipes.push(BedrockRecipe::Shaped(BedrockShapedRecipe {
                    recipe_id: id_str,
                    width: VarInt(width),
                    height: VarInt(height),
                    input,
                    output: vec![output.clone()],
                    uuid: Uuid::from_u128(u128::from(*network_id)),
                    block: "crafting_table".to_string(),
                    priority: VarInt(1),
                    assume_symmetry: true,
                    unlock_requirement: RecipeUnlockRequirement { context: 1 },
                    recipe_network_id: VarUInt(*network_id),
                }));
                *network_id += 1;
            }
        }
        OwnedCraftingRecipe::Shapeless {
            recipe_id,
            ingredients,
            result,
            ..
        } => {
            let options = ingredients
                .iter()
                .map(owned_ingredient_options)
                .collect::<Vec<_>>();
            let Some(output) = owned_output_descriptor(result) else {
                return;
            };
            for input in ingredient_variants(&options) {
                let id_str = recipe_id
                    .clone()
                    .unwrap_or_else(|| format!("pumpkin:dynamic_{network_id}"));
                recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                    recipe_id: id_str,
                    input,
                    output: vec![output.clone()],
                    uuid: Uuid::from_u128(u128::from(*network_id)),
                    block: "crafting_table".to_string(),
                    priority: VarInt(1),
                    unlock_requirement: RecipeUnlockRequirement { context: 1 },
                    recipe_network_id: VarUInt(*network_id),
                }));
                *network_id += 1;
            }
        }
    }
}

fn append_dynamic_stonecutter_recipe(
    recipes: &mut Vec<BedrockRecipe>,
    recipe: &OwnedStonecutterRecipe,
    network_id: &mut u32,
) {
    let options = owned_ingredient_options(&recipe.ingredient);
    let Some(output) = owned_output_descriptor(&recipe.result) else {
        return;
    };
    for input in ingredient_variants(&[options]) {
        recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
            recipe_id: recipe.recipe_id.clone(),
            input,
            output: vec![output.clone()],
            uuid: Uuid::from_u128(u128::from(*network_id)),
            block: "stonecutter".to_string(),
            priority: VarInt(1),
            unlock_requirement: RecipeUnlockRequirement { context: 1 },
            recipe_network_id: VarUInt(*network_id),
        }));
        *network_id += 1;
    }
}

fn append_dynamic_cooking_recipe(
    recipes: &mut Vec<BedrockRecipe>,
    recipe: &OwnedCookingRecipeType,
    network_id: &mut u32,
) {
    let (inner, blocks): (&OwnedCookingRecipe, &[&str]) = match recipe {
        OwnedCookingRecipeType::Blasting(inner) => (inner, &["blast_furnace"]),
        OwnedCookingRecipeType::Smelting(inner) => (inner, &["furnace"]),
        OwnedCookingRecipeType::Smoking(inner) => (inner, &["smoker"]),
        OwnedCookingRecipeType::CampfireCooking(inner) => (inner, &["campfire", "soul_campfire"]),
    };
    let options = owned_ingredient_options(&inner.ingredient);
    let Some(output) = owned_output_descriptor(&inner.result) else {
        return;
    };
    let variants = ingredient_variants(&[options]);
    for &block in blocks {
        for input in &variants {
            recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                recipe_id: inner.recipe_id.clone(),
                input: input.clone(),
                output: vec![output.clone()],
                uuid: Uuid::from_u128(u128::from(*network_id)),
                block: block.to_string(),
                priority: VarInt(0),
                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                recipe_network_id: VarUInt(*network_id),
            }));
            *network_id += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_protocol::{
        bedrock::client::{ItemDescriptorCount, RecipeItemDescriptor},
        codec::recipe::OwnedRecipeIngredient,
    };

    use super::{ingredient_variants, owned_ingredient_options, stonecutter_output_for_recipe_id};

    fn item(identifier: &str) -> ItemDescriptorCount {
        ItemDescriptorCount::item(identifier.to_string(), 0)
    }

    fn identifiers(variant: &[ItemDescriptorCount]) -> Vec<&str> {
        variant
            .iter()
            .map(|option| match &option.descriptor {
                RecipeItemDescriptor::Item { identifier, .. } => identifier.as_str(),
                other => panic!("expected an item descriptor, got {other:?}"),
            })
            .collect()
    }

    #[test]
    fn native_planks_tag_is_preserved() {
        let options =
            owned_ingredient_options(&OwnedRecipeIngredient::Tagged("#minecraft:planks".into()));

        assert_eq!(options.len(), 1);
        assert!(matches!(
            options.first().map(|option| &option.descriptor),
            Some(RecipeItemDescriptor::Tag(tag)) if tag == "minecraft:planks"
        ));
    }

    #[test]
    fn java_only_log_tag_is_expanded() {
        let options =
            owned_ingredient_options(&OwnedRecipeIngredient::Tagged("#minecraft:oak_logs".into()));
        let identifiers = options
            .iter()
            .filter_map(|option| match &option.descriptor {
                RecipeItemDescriptor::Item { identifier, .. } => Some(identifier.as_str()),
                RecipeItemDescriptor::Empty | RecipeItemDescriptor::Tag(_) => None,
            })
            .collect::<Vec<_>>();

        assert!(identifiers.contains(&"minecraft:oak_log"));
        assert!(identifiers.contains(&"minecraft:stripped_oak_log"));
        assert!(identifiers.contains(&"minecraft:oak_wood"));
        assert!(identifiers.contains(&"minecraft:stripped_oak_wood"));
    }

    #[test]
    fn repeated_wood_type_stays_aligned_across_variants() {
        let options =
            owned_ingredient_options(&OwnedRecipeIngredient::Tagged("#minecraft:oak_logs".into()));
        let variants = ingredient_variants(&[options.clone(), options]);

        assert!(variants.len() > 1);
        assert!(
            variants
                .iter()
                .all(|variant| variant.first() == variant.get(1))
        );
    }

    #[test]
    fn distinct_ingredients_produce_every_combination() {
        let logs = vec![
            item("minecraft:oak_log"),
            item("minecraft:birch_log"),
            item("minecraft:spruce_log"),
        ];
        let planks = vec![item("minecraft:oak_planks"), item("minecraft:birch_planks")];

        let variants = ingredient_variants(&[logs, planks]);

        let combinations = variants
            .iter()
            .map(|variant| identifiers(variant))
            .collect::<Vec<_>>();
        assert_eq!(combinations.len(), 6);
        for log in [
            "minecraft:oak_log",
            "minecraft:birch_log",
            "minecraft:spruce_log",
        ] {
            for planks in ["minecraft:oak_planks", "minecraft:birch_planks"] {
                assert!(
                    combinations.contains(&vec![log, planks]),
                    "missing combination of {log} with {planks}"
                );
            }
        }
    }

    #[test]
    fn static_stonecutter_recipe_id_resolves_to_bedrock_output() {
        let first = pumpkin_data::bedrock_recipe::STONECUTTER_RECIPES[0];
        let resolved = stonecutter_output_for_recipe_id(None, first.network_id);
        assert_eq!(
            resolved,
            Some((first.output_item_id, first.output_aux_value))
        );
        assert_eq!(stonecutter_output_for_recipe_id(None, 0), None);
    }
}
