use std::{collections::HashMap, io::Write};

use pumpkin_data::item::Item;
use pumpkin_data::item_id_remap::remap_item_id_for_version;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::packet::clientbound::PLAY_RECIPE_BOOK_ADD;
use pumpkin_data::recipes::{
    CookingRecipeType, CraftingRecipeTypes, RECIPES_COOKING, RECIPES_CRAFTING, RecipeCategoryTypes,
    RecipeIngredientTypes, RecipeResultStruct,
};
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

use crate::{
    ClientPacket, VarInt, WritingError, codec::item_stack_seralizer::ItemStackSerializer,
    ser::NetworkWriteExt,
};

// Recipe Display type IDs
const RECIPE_DISPLAY_SHAPELESS: i32 = 0;
const RECIPE_DISPLAY_SHAPED: i32 = 1;
const RECIPE_DISPLAY_FURNACE: i32 = 2;

// Slot Display type IDs
const SLOT_DISPLAY_EMPTY: i32 = 0;
const SLOT_DISPLAY_ANY_FUEL: i32 = 1;
// 1.21.2 - 1.21.11
const SLOT_DISPLAY_ITEM_LEGACY: i32 = 2;
const SLOT_DISPLAY_ITEM_STACK_LEGACY: i32 = 3;
const SLOT_DISPLAY_COMPOSITE_LEGACY: i32 = 7;
// 26.1+
const SLOT_DISPLAY_ITEM_26_1: i32 = 4;
const SLOT_DISPLAY_ITEM_STACK_26_1: i32 = 5;
const SLOT_DISPLAY_COMPOSITE_26_1: i32 = 10;

const ENTRY_FLAG_NOTIFICATION: u8 = 0x01;
const ENTRY_FLAG_HIGHLIGHT: u8 = 0x02;

// RecipeBookCategory IDs
const CATEGORY_CRAFTING_BUILDING: i32 = 0;
const CATEGORY_CRAFTING_REDSTONE: i32 = 1;
const CATEGORY_CRAFTING_EQUIPMENT: i32 = 2;
const CATEGORY_CRAFTING_MISC: i32 = 3;
const CATEGORY_FURNACE_FOOD: i32 = 4;
const CATEGORY_FURNACE_BLOCKS: i32 = 5;
const CATEGORY_FURNACE_MISC: i32 = 6;
const CATEGORY_BLAST_FURNACE_BLOCKS: i32 = 7;
const CATEGORY_BLAST_FURNACE_MISC: i32 = 8;
const CATEGORY_SMOKER_FOOD: i32 = 9;
const CATEGORY_CAMPFIRE: i32 = 12;

/// Clientbound packet that adds recipes to the client's recipe book.
/// `replace = true` means the client replaces its current recipe list.
#[java_packet(PLAY_RECIPE_BOOK_ADD)]
pub struct CRecipeBookAdd {
    pub replace: bool,
}

impl CRecipeBookAdd {
    #[must_use]
    pub const fn new(replace: bool) -> Self {
        Self { replace }
    }
}

fn item_id_versioned(item: &Item, version: MinecraftVersion) -> i32 {
    remap_item_id_for_version(item.id, version) as i32
}

fn slot_display_item_type(version: MinecraftVersion) -> i32 {
    if version >= MinecraftVersion::V_26_1 {
        SLOT_DISPLAY_ITEM_26_1
    } else {
        SLOT_DISPLAY_ITEM_LEGACY
    }
}

fn slot_display_composite_type(version: MinecraftVersion) -> i32 {
    if version >= MinecraftVersion::V_26_1 {
        SLOT_DISPLAY_COMPOSITE_26_1
    } else {
        SLOT_DISPLAY_COMPOSITE_LEGACY
    }
}

fn slot_display_item_stack_type(version: MinecraftVersion) -> i32 {
    if version >= MinecraftVersion::V_26_1 {
        SLOT_DISPLAY_ITEM_STACK_26_1
    } else {
        SLOT_DISPLAY_ITEM_STACK_LEGACY
    }
}

fn write_item_slot_display(
    write: &mut impl Write,
    item: &Item,
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(slot_display_item_type(version)))?;
    write.write_var_int(&VarInt(item_id_versioned(item, version)))?;
    Ok(())
}

fn write_item_stack_slot_display(
    write: &mut impl Write,
    item: &Item,
    count: u8,
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(slot_display_item_stack_type(version)))?;
    let static_item = Item::from_id(item.id)
        .ok_or_else(|| WritingError::Message(format!("item id {} must exist", item.id)))?;
    ItemStackSerializer::from(ItemStack::new(count, static_item))
        .write_with_version(write, &version)
}

fn write_empty_slot_display(write: &mut impl Write) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(SLOT_DISPLAY_EMPTY))?;
    Ok(())
}

fn write_any_fuel_slot_display(write: &mut impl Write) -> Result<(), WritingError> {
    write.write_var_int(&VarInt(SLOT_DISPLAY_ANY_FUEL))?;
    Ok(())
}

fn write_ingredient_slot_display(
    write: &mut impl Write,
    ingredient: &RecipeIngredientTypes,
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    match ingredient {
        RecipeIngredientTypes::Simple(id) => {
            let key = id.strip_prefix("minecraft:").unwrap_or(id);
            if let Some(item) = Item::from_registry_key(key) {
                write_item_slot_display(write, item, version)?;
            } else {
                write_empty_slot_display(write)?;
            }
        }
        RecipeIngredientTypes::Tagged(_tag) => {
            // TODO: We lack registry access here to resolve tags to a TagSlotDisplay.
            // Sending an empty slot prevents a client DecoderException, but will
            // result in invisible ingredients in the recipe book.
            write_empty_slot_display(write)?;
        }
        RecipeIngredientTypes::OneOf(ids) => {
            let mut items: Vec<&Item> = Vec::new();
            for id in *ids {
                let key = id.strip_prefix("minecraft:").unwrap_or(id);
                if let Some(item) = Item::from_registry_key(key) {
                    items.push(item);
                }
            }
            if items.is_empty() {
                write_empty_slot_display(write)?;
            } else if items.len() == 1 {
                write_item_slot_display(write, items[0], version)?;
            } else {
                write.write_var_int(&VarInt(slot_display_composite_type(version)))?;
                write.write_var_int(&VarInt(items.len() as i32))?;
                for item in &items {
                    write_item_slot_display(write, item, version)?;
                }
            }
        }
    }
    Ok(())
}

/// Write a single Ingredient as a `HolderSet`<Item> for craftingRequirements.
///
/// Vanilla wire format for `ByteBufCodecs.holderSet(Registries.ITEM)`:
///   VarInt(0)     -> named tag reference (followed by `ResourceLocation`)
///   VarInt(n + 1) -> direct list of n item IDs
///
/// So an empty/absent ingredient writes VarInt(1), one item writes VarInt(2) + id, etc.
fn write_ingredient_holderset(
    write: &mut impl Write,
    ingredient: Option<&RecipeIngredientTypes>,
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    match ingredient {
        // Empty ingredient slot -> direct list of 0 items -> VarInt(0 + 1) = VarInt(1)
        None => {
            write.write_var_int(&VarInt(1))?;
        }
        Some(RecipeIngredientTypes::Simple(id)) => {
            let key = id.strip_prefix("minecraft:").unwrap_or(id);
            if let Some(item) = Item::from_registry_key(key) {
                // 1 item -> VarInt(1 + 1) = VarInt(2)
                write.write_var_int(&VarInt(2))?;
                write.write_var_int(&VarInt(item_id_versioned(item, version)))?;
            } else {
                // Item not found -> empty direct list
                write.write_var_int(&VarInt(1))?;
            }
        }
        Some(RecipeIngredientTypes::Tagged(_tag)) => {
            // No current recipes use Tagged; write empty direct list.
            write.write_var_int(&VarInt(1))?;
        }
        Some(RecipeIngredientTypes::OneOf(ids)) => {
            let items: Vec<i32> = ids
                .iter()
                .filter_map(|id| {
                    let key = id.strip_prefix("minecraft:").unwrap_or(id);
                    Item::from_registry_key(key).map(|item| item_id_versioned(item, version))
                })
                .collect();
            // n items -> VarInt(n + 1)
            write.write_var_int(&VarInt(items.len() as i32 + 1))?;
            for id in &items {
                write.write_var_int(&VarInt(*id))?;
            }
        }
    }
    Ok(())
}

/// Write the `craftingRequirements: Option<List<Ingredient>>` field (present).
/// Each slot is either `None` (empty grid cell) or `Some(ingredient)`.
fn write_crafting_requirements(
    write: &mut impl Write,
    slots: &[Option<&RecipeIngredientTypes>],
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    write.write_bool(true)?; // present
    write.write_var_int(&VarInt(slots.len() as i32))?;
    for slot in slots {
        write_ingredient_holderset(write, *slot, version)?;
    }
    Ok(())
}

fn write_result_slot_display(
    write: &mut impl Write,
    result: &RecipeResultStruct,
    version: MinecraftVersion,
) -> Result<(), WritingError> {
    let key = result.id.strip_prefix("minecraft:").unwrap_or(result.id);
    if let Some(item) = Item::from_registry_key(key) {
        write_item_stack_slot_display(write, item, result.count, version)?;
    } else {
        write_empty_slot_display(write)?;
    }
    Ok(())
}

fn write_optional_var_int(write: &mut impl Write, value: Option<i32>) -> Result<(), WritingError> {
    let encoded = value.map_or(Ok(0), |v| {
        v.checked_add(1)
            .ok_or_else(|| WritingError::Message(format!("group id {v} overflow")))
    })?;
    write.write_var_int(&VarInt(encoded))?;
    Ok(())
}

fn resolve_group_id(
    group_ids: &mut HashMap<&'static str, i32>,
    next_group_id: &mut i32,
    group: Option<&'static str>,
) -> Option<i32> {
    let key = group?;
    Some(*group_ids.entry(key).or_insert_with(|| {
        let id = *next_group_id;
        *next_group_id += 1;
        id
    }))
}

const fn entry_flags(replace: bool, notification: bool, highlight: bool) -> u8 {
    if replace {
        return 0;
    }

    (if notification {
        ENTRY_FLAG_NOTIFICATION
    } else {
        0
    }) | (if highlight { ENTRY_FLAG_HIGHLIGHT } else { 0 })
}

const fn crafting_category(cat: &RecipeCategoryTypes) -> i32 {
    match cat {
        RecipeCategoryTypes::Equipment => CATEGORY_CRAFTING_EQUIPMENT,
        RecipeCategoryTypes::Building | RecipeCategoryTypes::Blocks => CATEGORY_CRAFTING_BUILDING,
        RecipeCategoryTypes::Restone => CATEGORY_CRAFTING_REDSTONE,
        RecipeCategoryTypes::Food | RecipeCategoryTypes::Misc => CATEGORY_CRAFTING_MISC,
    }
}

/// Write a single `RecipeDisplayEntry` + flags byte.
/// Returns `Ok(true)` if written, `Ok(false)` if skipped (special recipe).
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn write_entry(
    write: &mut impl Write,
    display_id: i32,
    version: MinecraftVersion,
    group_id: Option<i32>,
    flags: u8,
    crafting_table: &Item,
    furnace: &Item,
    blast_furnace: &Item,
    smoker: &Item,
    campfire: &Item,
    crafting_recipe: Option<&CraftingRecipeTypes>,
    cooking_recipe: Option<(&CookingRecipeType, i32)>,
) -> Result<bool, WritingError> {
    if let Some(recipe) = crafting_recipe {
        match recipe {
            CraftingRecipeTypes::CraftingShaped {
                category,
                pattern,
                key,
                result,
                ..
            } => {
                // Compute width and height from pattern
                let height = pattern.len() as i32;
                let width = pattern.first().map_or(0, |r| r.len()) as i32;

                // RecipeDisplayId
                write.write_var_int(&VarInt(display_id))?;
                // RecipeDisplay type = shaped (1)
                write.write_var_int(&VarInt(RECIPE_DISPLAY_SHAPED))?;
                // width, height
                write.write_var_int(&VarInt(width))?;
                write.write_var_int(&VarInt(height))?;
                // ingredients: flat list, row by row
                write.write_var_int(&VarInt(width * height))?;
                for row in *pattern {
                    for ch in row.chars() {
                        if ch == ' ' {
                            write_empty_slot_display(write)?;
                        } else if let Some((_, ingredient)) = key.iter().find(|(k, _)| *k == ch) {
                            write_ingredient_slot_display(write, ingredient, version)?;
                        } else {
                            write_empty_slot_display(write)?;
                        }
                    }
                }
                // result
                write_result_slot_display(write, result, version)?;
                // craftingStation
                write_item_slot_display(write, crafting_table, version)?;
                // group: OptionalVarInt
                write_optional_var_int(write, group_id)?;
                // category
                write.write_var_int(&VarInt(crafting_category(category)))?;
                // craftingRequirements: one HolderSet per non-empty grid slot
                // (Ingredient cannot be empty, so empty slots must be excluded)
                {
                    let mut slots: Vec<Option<&RecipeIngredientTypes>> = Vec::new();
                    for row in *pattern {
                        for ch in row.chars() {
                            if ch != ' '
                                && let Some((_, ing)) = key.iter().find(|(k, _)| *k == ch)
                            {
                                slots.push(Some(ing));
                            }
                        }
                    }
                    write_crafting_requirements(write, &slots, version)?;
                };
                write.write_u8(flags)?;
            }
            CraftingRecipeTypes::CraftingShapeless {
                category,
                ingredients,
                result,
                ..
            } => {
                // RecipeDisplayId
                write.write_var_int(&VarInt(display_id))?;
                // RecipeDisplay type = shapeless (0)
                write.write_var_int(&VarInt(RECIPE_DISPLAY_SHAPELESS))?;
                // ingredients list
                write.write_var_int(&VarInt(ingredients.len() as i32))?;
                for ing in *ingredients {
                    write_ingredient_slot_display(write, ing, version)?;
                }
                // result
                write_result_slot_display(write, result, version)?;
                // craftingStation
                write_item_slot_display(write, crafting_table, version)?;
                // group: OptionalVarInt
                write_optional_var_int(write, group_id)?;
                // category
                write.write_var_int(&VarInt(crafting_category(category)))?;
                // craftingRequirements: one HolderSet per ingredient
                {
                    let slots: Vec<Option<&RecipeIngredientTypes>> =
                        ingredients.iter().map(Some).collect();
                    write_crafting_requirements(write, &slots, version)?;
                };
                write.write_u8(flags)?;
            }
            CraftingRecipeTypes::CraftingTransmute {
                category,
                input,
                material,
                result,
                ..
            } => {
                // Transmute shown as shapeless with 2 ingredients
                write.write_var_int(&VarInt(display_id))?;
                write.write_var_int(&VarInt(RECIPE_DISPLAY_SHAPELESS))?;
                // 2 ingredients
                write.write_var_int(&VarInt(2))?;
                write_ingredient_slot_display(write, input, version)?;
                write_ingredient_slot_display(write, material, version)?;
                write_result_slot_display(write, result, version)?;
                write_item_slot_display(write, crafting_table, version)?;
                write_optional_var_int(write, group_id)?;
                write.write_var_int(&VarInt(crafting_category(category)))?;
                // craftingRequirements: input + material
                write_crafting_requirements(write, &[Some(input), Some(material)], version)?;
                write.write_u8(flags)?;
            }
            // Skip special/decorated_pot recipes as they have no useful display
            CraftingRecipeTypes::CraftingDecoratedPot { .. }
            | CraftingRecipeTypes::CraftingSpecial => {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    if let Some((recipe, book_category)) = cooking_recipe {
        let (cooking, station) = match recipe {
            CookingRecipeType::Smelting(r) => (r, furnace),
            CookingRecipeType::Blasting(r) => (r, blast_furnace),
            CookingRecipeType::Smoking(r) => (r, smoker),
            CookingRecipeType::CampfireCooking(r) => (r, campfire),
        };

        write.write_var_int(&VarInt(display_id))?;
        // RecipeDisplay type = furnace (2)
        write.write_var_int(&VarInt(RECIPE_DISPLAY_FURNACE))?;
        // ingredient
        write_ingredient_slot_display(write, &cooking.ingredient, version)?;
        // fuel: AnyFuel
        write_any_fuel_slot_display(write)?;
        // result
        write_result_slot_display(write, &cooking.result, version)?;
        // craftingStation
        write_item_slot_display(write, station, version)?;
        // duration
        write.write_var_int(&VarInt(cooking.cookingtime))?;
        // experience
        write.write_f32_be(cooking.experience)?;
        // group: OptionalVarInt
        write_optional_var_int(write, group_id)?;
        // category
        write.write_var_int(&VarInt(book_category))?;
        // craftingRequirements: the single ingredient
        write_crafting_requirements(write, &[Some(&cooking.ingredient)], version)?;
        write.write_u8(flags)?;
        return Ok(true);
    }

    Ok(false)
}

impl ClientPacket for CRecipeBookAdd {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // Station items (these IDs are stable across all versions we support)
        let crafting_table = Item::from_registry_key("crafting_table")
            .ok_or_else(|| WritingError::Message("crafting_table item must exist".into()))?;
        let furnace = Item::from_registry_key("furnace")
            .ok_or_else(|| WritingError::Message("furnace item must exist".into()))?;
        let blast_furnace = Item::from_registry_key("blast_furnace")
            .ok_or_else(|| WritingError::Message("blast_furnace item must exist".into()))?;
        let smoker = Item::from_registry_key("smoker")
            .ok_or_else(|| WritingError::Message("smoker item must exist".into()))?;
        let campfire = Item::from_registry_key("campfire")
            .ok_or_else(|| WritingError::Message("campfire item must exist".into()))?;

        // First pass - count and skip CraftingSpecial and CraftingDecoratedPot entries.
        let crafting_count: usize = RECIPES_CRAFTING
            .iter()
            .filter(|r| {
                !matches!(
                    r,
                    CraftingRecipeTypes::CraftingSpecial
                        | CraftingRecipeTypes::CraftingDecoratedPot { .. }
                )
            })
            .count();
        let total = crafting_count + RECIPES_COOKING.len();

        // Entry count (VarInt)
        write.write_var_int(&VarInt(total as i32))?;

        let mut display_id: i32 = 0;
        let mut group_ids: HashMap<&'static str, i32> = HashMap::new();
        let mut next_group_id: i32 = 0;
        let highlight = !self.replace;

        // Write crafting recipes
        for recipe in RECIPES_CRAFTING {
            let (group, notification) = match recipe {
                CraftingRecipeTypes::CraftingShaped {
                    group,
                    show_notification,
                    ..
                } => (*group, *show_notification),
                CraftingRecipeTypes::CraftingShapeless { group, .. }
                | CraftingRecipeTypes::CraftingTransmute { group, .. } => (*group, true),
                CraftingRecipeTypes::CraftingDecoratedPot { .. }
                | CraftingRecipeTypes::CraftingSpecial => (None, true),
            };
            let group_id = resolve_group_id(&mut group_ids, &mut next_group_id, group);
            let flags = entry_flags(self.replace, notification, highlight);
            let written = write_entry(
                &mut write,
                display_id,
                *version,
                group_id,
                flags,
                crafting_table,
                furnace,
                blast_furnace,
                smoker,
                campfire,
                Some(recipe),
                None,
            )?;
            if written {
                display_id += 1;
            }
        }

        // Write cooking recipes
        for recipe in RECIPES_COOKING {
            let (book_category, group) = match recipe {
                CookingRecipeType::Smelting(r) => (
                    match r.category {
                        RecipeCategoryTypes::Food => CATEGORY_FURNACE_FOOD,
                        RecipeCategoryTypes::Blocks => CATEGORY_FURNACE_BLOCKS,
                        _ => CATEGORY_FURNACE_MISC,
                    },
                    r.group,
                ),
                CookingRecipeType::Blasting(r) => (
                    match r.category {
                        RecipeCategoryTypes::Blocks => CATEGORY_BLAST_FURNACE_BLOCKS,
                        _ => CATEGORY_BLAST_FURNACE_MISC,
                    },
                    r.group,
                ),
                CookingRecipeType::Smoking(r) => (CATEGORY_SMOKER_FOOD, r.group),
                CookingRecipeType::CampfireCooking(r) => (CATEGORY_CAMPFIRE, r.group),
            };
            let group_id = resolve_group_id(&mut group_ids, &mut next_group_id, group);
            let flags = entry_flags(self.replace, true, highlight);
            write_entry(
                &mut write,
                display_id,
                *version,
                group_id,
                flags,
                crafting_table,
                furnace,
                blast_furnace,
                smoker,
                campfire,
                None,
                Some((recipe, book_category)),
            )?;
            display_id += 1;
        }

        // replace flag
        write.write_bool(self.replace)?;
        Ok(())
    }
}
