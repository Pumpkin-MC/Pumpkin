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

pub fn crafting_data() -> &'static [BedrockRecipe] {
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

fn ingredient_variants(options: &[Vec<ItemDescriptorCount>]) -> Vec<Vec<ItemDescriptorCount>> {
    if options.iter().any(Vec::is_empty) {
        return Vec::new();
    }

    let variant_count = options.iter().map(Vec::len).max().unwrap_or(0);
    (0..variant_count)
        .map(|variant| {
            options
                .iter()
                .map(|options| options.get(variant).unwrap_or(&options[0]).clone())
                .collect()
        })
        .collect()
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

    recipes
}

#[cfg(test)]
mod tests {
    use pumpkin_data::recipes::RecipeIngredientTypes;
    use pumpkin_protocol::bedrock::client::RecipeItemDescriptor;

    use super::{ingredient_options, ingredient_variants};

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
}
