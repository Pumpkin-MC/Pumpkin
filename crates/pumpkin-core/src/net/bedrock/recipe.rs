use std::sync::OnceLock;

use pumpkin_data::{
    item::{Item, JavaToBedrockItemMapping},
    recipes::{CraftingRecipeTypes, RecipeIngredientTypes, RecipeResultStruct},
};
use pumpkin_protocol::{
    bedrock::{
        client::{
            BedrockRecipe, BedrockShapedRecipe, BedrockShapelessRecipe, ItemDescriptorCount,
            RecipeUnlockRequirement,
        },
        network_item::NetworkItemDescriptor,
    },
    codec::{var_int::VarInt, var_uint::VarUInt},
};
use uuid::Uuid;

// These Java tags also exist in Bedrock 1.26.40. Java-only tags are expanded below.
const NATIVE_WOOD_TAGS: &[&str] = &[
    "minecraft:crimson_stems",
    "minecraft:logs",
    "minecraft:logs_that_burn",
    "minecraft:mangrove_logs",
    "minecraft:planks",
    "minecraft:warped_stems",
    "minecraft:wooden_slabs",
];

pub fn crafting_data(recipe_manager: &crate::server::RecipeManager) -> Vec<BedrockRecipe> {
    let mut recipes = static_recipes().to_vec();
    let dynamic = recipe_manager.get_dynamic_recipes_internal();
    if !dynamic.is_empty() {
        let mut network_id = (recipes.len() + 1) as u32;
        for recipe in dynamic {
            if let pumpkin_protocol::codec::recipe::DynamicRecipe::Crafting(crafting) = recipe {
                append_dynamic_crafting_recipe(&mut recipes, &crafting, &mut network_id);
            }
        }
    }
    recipes
}

pub fn static_recipes() -> &'static [BedrockRecipe] {
    static RECIPES: OnceLock<Vec<BedrockRecipe>> = OnceLock::new();
    RECIPES.get_or_init(build).as_slice()
}

fn item_descriptor(identifier: &str) -> Option<ItemDescriptorCount> {
    let item = Item::from_registry_key(identifier)?;
    let mapping = JavaToBedrockItemMapping::from_java_item_id(item.id)?;
    Some(ItemDescriptorCount::item(
        mapping.bedrock_item.registry_key.to_string(),
        mapping.bedrock_data as i32,
    ))
}

fn ingredient_options(ingredient: &RecipeIngredientTypes) -> Vec<ItemDescriptorCount> {
    match ingredient {
        RecipeIngredientTypes::Simple(identifier) => {
            item_descriptor(identifier).into_iter().collect()
        }
        RecipeIngredientTypes::Tagged(tag) => {
            let tag = tag.strip_prefix('#').unwrap_or(tag);
            if NATIVE_WOOD_TAGS.contains(&tag) {
                return vec![ItemDescriptorCount::tag(tag.to_string())];
            }

            pumpkin_data::tag::get_tag_ids(pumpkin_data::tag::RegistryKey::Item, tag)
                .into_iter()
                .flatten()
                .filter_map(|&id| Item::from_id(id))
                .filter_map(|item| item_descriptor(item.registry_key))
                .fold(Vec::new(), push_unique)
        }
        RecipeIngredientTypes::OneOf(identifiers) => identifiers
            .iter()
            .filter_map(|identifier| item_descriptor(identifier))
            .fold(Vec::new(), push_unique),
    }
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

fn output_descriptor(result: &RecipeResultStruct) -> Option<NetworkItemDescriptor> {
    let item = Item::from_registry_key(result.id)?;
    let mapping = JavaToBedrockItemMapping::from_java_item_id(item.id)?;
    Some(NetworkItemDescriptor {
        id: VarInt::from(mapping.bedrock_item.id),
        stack_size: result.count as u16,
        aux_value: VarUInt(mapping.bedrock_data),
        block_runtime_id: VarInt::from(mapping.bedrock_block_state),
        nbt_data: pumpkin_nbt::Nbt::default(),
        place_on_blocks: Vec::new(),
        destroy_blocks: Vec::new(),
        shield_blocking_tick: 0,
    })
}

fn build() -> Vec<BedrockRecipe> {
    let mut recipes = Vec::new();
    let mut network_id = 1u32;

    for recipe in pumpkin_data::recipes::RECIPES_CRAFTING {
        match recipe {
            CraftingRecipeTypes::CraftingShaped {
                key,
                pattern,
                result,
                ..
            } => {
                let height = pattern.len() as i32;
                let width = pattern.iter().map(|row| row.len()).max().unwrap_or(0) as i32;
                let mut options = Vec::new();
                for row in *pattern {
                    for column in 0..width as usize {
                        let symbol = row.chars().nth(column).unwrap_or(' ');
                        options.push(if symbol == ' ' {
                            vec![ItemDescriptorCount::empty()]
                        } else {
                            key.iter()
                                .find(|(key, _)| *key == symbol)
                                .map_or_else(Vec::new, |(_, ingredient)| {
                                    ingredient_options(ingredient)
                                })
                        });
                    }
                }

                let Some(output) = output_descriptor(result) else {
                    continue;
                };
                for input in ingredient_variants(&options) {
                    recipes.push(BedrockRecipe::Shaped(BedrockShapedRecipe {
                        recipe_id: format!("pumpkin:recipe_{network_id}"),
                        width: VarInt(width),
                        height: VarInt(height),
                        input,
                        output: vec![output.clone()],
                        uuid: Uuid::from_u128(u128::from(network_id)),
                        block: "crafting_table".to_string(),
                        priority: VarInt(1),
                        assume_symmetry: true,
                        unlock_requirement: RecipeUnlockRequirement { context: 1 },
                        recipe_network_id: VarUInt(network_id),
                    }));
                    network_id += 1;
                }
            }
            CraftingRecipeTypes::CraftingShapeless {
                ingredients,
                result,
                ..
            } => {
                let options = ingredients
                    .iter()
                    .map(ingredient_options)
                    .collect::<Vec<_>>();
                let Some(output) = output_descriptor(result) else {
                    continue;
                };
                for input in ingredient_variants(&options) {
                    recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                        recipe_id: format!("pumpkin:recipe_{network_id}"),
                        input,
                        output: vec![output.clone()],
                        uuid: Uuid::from_u128(u128::from(network_id)),
                        block: "crafting_table".to_string(),
                        priority: VarInt(1),
                        unlock_requirement: RecipeUnlockRequirement { context: 1 },
                        recipe_network_id: VarUInt(network_id),
                    }));
                    network_id += 1;
                }
            }
            _ => {}
        }
    }

    for recipe in pumpkin_data::recipes::RECIPES_STONECUTTING {
        let options = ingredient_options(&recipe.ingredient);
        let Some(output) = output_descriptor(&recipe.result) else {
            continue;
        };
        for input in ingredient_variants(&[options]) {
            recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                recipe_id: format!("pumpkin:stonecutter_{network_id}"),
                input,
                output: vec![output.clone()],
                uuid: Uuid::from_u128(u128::from(network_id)),
                block: "stonecutter".to_string(),
                priority: VarInt(1),
                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                recipe_network_id: VarUInt(network_id),
            }));
            network_id += 1;
        }
    }

    recipes
}

fn owned_ingredient_options(
    ingredient: &pumpkin_protocol::codec::recipe::OwnedRecipeIngredient,
) -> Vec<ItemDescriptorCount> {
    match ingredient {
        pumpkin_protocol::codec::recipe::OwnedRecipeIngredient::Simple(identifier) => {
            item_descriptor(identifier).into_iter().collect()
        }
        pumpkin_protocol::codec::recipe::OwnedRecipeIngredient::Tagged(tag) => {
            let tag = tag.strip_prefix('#').unwrap_or(tag);
            if NATIVE_WOOD_TAGS.contains(&tag) {
                return vec![ItemDescriptorCount::tag(tag.to_string())];
            }

            pumpkin_data::tag::get_tag_ids(pumpkin_data::tag::RegistryKey::Item, tag)
                .into_iter()
                .flatten()
                .filter_map(|&id| Item::from_id(id))
                .filter_map(|item| item_descriptor(item.registry_key))
                .fold(Vec::new(), push_unique)
        }
        pumpkin_protocol::codec::recipe::OwnedRecipeIngredient::OneOf(identifiers) => identifiers
            .iter()
            .filter_map(|identifier| item_descriptor(identifier))
            .fold(Vec::new(), push_unique),
    }
}

fn owned_output_descriptor(
    result: &pumpkin_protocol::codec::recipe::OwnedRecipeResult,
) -> Option<NetworkItemDescriptor> {
    let item = Item::from_registry_key(&result.item_id)?;
    let mapping = JavaToBedrockItemMapping::from_java_item_id(item.id)?;
    Some(NetworkItemDescriptor {
        id: VarInt::from(mapping.bedrock_item.id),
        stack_size: result.count as u16,
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
    recipe: &pumpkin_protocol::codec::recipe::OwnedCraftingRecipe,
    network_id: &mut u32,
) {
    match recipe {
        pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shaped {
            recipe_id,
            key,
            pattern,
            result,
            ..
        } => {
            let height = pattern.len() as i32;
            let width = pattern.iter().map(String::len).max().unwrap_or(0) as i32;
            let mut options = Vec::new();
            for row in pattern {
                for column in 0..width as usize {
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
        pumpkin_protocol::codec::recipe::OwnedCraftingRecipe::Shapeless {
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

#[cfg(test)]
mod tests {
    use pumpkin_data::recipes::RecipeIngredientTypes;
    use pumpkin_protocol::bedrock::client::{ItemDescriptorCount, RecipeItemDescriptor};

    use super::{ingredient_options, ingredient_variants};

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
        let options = ingredient_options(&RecipeIngredientTypes::Tagged("#minecraft:planks"));

        assert_eq!(options.len(), 1);
        assert!(matches!(
            options.first().map(|option| &option.descriptor),
            Some(RecipeItemDescriptor::Tag(tag)) if tag == "minecraft:planks"
        ));
    }

    #[test]
    fn java_only_log_tag_is_expanded() {
        let options = ingredient_options(&RecipeIngredientTypes::Tagged("#minecraft:oak_logs"));
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
        let options = ingredient_options(&RecipeIngredientTypes::Tagged("#minecraft:oak_logs"));
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
}
