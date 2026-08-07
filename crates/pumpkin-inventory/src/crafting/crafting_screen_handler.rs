//! Crafting screen handler implementation.
//!
//! This module provides screen handlers for crafting mechanics:
//! - [`CraftingScreenHandler`] - Trait for crafting screen handlers
//! - [`CraftingTableScreenHandler`] - The 3x3 crafting table UI
//! - [`ResultSlot`] - The special result slot that shows crafted items
//!
//! # Recipe Matching
//!
//! Crafting recipes are matched against the items in the crafting grid.
//! The system supports:
//! - Shaped recipes (specific patterns)
//! - Shapeless recipes (any arrangement)
//! - Transmute recipes (upgrading items)
//! - Special recipes (like decorated pots)

use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

use super::recipe_provider::{GenericRecipe, RecipeProvider};
use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ItemStackFuture, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerFuture,
    ScreenHandlerListener,
};
use crate::slot::{BoxFuture, NormalSlot, Slot};

use pumpkin_data::Enchantment;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DamageImpl, DataComponentImpl, EnchantmentsImpl, FireworkExplosionImpl, FireworkExplosionShape,
    FireworksImpl, MaxDamageImpl, PotDecorationsImpl, WrittenBookContentImpl,
};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipe_remainder::get_recipe_remainder_id;
use pumpkin_data::recipes::{CraftingRecipeTypes, RECIPES_CRAFTING};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::codec::recipe::{DynamicRecipe, OwnedCraftingRecipe};
use pumpkin_world::inventory::Inventory;
use tokio::sync::Mutex;

/// The result slot in a crafting screen.
pub struct ResultSlot {
    /// The crafting inventory (grid) that provides recipe input.
    pub inventory: Arc<dyn RecipeInputInventory>,
    /// Protocol ID for this slot (assigned by screen handler).
    pub id: AtomicU8,
    /// The cached result item stack.
    pub result: Arc<Mutex<ItemStack>>,
    /// Provider for dynamic recipes.
    pub recipe_provider: Option<Arc<dyn RecipeProvider>>,
    /// Cached `RecipeResult::remaining_items` from the last match, consulted by
    /// `on_take_item` in place of the static per-item remainder table.
    pub remaining_items: Arc<Mutex<Vec<(usize, ItemStack)>>>,
}

pub struct RecipeResult {
    pub item_id: String,
    pub count: u8,
    /// Components copied from a transmuted input, matching vanilla's
    /// `TransmuteRecipe::createWithOriginalComponents`.
    pub component_patch: Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>,
    /// Per-recipe `Recipe::getRemainingItems` override (26.2 decompile
    /// `Recipe.java`'s default, overridden by e.g. `BookCloningRecipe.java:127-143`):
    /// absolute inventory-slot index paired with the exact stack that survives the
    /// craft in that slot. Empty means "use the static per-item crafting-remainder
    /// table for every consumed slot", the prior/default behavior.
    pub remaining_items: Vec<(usize, ItemStack)>,
}

impl RecipeResult {
    fn new(item_id: String, count: u8) -> Self {
        Self {
            item_id,
            count,
            component_patch: Vec::new(),
            remaining_items: Vec::new(),
        }
    }

    fn transmuted(item_id: String, count: u8, input: &ItemStack) -> Self {
        Self {
            item_id,
            count,
            component_patch: input.patch.clone(),
            remaining_items: Vec::new(),
        }
    }
}

/// Checks if a recipe pattern is symmetrical horizontally.
fn is_symmetrical_horizontally(pattern: &[&str]) -> bool {
    let width = pattern.first().map_or(0, |s| s.len());
    for row in pattern {
        if row.len() != width {
            return false;
        }
        for j in 0..width / 2 {
            if row.chars().nth(j) != row.chars().nth(width - j - 1) {
                return false;
            }
        }
    }
    true
}

/// Checks if a crafting recipe matches the current inventory state.
#[expect(clippy::too_many_lines)]
async fn recipe_matches(
    recipe: GenericRecipe<'_>,
    input_height: usize,
    input_width: usize,
    top_x: usize,
    top_y: usize,
    count: usize,
    inventory: &dyn RecipeInputInventory,
) -> Option<RecipeResult> {
    match recipe {
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingShaped {
            key,
            pattern,
            result,
            ..
        }) => {
            if pattern.len() != input_height || pattern.first().unwrap().len() != input_width {
                return None;
            }

            if count
                != pattern
                    .iter()
                    .map(|l| l.chars().filter(|c| *c != ' ').count())
                    .sum::<usize>()
            {
                return None;
            }

            let x_offset = top_x;
            let y_offset = top_y;

            let mut matched = true;
            'outer: for (y, row_str) in pattern.iter().enumerate() {
                for (x, current_key) in row_str.chars().enumerate() {
                    let slot = inventory
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset))
                        .await;
                    if current_key == ' ' {
                        if !slot.lock().await.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }

                    let ingredient = key
                        .iter()
                        .find_map(|(k, v)| (*k == current_key).then_some(v))
                        .expect("Crafting recipe used invalid key");

                    let slot = slot.lock().await;

                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }

            if !matched && !is_symmetrical_horizontally(pattern) {
                matched = true;
                'outer: for y in 0..pattern.len() {
                    for x in 0..pattern[y].len() {
                        let current_key = pattern[y].chars().nth(x).unwrap();
                        let slot = inventory
                            .get_stack(
                                (y + y_offset) * inventory.get_height()
                                    + (x_offset + input_width - 1 - x),
                            )
                            .await;
                        if current_key == ' ' {
                            if !slot.lock().await.is_empty() {
                                matched = false;
                                break 'outer;
                            }
                            continue;
                        }
                        let ingredient = key
                            .iter()
                            .find_map(|(k, v)| (*k == current_key).then_some(v))
                            .expect("Crafting recipe used invalid key");
                        let slot = slot.lock().await;
                        if !ingredient.match_item(slot.item) {
                            matched = false;
                            break 'outer;
                        }
                    }
                }
            }

            matched.then(|| RecipeResult::new(result.id.to_string(), result.count))
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingShapeless {
            ingredients,
            result,
            ..
        }) => {
            if count != ingredients.len() {
                return None;
            }
            let mut ingredient_used = vec![false; ingredients.len()];
            'next_slot: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'next_slot;
                }
                for i in 0..ingredients.len() {
                    if !ingredient_used[i] && ingredients[i].match_item(slot.item) {
                        ingredient_used[i] = true;
                        continue 'next_slot;
                    }
                }
                return None;
            }
            Some(RecipeResult::new(result.id.to_string(), result.count))
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingTransmute {
            input,
            material,
            result,
            ..
        }) => {
            // Vanilla transmute recipes require one input stack and their configured
            // material count. Built-in recipes currently use the default of one
            // material stack.
            const MATERIAL_COUNT: usize = 1;
            if count != MATERIAL_COUNT + 1 {
                return None;
            }

            let mut input_count = 0;
            let mut material_count = 0;
            let mut input_stack = None;
            'item_stack: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'item_stack;
                }

                // This ordering matches TransmuteRecipe::matches: a stack that
                // satisfies both ingredients is the input, never the material.
                if input.match_item(slot.item) {
                    input_count += 1;
                    input_stack = Some(slot.clone());
                } else if material.match_item(slot.item) {
                    material_count += 1;
                } else {
                    return None;
                }
            }

            if input_count != 1 || material_count != MATERIAL_COUNT {
                return None;
            }

            Some(RecipeResult::transmuted(
                result.id.to_string(),
                result.count,
                input_stack
                    .as_ref()
                    .expect("matched transmute recipe has an input"),
            ))
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingDecoratedPot { .. }) => {
            if count != 4 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            let mut decorations = Vec::with_capacity(4);
            for position in (1..=7).step_by(2) {
                let slot = inventory.get_stack(position).await;
                let slot = slot.lock().await;
                if slot.is_empty()
                    || !slot
                        .item
                        .has_tag(&tag::Item::MINECRAFT_DECORATED_POT_INGREDIENTS)
                {
                    return None;
                }
                decorations.push(std::borrow::Cow::Borrowed(slot.item.registry_key));
            }
            let decorations = decorations
                .try_into()
                .expect("decorated pot recipe has exactly four ingredients");
            Some(RecipeResult {
                item_id: "minecraft:decorated_pot".to_string(),
                count: 1,
                component_patch: vec![(
                    DataComponent::PotDecorations,
                    Some(PotDecorationsImpl { decorations }.to_dyn()),
                )],
                remaining_items: Vec::new(),
            })
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shaped {
            pattern,
            key,
            result,
            ..
        }) => {
            if pattern.len() != input_height || pattern.first().unwrap().len() != input_width {
                return None;
            }
            if count
                != pattern
                    .iter()
                    .map(|l| l.chars().filter(|c| *c != ' ').count())
                    .sum::<usize>()
            {
                return None;
            }
            let x_offset = top_x;
            let y_offset = top_y;
            let mut matched = true;
            'outer: for (y, row_str) in pattern.iter().enumerate() {
                for (x, current_key) in row_str.chars().enumerate() {
                    let slot = inventory
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset))
                        .await;
                    if current_key == ' ' {
                        if !slot.lock().await.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }
                    let ingredient = key
                        .iter()
                        .find(|(k, _)| *k == current_key)
                        .map(|(_, v)| v)
                        .expect("Crafting recipe used invalid key");
                    let slot = slot.lock().await;
                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }
            matched.then(|| RecipeResult::new(result.item_id.clone(), result.count))
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shapeless {
            ingredients,
            result,
            ..
        }) => {
            if count != ingredients.len() {
                return None;
            }
            let mut ingredient_used = vec![false; ingredients.len()];
            'next_slot: for i in 0..inventory.size() {
                let slot = inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if slot.is_empty() {
                    continue 'next_slot;
                }
                for i in 0..ingredients.len() {
                    if !ingredient_used[i] && ingredients[i].match_item(slot.item) {
                        ingredient_used[i] = true;
                        continue 'next_slot;
                    }
                }
                return None;
            }
            Some(RecipeResult::new(result.item_id.clone(), result.count))
        }
        _ => None,
    }
}

/// Dye item -> vanilla firework explosion color. 26.2 decompile
/// `net/minecraft/world/item/DyeColor.java:29-45,102-104` (the `fireworkColor`
/// constructor argument, in enum declaration order white..black).
fn firework_dye_color(item: &Item) -> Option<i32> {
    Some(match item.registry_key {
        "white_dye" => 15_790_320,
        "orange_dye" => 15_435_844,
        "magenta_dye" => 12_801_229,
        "light_blue_dye" => 6_719_955,
        "yellow_dye" => 14_602_026,
        "lime_dye" => 4_312_372,
        "pink_dye" => 14_188_952,
        "gray_dye" => 4_408_131,
        "light_gray_dye" => 11_250_603,
        "cyan_dye" => 2_651_799,
        "purple_dye" => 8_073_150,
        "blue_dye" => 2_437_522,
        "brown_dye" => 5_320_730,
        "green_dye" => 3_887_386,
        "red_dye" => 11_743_532,
        "black_dye" => 1_973_019,
        _ => return None,
    })
}

/// Firework star shape ingredient, from `FireworkStarRecipe.shapes` as loaded
/// from `assets/recipes.json` (`minecraft:firework_star`).
fn firework_star_shape(item: &Item) -> Option<FireworkExplosionShape> {
    match item.registry_key {
        "feather" => Some(FireworkExplosionShape::Burst),
        "fire_charge" => Some(FireworkExplosionShape::LargeBall),
        "gold_nugget" => Some(FireworkExplosionShape::Star),
        "player_head"
        | "creeper_head"
        | "zombie_head"
        | "skeleton_skull"
        | "wither_skeleton_skull"
        | "dragon_head"
        | "piglin_head" => Some(FireworkExplosionShape::Creeper),
        _ => None,
    }
}

/// `FireworkRocketRecipe::matches`/`assemble`, 26.2 decompile
/// `FireworkRocketRecipe.java:52-96`. Only reached once the plain
/// `firework_rocket_simple` shapeless recipe (paper + one gunpowder) has
/// already failed to match; that recipe already covers the star-less,
/// single-gunpowder case with an identical result.
fn match_firework_rocket(items: &[ItemStack]) -> Option<RecipeResult> {
    if items.len() < 2 {
        return None;
    }
    let mut has_shell = false;
    let mut fuel_count = 0;
    let mut explosions = Vec::new();
    for stack in items {
        if stack.item == &Item::PAPER {
            if has_shell {
                return None;
            }
            has_shell = true;
        } else if stack.item == &Item::GUNPOWDER {
            fuel_count += 1;
            if fuel_count > 3 {
                return None;
            }
        } else if stack.item == &Item::FIREWORK_STAR {
            if let Some(explosion) = stack.get_data_component::<FireworkExplosionImpl>() {
                explosions.push(explosion.clone());
            }
        } else {
            return None;
        }
    }
    if !has_shell || fuel_count < 1 {
        return None;
    }
    Some(RecipeResult {
        item_id: "minecraft:firework_rocket".to_string(),
        count: 3,
        component_patch: vec![(
            DataComponent::Fireworks,
            Some(FireworksImpl::new(fuel_count, explosions).to_dyn()),
        )],
        remaining_items: Vec::new(),
    })
}

/// `FireworkStarRecipe::matches`/`assemble`, 26.2 decompile
/// `FireworkStarRecipe.java:87-160`.
fn match_firework_star(items: &[ItemStack]) -> Option<RecipeResult> {
    if items.len() < 2 {
        return None;
    }
    let mut has_fuel = false;
    let mut has_dye = false;
    let mut has_trail = false;
    let mut has_twinkle = false;
    let mut has_shape = false;
    let mut shape = FireworkExplosionShape::SmallBall;
    let mut colors = Vec::new();
    for stack in items {
        if stack.item == &Item::GLOWSTONE_DUST {
            if has_twinkle {
                return None;
            }
            has_twinkle = true;
        } else if stack.item == &Item::DIAMOND {
            if has_trail {
                return None;
            }
            has_trail = true;
        } else if stack.item == &Item::GUNPOWDER {
            if has_fuel {
                return None;
            }
            has_fuel = true;
        } else if let Some(color) = firework_dye_color(stack.item) {
            has_dye = true;
            colors.push(color);
        } else {
            let found_shape = firework_star_shape(stack.item)?;
            if has_shape {
                return None;
            }
            has_shape = true;
            shape = found_shape;
        }
    }
    if !has_fuel || !has_dye {
        return None;
    }
    Some(RecipeResult {
        item_id: "minecraft:firework_star".to_string(),
        count: 1,
        component_patch: vec![(
            DataComponent::FireworkExplosion,
            Some(
                FireworkExplosionImpl::new(shape, colors, Vec::new(), has_trail, has_twinkle)
                    .to_dyn(),
            ),
        )],
        remaining_items: Vec::new(),
    })
}

/// `FireworkStarFadeRecipe::matches`/`assemble`, 26.2 decompile
/// `FireworkStarFadeRecipe.java:44-88`. Copies the target star's component
/// patch and overwrites only `fade_colors`, matching
/// `FireworkExplosion::withFadeColors`.
fn match_firework_star_fade(items: &[ItemStack]) -> Option<RecipeResult> {
    if items.len() < 2 {
        return None;
    }
    let mut target = None;
    let mut colors = Vec::new();
    for stack in items {
        if let Some(color) = firework_dye_color(stack.item) {
            colors.push(color);
        } else if stack.item == &Item::FIREWORK_STAR {
            if target.is_some() {
                return None;
            }
            target = Some(stack);
        } else {
            return None;
        }
    }
    let target = target?;
    if colors.is_empty() {
        return None;
    }
    let explosion = target
        .get_data_component::<FireworkExplosionImpl>()
        .cloned()
        .unwrap_or_else(|| {
            FireworkExplosionImpl::new(
                FireworkExplosionShape::SmallBall,
                Vec::new(),
                Vec::new(),
                false,
                false,
            )
        });
    let mut patch = target.patch.clone();
    patch.retain(|(id, _)| *id != DataComponent::FireworkExplosion);
    patch.push((
        DataComponent::FireworkExplosion,
        Some(
            FireworkExplosionImpl {
                fade_colors: colors,
                ..explosion
            }
            .to_dyn(),
        ),
    ));
    Some(RecipeResult {
        item_id: "minecraft:firework_star".to_string(),
        count: 1,
        component_patch: patch,
        remaining_items: Vec::new(),
    })
}

/// Curse enchantments carried over from both inputs, keeping the higher
/// level of each. 26.2 decompile `RepairItemRecipe.java:71-79`: only
/// `EnchantmentTags.CURSE` survives a crafting-grid repair; everything else
/// is dropped (the anvil is the path that preserves ordinary enchantments).
fn merged_curse_enchantments(
    first: &ItemStack,
    second: &ItemStack,
) -> Vec<(&'static Enchantment, i32)> {
    let mut merged: Vec<(&'static Enchantment, i32)> = Vec::new();
    for stack in [first, second] {
        let Some(data) = stack.get_data_component::<EnchantmentsImpl>() else {
            continue;
        };
        for (enchantment, level) in data.enchantment.iter() {
            if !enchantment.has_tag(&tag::Enchantment::MINECRAFT_CURSE) {
                continue;
            }
            if let Some(existing) = merged.iter_mut().find(|(e, _)| *e == *enchantment) {
                existing.1 = existing.1.max(*level);
            } else {
                merged.push((*enchantment, *level));
            }
        }
    }
    merged
}

/// `RepairItemRecipe::matches`/`assemble`, 26.2 decompile
/// `RepairItemRecipe.java:24-79`.
fn match_repair_item(items: &[ItemStack]) -> Option<RecipeResult> {
    let [first, second] = items else {
        return None;
    };
    if first.item != second.item || first.item_count != 1 || second.item_count != 1 {
        return None;
    }
    let (Some(first_max), Some(second_max)) = (first.get_max_damage(), second.get_max_damage())
    else {
        return None;
    };

    let durability = first_max.max(second_max);
    let remaining = (first_max - first.get_damage())
        + (second_max - second.get_damage())
        + durability * 5 / 100;
    let damage = (durability - remaining).max(0);

    let mut patch = vec![(
        DataComponent::MaxDamage,
        Some(
            MaxDamageImpl {
                max_damage: durability,
            }
            .to_dyn(),
        ),
    )];
    if damage > 0 {
        patch.push((DataComponent::Damage, Some(DamageImpl { damage }.to_dyn())));
    }
    let enchantments = merged_curse_enchantments(first, second);
    if !enchantments.is_empty() {
        patch.push((
            DataComponent::Enchantments,
            Some(
                EnchantmentsImpl {
                    enchantment: std::borrow::Cow::Owned(enchantments),
                }
                .to_dyn(),
            ),
        ));
    }

    Some(RecipeResult {
        item_id: format!("minecraft:{}", first.item.registry_key),
        count: 1,
        component_patch: patch,
        remaining_items: Vec::new(),
    })
}

/// `BookCloningRecipe::matches`/`assemble`/`getRemainingItems`, 26.2 decompile
/// `BookCloningRecipe.java:58-143`. `book_cloning.json` does not override
/// `allowed_generations`, so `DEFAULT_BOOK_GENERATION_RANGES` (line 18,
/// `MinMaxBounds.Ints.between(0, 1)`) applies: only generation 0 (an original)
/// or 1 (a copy of an original) may be cloned; a copy of a copy cannot be.
/// The source book is never consumed as an ingredient -- it is returned via
/// `remaining_items` (line 128-143), one slot short of the material count so
/// its own occupied slot does not count toward the copies produced.
fn match_book_cloning(items: &[(usize, ItemStack)]) -> Option<RecipeResult> {
    if items.len() < 2 {
        return None;
    }
    let mut source: Option<(usize, &ItemStack)> = None;
    let mut material_count: u8 = 0;
    for (slot, stack) in items {
        if stack.item == &Item::WRITTEN_BOOK {
            if source.is_some() {
                return None;
            }
            source = Some((*slot, stack));
        } else if stack
            .item
            .has_tag(&tag::Item::MINECRAFT_BOOK_CLONING_TARGET)
        {
            material_count += 1;
        } else {
            return None;
        }
    }
    let (source_slot, source_stack) = source?;
    if material_count == 0 {
        return None;
    }
    let content = source_stack.get_data_component::<WrittenBookContentImpl>()?;
    if !(0..=1).contains(&content.generation) {
        return None;
    }

    let mut copied = content.clone();
    copied.generation += 1;
    let mut patch = source_stack.patch.clone();
    patch.retain(|(id, _)| *id != DataComponent::WrittenBookContent);
    patch.push((DataComponent::WrittenBookContent, Some(copied.to_dyn())));

    Some(RecipeResult {
        item_id: "minecraft:written_book".to_string(),
        count: material_count,
        component_patch: patch,
        remaining_items: vec![(source_slot, source_stack.copy_with_count(1))],
    })
}

/// Vanilla's `CustomRecipe` special recipes (`CraftingSpecial` in
/// `pumpkin-data`'s generated recipe types carries no data to dispatch on,
/// since each one is a Java class rather than JSON shape data), tried after
/// every data-driven recipe has failed to match.
fn match_special_recipe(items: &[(usize, ItemStack)]) -> Option<RecipeResult> {
    let unindexed: Vec<ItemStack> = items.iter().map(|(_, s)| s.clone()).collect();
    match_firework_rocket(&unindexed)
        .or_else(|| match_firework_star(&unindexed))
        .or_else(|| match_firework_star_fade(&unindexed))
        .or_else(|| match_repair_item(&unindexed))
        .or_else(|| match_book_cloning(items))
}

impl ResultSlot {
    pub fn new(
        inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        Self {
            inventory,
            id: AtomicU8::new(0),
            result: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            recipe_provider: provider,
            remaining_items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn match_recipe(&self) -> Option<RecipeResult> {
        let mut count: usize = 0;
        let inventory_width = self.inventory.get_width();
        let mut top_x = 9;
        let mut top_y = 9;
        let mut bottom_x = 0;
        let mut bottom_y = 0;
        for i in 0..self.inventory.size() {
            let x = i % inventory_width;
            let y = i / inventory_width;
            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                top_x = top_x.min(x);
                top_y = top_y.min(y);
                bottom_x = bottom_x.max(x);
                bottom_y = bottom_y.max(y);
                count += 1;
            }
        }
        if count == 0 {
            return None;
        }
        let input_width = bottom_x + 1 - top_x;
        let input_height = bottom_y + 1 - top_y;

        for recipe in RECIPES_CRAFTING {
            if let Some(result) = recipe_matches(
                GenericRecipe::Vanilla(recipe),
                input_height,
                input_width,
                top_x,
                top_y,
                count,
                &*self.inventory,
            )
            .await
            {
                return Some(result);
            }
        }

        if let Some(provider) = &self.recipe_provider {
            let dynamic = provider.get_dynamic_recipes().await;
            for recipe in &dynamic {
                if let DynamicRecipe::Crafting(crafting) = recipe
                    && let Some(result) = recipe_matches(
                        GenericRecipe::Dynamic(crafting),
                        input_height,
                        input_width,
                        top_x,
                        top_y,
                        count,
                        &*self.inventory,
                    )
                    .await
                {
                    return Some(result);
                }
            }
        }

        let mut items = Vec::with_capacity(count);
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i).await;
            let slot = slot.lock().await;
            if !slot.is_empty() {
                items.push((i, slot.clone()));
            }
        }
        match_special_recipe(&items)
    }

    async fn refill_output(&self) -> ItemStack {
        let (result, remaining_items) = if let Some(matched) = self.match_recipe().await {
            let key = matched
                .item_id
                .strip_prefix("minecraft:")
                .unwrap_or(&matched.item_id);
            let item = pumpkin_data::item::Item::from_registry_key(key)
                .unwrap_or(&pumpkin_data::item::Item::AIR);
            (
                ItemStack::new_with_component(matched.count, item, matched.component_patch),
                matched.remaining_items,
            )
        } else {
            (ItemStack::EMPTY.clone(), Vec::new())
        };
        *self.result.lock().await = result.clone();
        *self.remaining_items.lock().await = remaining_items;
        result
    }
}

/// Applies vanilla's crafting-remainder precedence to one input slot.
///
/// The ingredient has already been consumed when this is called. A remainder
/// replaces an empty slot, merges with an equal surviving stack, or is returned
/// for insertion into the player's inventory (and dropping if that is full).
fn apply_recipe_remainder(input: &mut ItemStack, remainder: ItemStack) -> Option<ItemStack> {
    if remainder.is_empty() {
        return None;
    }

    if input.is_empty() {
        *input = remainder;
        None
    } else if input.are_items_and_components_equal(&remainder) {
        input.increment(remainder.item_count);
        None
    } else {
        Some(remainder)
    }
}

impl Slot for ResultSlot {
    fn get_inventory(&self) -> Arc<dyn Inventory> {
        self.inventory.clone()
    }
    fn get_index(&self) -> usize {
        999
    }
    fn set_id(&self, id: usize) {
        self.id.store(id as u8, Ordering::Relaxed);
    }
    fn on_quick_move_crafted(
        &self,
        _stack: ItemStack,
        _stack_prev: ItemStack,
    ) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn on_take_item<'a>(
        &'a self,
        player: &'a dyn InventoryPlayer,
        stack: &'a ItemStack,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            player
                .increment_stat(
                    StatisticCategory::Crafted,
                    stack.item.id as i32,
                    stack.item_count as i32,
                )
                .await;
            let recipe_remaining_items = self.remaining_items.lock().await.clone();
            for i in 0..self.inventory.size() {
                let slot = self.inventory.get_stack(i).await;
                let mut stack = slot.lock().await;
                if !stack.is_empty() {
                    // A per-recipe remaining item (`Recipe::getRemainingItems`, e.g.
                    // `BookCloningRecipe.java:127-143`) overrides the static per-item
                    // crafting-remainder table for its slot. Guarded by item identity:
                    // the cache is filled at match time but consulted here after any
                    // number of awaits, so a slot whose contents changed in between
                    // (a second concurrent take) must not hand back a stale item.
                    let remainder = if let Some((_, item)) = recipe_remaining_items
                        .iter()
                        .find(|(slot, item)| *slot == i && item.item == stack.item)
                    {
                        Some(item.clone())
                    } else {
                        get_recipe_remainder_id(stack.item.id)
                            .and_then(pumpkin_data::item::Item::from_id)
                            .map(|item| ItemStack::new(1, item))
                    };
                    stack.item_count -= 1;
                    if let Some(remainder) = remainder
                        && let Some(mut remainder) = apply_recipe_remainder(&mut stack, remainder)
                    {
                        drop(stack);
                        player
                            .get_inventory()
                            .insert_stack_anywhere(&mut remainder)
                            .await;
                        if !remainder.is_empty() {
                            player.drop_item(remainder, false).await;
                        }
                    }
                }
            }
            self.mark_dirty().await;
        })
    }
    fn can_insert(&self, _stack: &ItemStack) -> BoxFuture<'_, bool> {
        Box::pin(async move { false })
    }
    fn get_stack(&self) -> BoxFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.result.clone() })
    }
    fn get_cloned_stack(&self) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move { self.result.lock().await.clone() })
    }
    fn has_stack(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move { !self.result.lock().await.is_empty() })
    }
    fn set_stack(&self, _stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.refill_output().await;
        })
    }
    fn mark_dirty(&self) -> BoxFuture<'_, ()> {
        Box::pin(async move {
            self.inventory.mark_dirty();
        })
    }
    fn get_max_item_count(&self) -> BoxFuture<'_, u8> {
        Box::pin(async move {
            let mut count = u8::MAX;
            for i in 0..self.inventory.size() {
                let slot = self.inventory.get_stack(i).await;
                let slot = slot.lock().await;
                if !slot.is_empty() {
                    count = count.min(slot.item_count);
                }
            }
            count
        })
    }
    fn take_stack(&self, _amount: u8) -> BoxFuture<'_, ItemStack> {
        Box::pin(async move {
            if self.has_stack().await {
                self.result.lock().await.clone()
            } else {
                ItemStack::EMPTY.clone()
            }
        })
    }
}

impl ScreenHandlerListener for ResultSlot {
    fn on_slot_update<'a>(
        &'a self,
        screen_handler: &'a ScreenHandlerBehaviour,
        slot: u8,
        _stack: ItemStack,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if (0..=(self.inventory.get_width() * self.inventory.get_height()))
                .contains(&(slot as usize))
            {
                let result = self.refill_output().await;
                let next_revision = screen_handler.next_revision();
                if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                    sync_handler
                        .update_slot(screen_handler, 0, &result, next_revision)
                        .await;
                }
            }
        })
    }
}

pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    fn add_recipe_slots<'a>(
        &'a mut self,
        crafing_inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let result_slot = Arc::new(ResultSlot::new(crafing_inventory.clone(), provider));
            self.add_slot(result_slot.clone());
            let width = crafing_inventory.get_width();
            let height = crafing_inventory.get_height();
            for i in 0..width {
                for j in 0..height {
                    let input_slot = NormalSlot::new(crafing_inventory.clone(), j + i * width);
                    self.add_slot(Arc::new(input_slot));
                }
            }
            self.add_listener(result_slot).await;
        })
    }
}

pub struct CraftingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl CraftingTableScreenHandler {
    pub async fn new(
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) -> Self {
        let crafting_inventory: Arc<dyn RecipeInputInventory> =
            Arc::new(CraftingInventory::new(3, 3));
        let mut crafting_table_handler = Self {
            behaviour: ScreenHandlerBehaviour::new(sync_id, Some(WindowType::Crafting)),
            crafting_inventory: crafting_inventory.clone(),
        };
        crafting_table_handler
            .add_recipe_slots(crafting_inventory, provider)
            .await;
        let player_inventory: Arc<dyn Inventory> = player_inventory.clone();
        crafting_table_handler.add_player_slots(&player_inventory);
        crafting_table_handler
    }
}

impl RecipeFinderScreenHandler for CraftingTableScreenHandler {}

impl ScreenHandler for CraftingTableScreenHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn get_behaviour(&self) -> &ScreenHandlerBehaviour {
        &self.behaviour
    }
    fn get_behaviour_mut(&mut self) -> &mut ScreenHandlerBehaviour {
        &mut self.behaviour
    }
    fn on_closed<'a>(&'a mut self, player: &'a dyn InventoryPlayer) -> ScreenHandlerFuture<'a, ()> {
        Box::pin(async move {
            self.default_on_closed(player).await;
            self.drop_inventory(player, self.crafting_inventory.clone())
                .await;
        })
    }
    fn quick_move<'a>(
        &'a mut self,
        player: &'a dyn InventoryPlayer,
        slot_index: i32,
    ) -> ItemStackFuture<'a> {
        Box::pin(async move {
            let slot = self.get_behaviour().slots[slot_index as usize].clone();
            if slot.has_stack().await {
                let slot_stack = slot.get_stack().await;
                let mut slot_stack = slot_stack.lock().await;
                let stack_prev = slot_stack.clone();
                if slot_index == 0 {
                    if !self.insert_item(&mut slot_stack, 10, 46, true).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (1..=9).contains(&slot_index) {
                    if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                        return ItemStack::EMPTY.clone();
                    }
                } else if (10..46).contains(&slot_index) {
                    if !self.insert_item(&mut slot_stack, 1, 10, false).await {
                        if slot_index < 37 {
                            if !self.insert_item(&mut slot_stack, 37, 46, false).await {
                                return ItemStack::EMPTY.clone();
                            }
                        } else if !self.insert_item(&mut slot_stack, 10, 37, false).await {
                            return ItemStack::EMPTY.clone();
                        }
                    }
                } else if !self.insert_item(&mut slot_stack, 10, 46, false).await {
                    return ItemStack::EMPTY.clone();
                }
                let stack = slot_stack.clone();
                drop(slot_stack);
                if stack.is_empty() {
                    slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone())
                        .await;
                } else {
                    slot.mark_dirty().await;
                }
                if stack.item_count == stack_prev.item_count {
                    return ItemStack::EMPTY.clone();
                }

                let mut taken_stack = stack_prev.clone();
                taken_stack.set_count(stack_prev.item_count - stack.item_count);
                slot.on_take_item(player, &taken_stack).await;

                if slot_index == 0 {
                    slot.on_quick_move_crafted(stack.clone(), stack_prev.clone())
                        .await;
                    if !stack.is_empty() {
                        player.drop_item(stack, false).await;
                    }
                }
                return stack_prev;
            }
            ItemStack::EMPTY.clone()
        })
    }
}

impl CraftingScreenHandler<CraftingInventory> for CraftingTableScreenHandler {}

#[cfg(test)]
mod recipe_remainder_tests {
    use super::apply_recipe_remainder;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    #[test]
    fn cake_milk_bucket_remainder_replaces_consumed_ingredient() {
        let mut consumed_milk_bucket = ItemStack::new(0, &Item::MILK_BUCKET);
        let bucket = ItemStack::new(1, &Item::BUCKET);

        assert!(apply_recipe_remainder(&mut consumed_milk_bucket, bucket).is_none());
        assert!(consumed_milk_bucket.are_equal(&ItemStack::new(1, &Item::BUCKET)));
    }

    #[test]
    fn remainder_merges_with_a_surviving_equal_input_stack() {
        let mut buckets = ItemStack::new(2, &Item::BUCKET);
        let bucket = ItemStack::new(1, &Item::BUCKET);

        assert!(apply_recipe_remainder(&mut buckets, bucket).is_none());
        assert_eq!(buckets.item_count, 3);
    }

    #[test]
    fn book_cloning_source_survives_via_remaining_item() {
        // `ResultSlot.onTake` (26.2 decompile `ResultSlot.java:100-115`) decrements the
        // consumed slot by 1 first, then applies the recipe's remaining item exactly
        // like a static crafting remainder. A written book has stack size 1, so the
        // source slot is empty by the time this runs.
        let mut consumed_source = ItemStack::new(0, &Item::WRITTEN_BOOK);
        let returned_original = ItemStack::new(1, &Item::WRITTEN_BOOK);

        assert!(apply_recipe_remainder(&mut consumed_source, returned_original).is_none());
        assert!(consumed_source.item == &Item::WRITTEN_BOOK);
        assert_eq!(consumed_source.item_count, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crafting::crafting_inventory::CraftingInventory;
    use pumpkin_data::item::Item;
    use pumpkin_data::recipes::{RecipeCategoryTypes, RecipeIngredientTypes, RecipeResultStruct};

    fn transmute_recipe() -> CraftingRecipeTypes {
        CraftingRecipeTypes::CraftingTransmute {
            category: RecipeCategoryTypes::Misc,
            group: None,
            input: RecipeIngredientTypes::Simple("minecraft:shulker_box"),
            material: RecipeIngredientTypes::Simple("minecraft:black_dye"),
            result: RecipeResultStruct {
                id: "minecraft:black_shulker_box",
                count: 1,
            },
        }
    }

    async fn matches_transmute(stacks: &[&'static Item]) -> bool {
        let inventory = CraftingInventory::new(3, 3);
        for (slot, item) in stacks.iter().enumerate() {
            inventory.set_stack(slot, ItemStack::new(1, item)).await;
        }

        let recipe = transmute_recipe();
        recipe_matches(
            GenericRecipe::Vanilla(&recipe),
            1,
            stacks.len(),
            0,
            0,
            stacks.len(),
            &inventory,
        )
        .await
        .is_some()
    }

    #[tokio::test]
    async fn transmute_matches_one_input_and_one_material() {
        assert!(matches_transmute(&[&Item::SHULKER_BOX, &Item::BLACK_DYE]).await);
    }

    #[tokio::test]
    async fn transmute_rejects_two_material_stacks() {
        assert!(
            !matches_transmute(&[&Item::SHULKER_BOX, &Item::BLACK_DYE, &Item::BLACK_DYE,]).await
        );
    }

    #[tokio::test]
    async fn transmute_rejects_duplicate_input_stacks() {
        assert!(!matches_transmute(&[&Item::SHULKER_BOX, &Item::SHULKER_BOX]).await);
    }

    #[tokio::test]
    async fn transmute_preserves_the_input_component_patch() {
        let inventory = CraftingInventory::new(3, 3);
        let mut input = ItemStack::new(1, &Item::SHULKER_BOX);
        input.set_damage(26);
        inventory.set_stack(0, input).await;
        inventory
            .set_stack(1, ItemStack::new(1, &Item::BLACK_DYE))
            .await;

        let recipe = transmute_recipe();
        let result = recipe_matches(GenericRecipe::Vanilla(&recipe), 1, 2, 0, 0, 2, &inventory)
            .await
            .expect("transmute recipe should match");
        let output = ItemStack::new_with_component(
            result.count,
            &Item::BLACK_SHULKER_BOX,
            result.component_patch,
        );

        assert_eq!(output.get_damage(), 26);
    }

    #[tokio::test]
    async fn decorated_pot_preserves_sherd_order_in_its_component() {
        let inventory = CraftingInventory::new(3, 3);
        for (slot, item) in [
            (1, &Item::ANGLER_POTTERY_SHERD),
            (3, &Item::ARCHER_POTTERY_SHERD),
            (5, &Item::ARMS_UP_POTTERY_SHERD),
            (7, &Item::BLADE_POTTERY_SHERD),
        ] {
            inventory.set_stack(slot, ItemStack::new(1, item)).await;
        }

        let recipe = CraftingRecipeTypes::CraftingDecoratedPot {
            category: RecipeCategoryTypes::Misc,
        };
        let result = recipe_matches(GenericRecipe::Vanilla(&recipe), 3, 3, 0, 0, 4, &inventory)
            .await
            .expect("decorated pot recipe should match");
        let output = ItemStack::new_with_component(
            result.count,
            &Item::DECORATED_POT,
            result.component_patch,
        );
        let decorations = output
            .get_data_component::<PotDecorationsImpl>()
            .expect("decorated pot output has decorations");

        assert_eq!(
            decorations.decorations,
            [
                std::borrow::Cow::Borrowed("angler_pottery_sherd"),
                std::borrow::Cow::Borrowed("archer_pottery_sherd"),
                std::borrow::Cow::Borrowed("arms_up_pottery_sherd"),
                std::borrow::Cow::Borrowed("blade_pottery_sherd"),
            ]
        );
    }
}

#[cfg(test)]
mod special_recipe_tests {
    use super::{
        DataComponent, DataComponentImpl, FireworkExplosionImpl, FireworkExplosionShape,
        FireworksImpl, match_firework_rocket, match_firework_star, match_repair_item,
    };
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    fn star_with_explosion(shape: FireworkExplosionShape, colors: Vec<i32>) -> ItemStack {
        ItemStack::new_with_component(
            1,
            &Item::FIREWORK_STAR,
            vec![(
                DataComponent::FireworkExplosion,
                Some(FireworkExplosionImpl::new(shape, colors, Vec::new(), false, false).to_dyn()),
            )],
        )
    }

    fn output_of(result: super::RecipeResult, item: &'static Item) -> ItemStack {
        ItemStack::new_with_component(result.count, item, result.component_patch)
    }

    #[test]
    fn firework_rocket_counts_gunpowder_and_collects_star_explosions() {
        let items = vec![
            ItemStack::new(1, &Item::PAPER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::GUNPOWDER),
            star_with_explosion(FireworkExplosionShape::Star, vec![11_743_532]),
        ];
        let result = match_firework_rocket(&items).expect("recipe should match");
        assert_eq!(result.item_id, "minecraft:firework_rocket");
        let output = output_of(result, &Item::FIREWORK_ROCKET);
        let fireworks = output
            .get_data_component::<FireworksImpl>()
            .expect("fireworks component present");
        assert_eq!(fireworks.flight_duration, 2);
        assert_eq!(fireworks.explosions.len(), 1);
        assert_eq!(fireworks.explosions[0].shape, FireworkExplosionShape::Star);
    }

    #[test]
    fn firework_rocket_rejects_more_than_three_gunpowder() {
        let items = vec![
            ItemStack::new(1, &Item::PAPER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::GUNPOWDER),
        ];
        assert!(match_firework_rocket(&items).is_none());
    }

    #[test]
    fn firework_star_reads_shape_and_dye_color() {
        let items = vec![
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::RED_DYE),
            ItemStack::new(1, &Item::GOLD_NUGGET),
        ];
        let result = match_firework_star(&items).expect("recipe should match");
        let output = output_of(result, &Item::FIREWORK_STAR);
        let explosion = output
            .get_data_component::<FireworkExplosionImpl>()
            .expect("explosion component present");
        assert_eq!(explosion.shape, FireworkExplosionShape::Star);
        assert_eq!(explosion.colors, vec![11_743_532]);
    }

    #[test]
    fn firework_star_rejects_two_shape_items() {
        let items = vec![
            ItemStack::new(1, &Item::GUNPOWDER),
            ItemStack::new(1, &Item::WHITE_DYE),
            ItemStack::new(1, &Item::FEATHER),
            ItemStack::new(1, &Item::FIRE_CHARGE),
        ];
        assert!(match_firework_star(&items).is_none());
    }

    fn damaged_iron_pickaxe(damage: i32, count: u8) -> ItemStack {
        let mut stack = ItemStack::new(count, &Item::IRON_PICKAXE);
        stack.set_damage(damage);
        stack
    }

    #[test]
    fn repair_item_combines_durability_with_five_percent_bonus() {
        // Vanilla RepairItemRecipe.java:69-73: durability = max(maxDamage),
        // remaining = (max-dmg1) + (max-dmg2) + durability*5/100.
        // Both inputs: max_damage 250, damage 200 -> remaining 50 each.
        // remaining = 50 + 50 + 250*5/100 (=12) = 112; damage = 250-112 = 138.
        let items = vec![damaged_iron_pickaxe(200, 1), damaged_iron_pickaxe(200, 1)];
        let result = match_repair_item(&items).expect("recipe should match");
        assert_eq!(result.item_id, "minecraft:iron_pickaxe");
        let output = output_of(result, &Item::IRON_PICKAXE);
        assert_eq!(output.get_max_damage(), Some(250));
        assert_eq!(output.get_damage(), 138);
    }

    #[test]
    fn repair_item_rejects_stacked_inputs() {
        let items = vec![damaged_iron_pickaxe(50, 2), damaged_iron_pickaxe(50, 1)];
        assert!(match_repair_item(&items).is_none());
    }

    #[test]
    fn repair_item_keeps_only_curse_enchantments_at_the_higher_level() {
        let mut first = damaged_iron_pickaxe(50, 1);
        first.add_enchantment(&pumpkin_data::Enchantment::VANISHING_CURSE, 1);
        let mut second = damaged_iron_pickaxe(50, 1);
        second.add_enchantment(&pumpkin_data::Enchantment::UNBREAKING, 3);

        let result = match_repair_item(&[first, second]).expect("recipe should match");
        let output = output_of(result, &Item::IRON_PICKAXE);
        assert_eq!(
            output.get_enchantment_level(&pumpkin_data::Enchantment::VANISHING_CURSE),
            1
        );
        assert_eq!(
            output.get_enchantment_level(&pumpkin_data::Enchantment::UNBREAKING),
            0
        );
    }

    fn written_book(generation: i32) -> ItemStack {
        ItemStack::new_with_component(
            1,
            &Item::WRITTEN_BOOK,
            vec![(
                DataComponent::WrittenBookContent,
                Some(
                    super::WrittenBookContentImpl {
                        pages: vec!["hi".to_string()],
                        generation,
                    }
                    .to_dyn(),
                ),
            )],
        )
    }

    #[test]
    fn book_cloning_bumps_generation_and_returns_the_original() {
        let items = vec![
            (0, written_book(0)),
            (1, ItemStack::new(1, &Item::WRITABLE_BOOK)),
            (4, ItemStack::new(1, &Item::WRITABLE_BOOK)),
        ];
        let result = super::match_book_cloning(&items).expect("recipe should match");
        assert_eq!(result.count, 2);
        assert_eq!(result.remaining_items.len(), 1);
        let (returned_slot, returned_stack) = &result.remaining_items[0];
        assert_eq!(*returned_slot, 0);
        assert!(returned_stack.item == &Item::WRITTEN_BOOK);
        assert_eq!(returned_stack.item_count, 1);
        assert_eq!(
            returned_stack
                .get_data_component::<super::WrittenBookContentImpl>()
                .expect("original keeps its content")
                .generation,
            0
        );

        let output = output_of(result, &Item::WRITTEN_BOOK);
        let content = output
            .get_data_component::<super::WrittenBookContentImpl>()
            .expect("written book content present");
        assert_eq!(content.generation, 1);
    }

    #[test]
    fn book_cloning_rejects_a_copy_of_a_copy() {
        let items = vec![
            (0, written_book(2)),
            (1, ItemStack::new(1, &Item::WRITABLE_BOOK)),
        ];
        assert!(super::match_book_cloning(&items).is_none());
    }

    #[test]
    fn book_cloning_rejects_two_written_books() {
        let items = vec![
            (0, written_book(0)),
            (1, written_book(0)),
            (2, ItemStack::new(1, &Item::WRITABLE_BOOK)),
        ];
        assert!(super::match_book_cloning(&items).is_none());
    }
}
