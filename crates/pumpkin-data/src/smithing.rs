use crate::item::Item;
use crate::tag::Taggable;
use crate::trim::{TrimMaterial, TrimPattern};

// Transcribed from data/minecraft/recipe/netherite_*_smithing.json: each recipe requires
// the netherite upgrade template, a specific diamond base item, and an item tagged
// `minecraft:netherite_tool_materials` (only `minecraft:netherite_ingot`) as the addition.
pub struct SmithingTransformRecipe {
    pub base: &'static Item,
    pub result: &'static Item,
}

pub static SMITHING_TRANSFORM_RECIPES: &[SmithingTransformRecipe] = &[
    SmithingTransformRecipe {
        base: &Item::DIAMOND_SWORD,
        result: &Item::NETHERITE_SWORD,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_PICKAXE,
        result: &Item::NETHERITE_PICKAXE,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_AXE,
        result: &Item::NETHERITE_AXE,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_SHOVEL,
        result: &Item::NETHERITE_SHOVEL,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_HOE,
        result: &Item::NETHERITE_HOE,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_HELMET,
        result: &Item::NETHERITE_HELMET,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_CHESTPLATE,
        result: &Item::NETHERITE_CHESTPLATE,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_LEGGINGS,
        result: &Item::NETHERITE_LEGGINGS,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_BOOTS,
        result: &Item::NETHERITE_BOOTS,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_SPEAR,
        result: &Item::NETHERITE_SPEAR,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_NAUTILUS_ARMOR,
        result: &Item::NETHERITE_NAUTILUS_ARMOR,
    },
    SmithingTransformRecipe {
        base: &Item::DIAMOND_HORSE_ARMOR,
        result: &Item::NETHERITE_HORSE_ARMOR,
    },
];

#[must_use]
pub fn find_transform_recipe(base: &Item) -> Option<&'static SmithingTransformRecipe> {
    SMITHING_TRANSFORM_RECIPES
        .iter()
        .find(|recipe| recipe.base == base)
}

#[must_use]
pub fn is_template_item(item: &Item) -> bool {
    item == &Item::NETHERITE_UPGRADE_SMITHING_TEMPLATE
}

#[must_use]
pub fn is_transform_addition(item: &Item) -> bool {
    item.is_tagged_with("minecraft:netherite_tool_materials")
        .unwrap_or(false)
}

// Transcribed from data/minecraft/recipe/*_armor_trim_smithing_template_smithing_trim.json:
// template item selects the pattern, base must be tagged `minecraft:trimmable_armor`, and
// addition must be tagged `minecraft:trim_materials`.
#[must_use]
pub fn is_trimmable_base(item: &Item) -> bool {
    item.is_tagged_with("minecraft:trimmable_armor")
        .unwrap_or(false)
}

#[must_use]
pub fn is_trim_addition(item: &Item) -> bool {
    item.is_tagged_with("minecraft:trim_materials")
        .unwrap_or(false)
}

#[must_use]
pub fn trim_pattern_for_template(item: &Item) -> Option<TrimPattern> {
    TrimPattern::from_template_item(item)
}

#[must_use]
pub fn trim_material_for_addition(item: &Item) -> Option<TrimMaterial> {
    TrimMaterial::from_item(item)
}
