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

use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use std::any::Any;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

use super::recipe_provider::{GenericRecipe, RecipeProvider};
use super::recipes::{RecipeFinderScreenHandler, RecipeInputInventory};
use crate::crafting::crafting_inventory::CraftingInventory;
use crate::player::player_inventory::PlayerInventory;
use crate::screen_handler::{
    InventoryPlayer, ScreenHandler, ScreenHandlerBehaviour, ScreenHandlerListener,
};
use crate::slot::{NormalSlot, Slot};

use crate::inventory::Inventory;
use pumpkin_data::data_component_impl::{
    BannerPatternsImpl, BaseColorImpl, DamageImpl, DyeImpl, DyedColorImpl, EnchantmentsImpl,
    FireworkExplosionImpl, FireworkExplosionShape, FireworksImpl, MapIdImpl, MapPostProcessing,
    MapPostProcessingImpl, MaxDamageImpl, PotDecorationsImpl, PotionContentsImpl,
    ProvidesPotteryPatternImpl, WrittenBookContentImpl,
};
use pumpkin_data::dye_color::DyeColor;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::recipes::{
    CraftingRecipeTypes, RECIPES_CRAFTING, RecipeResultStruct, SpecialCraftingRecipe,
};
use pumpkin_data::screen::WindowType;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_protocol::codec::recipe::{DynamicRecipe, OwnedCraftingRecipe};

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
}

pub struct RecipeResult {
    pub item_id: String,
    pub count: u8,
    pub pot_decorations: Option<[ItemStack; 4]>,
    pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
    pub assembled_stack: Option<ItemStack>,
    pub remaining_items: Option<Vec<Option<ItemStack>>>,
}

impl RecipeResult {
    #[must_use]
    pub const fn simple(
        item_id: String,
        count: u8,
        components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
    ) -> Self {
        Self {
            item_id,
            count,
            pot_decorations: None,
            components,
            assembled_stack: None,
            remaining_items: None,
        }
    }

    #[must_use]
    pub fn from_stack(stack: ItemStack, remaining_items: Option<Vec<Option<ItemStack>>>) -> Self {
        Self {
            item_id: format!("minecraft:{}", stack.item.registry_key),
            count: stack.item_count,
            pot_decorations: None,
            components: &[],
            assembled_stack: Some(stack),
            remaining_items,
        }
    }

    #[must_use]
    pub fn to_item_stack(&self) -> ItemStack {
        if let Some(stack) = &self.assembled_stack {
            return stack.clone();
        }
        let key = self
            .item_id
            .strip_prefix("minecraft:")
            .unwrap_or(&self.item_id);
        let item = Item::from_registry_key(key).unwrap_or(&Item::AIR);
        let mut stack = ItemStack::new(self.count, item);
        for (component, value) in self.components {
            stack.patch.push((*component, Some(value.clone_dyn())));
        }
        if let Some([back, left, right, front]) = &self.pot_decorations {
            stack.set_data_component(PotDecorationsImpl {
                back: Some(back.clone()),
                left: Some(left.clone()),
                right: Some(right.clone()),
                front: Some(front.clone()),
            });
        }
        stack
    }

    #[must_use]
    pub fn remaining_item_for_slot(
        &self,
        slot: usize,
        ingredient: &ItemStack,
    ) -> Option<ItemStack> {
        ingredient.crafting_remainder().or_else(|| {
            self.remaining_items
                .as_ref()
                .and_then(|rems| rems.get(slot))
                .and_then(Clone::clone)
        })
    }
}

fn resolve_result_item(result: &RecipeResultStruct, fallback: &'static Item) -> &'static Item {
    if result.id == "minecraft:air" || result.id.is_empty() {
        return fallback;
    }
    let key = result.id.strip_prefix("minecraft:").unwrap_or(result.id);
    Item::from_registry_key(key).unwrap_or(fallback)
}

fn create_transmuted_stack(
    result: &RecipeResultStruct,
    input_stack: &ItemStack,
    extra_count: u8,
) -> ItemStack {
    let target_item = resolve_result_item(result, input_stack.item);
    let mut stack =
        input_stack.transmute_copy(target_item, result.count.saturating_add(extra_count));
    for (component, value) in result.components {
        let boxed = Some(value.clone_dyn());
        if let Some((_, c)) = stack.patch.iter_mut().find(|(id, _)| id == component) {
            *c = boxed;
        } else {
            stack.patch.push((*component, boxed));
        }
    }
    stack
}

/// Vanilla `DyedItemColor.applyDyes`.
fn apply_dyes_rgb(current: Option<&DyedColorImpl>, dyes: &[DyeColor]) -> DyedColorImpl {
    let mut r_sum: i32 = 0;
    let mut g_sum: i32 = 0;
    let mut b_sum: i32 = 0;
    let mut intensity_sum: i32 = 0;
    let mut count: i32 = 0;

    if let Some(cur) = current {
        let r = (cur.rgb >> 16) & 0xFF;
        let g = (cur.rgb >> 8) & 0xFF;
        let b = cur.rgb & 0xFF;
        intensity_sum += r.max(g).max(b);
        r_sum += r;
        g_sum += g;
        b_sum += b;
        count += 1;
    }

    for dye in dyes {
        let diffuse = dye.texture_diffuse_color() as i32;
        let r = (diffuse >> 16) & 0xFF;
        let g = (diffuse >> 8) & 0xFF;
        let b = diffuse & 0xFF;
        intensity_sum += r.max(g).max(b);
        r_sum += r;
        g_sum += g;
        b_sum += b;
        count += 1;
    }

    if count == 0 {
        return DyedColorImpl { rgb: 0 };
    }

    let r_avg = r_sum / count;
    let g_avg = g_sum / count;
    let b_avg = b_sum / count;
    let avg_intensity = intensity_sum as f32 / count as f32;
    let res_intensity = r_avg.max(g_avg).max(b_avg) as f32;
    let (r, g, b) = if res_intensity > 0.0 {
        (
            (r_avg as f32 * avg_intensity / res_intensity) as i32,
            (g_avg as f32 * avg_intensity / res_intensity) as i32,
            (b_avg as f32 * avg_intensity / res_intensity) as i32,
        )
    } else {
        (0, 0, 0)
    };
    DyedColorImpl {
        rgb: ((r & 0xFF) << 16) | ((g & 0xFF) << 8) | (b & 0xFF),
    }
}

#[expect(clippy::too_many_lines)]
fn match_special_recipe(
    special: &SpecialCraftingRecipe,
    count: usize,
    inventory: &dyn RecipeInputInventory,
) -> Option<RecipeResult> {
    match special {
        SpecialCraftingRecipe::Dye {
            target,
            dye,
            result,
        } => {
            if count < 2 {
                return None;
            }
            let mut target_stack: Option<ItemStack> = None;
            let mut dyes = Vec::new();
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if target.match_item(slot.item) {
                    if target_stack.is_some() {
                        return None;
                    }
                    target_stack = Some(slot);
                } else if dye.match_item(slot.item)
                    && let Some(d) = slot.get_data_component::<DyeImpl>()
                {
                    dyes.push(d.color);
                } else {
                    return None;
                }
            }
            let target_stack = target_stack?;
            if dyes.is_empty() {
                return None;
            }
            let new_color =
                apply_dyes_rgb(target_stack.get_data_component::<DyedColorImpl>(), &dyes);
            let mut out = create_transmuted_stack(result, &target_stack, 0);
            out.set_data_component(new_color);
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::Imbue {
            source,
            material,
            result,
        } => {
            if count != 9 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            for y in 0..3 {
                for x in 0..3 {
                    let slot = inventory.get_stack(y * 3 + x);
                    if slot.is_empty() {
                        return None;
                    }
                    let expected = if x == 1 && y == 1 { source } else { material };
                    if !expected.match_item(slot.item) {
                        return None;
                    }
                }
            }
            let center = inventory.get_stack(4);
            let out_item = resolve_result_item(result, &Item::TIPPED_ARROW);
            let mut out = ItemStack::new(result.count, out_item);
            if let Some(potion) = center.get_data_component::<PotionContentsImpl>() {
                out.set_data_component(potion.clone());
            }
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::ShieldDecoration {
            banner,
            target,
            result,
        } => {
            if count != 2 {
                return None;
            }
            let mut banner_stack: Option<(ItemStack, DyeColor)> = None;
            let mut target_stack: Option<ItemStack> = None;
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if banner.match_item(slot.item)
                    && let Some(color_name) = slot.item.registry_key.strip_suffix("_banner")
                    && let Some(color) = DyeColor::by_name(color_name)
                {
                    if banner_stack.is_some() {
                        return None;
                    }
                    banner_stack = Some((slot, color));
                } else if target.match_item(slot.item) {
                    if target_stack.is_some() {
                        return None;
                    }
                    if slot
                        .get_data_component::<BannerPatternsImpl>()
                        .is_some_and(|p| !p.layers.is_empty())
                    {
                        return None;
                    }
                    target_stack = Some(slot);
                } else {
                    return None;
                }
            }
            let (banner_stack, base_color) = banner_stack?;
            let target_stack = target_stack?;
            let mut out = create_transmuted_stack(result, &target_stack, 0);
            if let Some(patterns) = banner_stack.get_data_component::<BannerPatternsImpl>() {
                out.set_data_component(patterns.clone());
            }
            out.set_data_component(BaseColorImpl {
                color: base_color.name().to_string(),
            });
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::BannerDuplicate { banner, result } => {
            if count != 2 {
                return None;
            }
            let mut color: Option<DyeColor> = None;
            let mut source_slot_and_stack: Option<(usize, ItemStack)> = None;
            let mut has_blank = false;
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                let c_name = slot.item.registry_key.strip_suffix("_banner")?;
                let slot_color = DyeColor::by_name(c_name)?;
                if !banner.match_item(slot.item) {
                    return None;
                }
                if let Some(existing) = color {
                    if existing != slot_color {
                        return None;
                    }
                } else {
                    color = Some(slot_color);
                }
                let pattern_count = slot
                    .get_data_component::<BannerPatternsImpl>()
                    .map_or(0, |p| p.layers.len());
                if pattern_count > 6 {
                    return None;
                }
                if pattern_count > 0 {
                    if source_slot_and_stack.is_some() {
                        return None;
                    }
                    source_slot_and_stack = Some((i, slot));
                } else {
                    if has_blank {
                        return None;
                    }
                    has_blank = true;
                }
            }
            let (source_idx, source_stack) = source_slot_and_stack?;
            if !has_blank {
                return None;
            }
            let out = create_transmuted_stack(result, &source_stack, 0);
            let mut remainders = vec![None; inventory.size()];
            remainders[source_idx] = Some(source_stack.copy_with_count(1));
            Some(RecipeResult::from_stack(out, Some(remainders)))
        }
        SpecialCraftingRecipe::BookCloning {
            source,
            material,
            allowed_generations,
            result,
        } => {
            if count < 2 {
                return None;
            }
            let mut source_slot_and_stack: Option<(usize, ItemStack, WrittenBookContentImpl)> =
                None;
            let mut material_count: u8 = 0;
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if source.match_item(slot.item) {
                    let content = slot.get_data_component::<WrittenBookContentImpl>()?.clone();
                    if content.generation < allowed_generations.0
                        || content.generation > allowed_generations.1
                        || source_slot_and_stack.is_some()
                    {
                        return None;
                    }
                    source_slot_and_stack = Some((i, slot, content));
                } else if material.match_item(slot.item) {
                    material_count = material_count.saturating_add(1);
                } else {
                    return None;
                }
            }
            let (source_idx, source_stack, mut content) = source_slot_and_stack?;
            if material_count == 0 {
                return None;
            }
            content.generation += 1;
            let mut out =
                create_transmuted_stack(result, &source_stack, material_count.saturating_sub(1));
            out.set_data_component(content);
            let mut remainders = vec![None; inventory.size()];
            remainders[source_idx] = Some(source_stack.copy_with_count(1));
            Some(RecipeResult::from_stack(out, Some(remainders)))
        }
        SpecialCraftingRecipe::FireworkStar {
            shapes,
            trail,
            twinkle,
            fuel,
            dye,
            result,
        } => {
            if count < 2 {
                return None;
            }
            let mut has_fuel = false;
            let mut has_trail = false;
            let mut has_twinkle = false;
            let mut chosen_shape: Option<FireworkExplosionShape> = None;
            let mut colors = Vec::new();
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if twinkle.match_item(slot.item) {
                    if has_twinkle {
                        return None;
                    }
                    has_twinkle = true;
                } else if trail.match_item(slot.item) {
                    if has_trail {
                        return None;
                    }
                    has_trail = true;
                } else if fuel.match_item(slot.item) {
                    if has_fuel {
                        return None;
                    }
                    has_fuel = true;
                } else if dye.match_item(slot.item)
                    && let Some(d) = slot.get_data_component::<DyeImpl>()
                {
                    colors.push(d.color.firework_color() as i32);
                } else if let Some((shape_name, _)) =
                    shapes.iter().find(|(_, ing)| ing.match_item(slot.item))
                {
                    if chosen_shape.is_some() {
                        return None;
                    }
                    chosen_shape = Some(FireworkExplosionShape::from_name(shape_name)?);
                } else {
                    return None;
                }
            }
            if !has_fuel || colors.is_empty() {
                return None;
            }
            let out_item = resolve_result_item(result, &Item::FIREWORK_STAR);
            let mut out = ItemStack::new(result.count, out_item);
            out.set_data_component(FireworkExplosionImpl {
                shape: chosen_shape.unwrap_or(FireworkExplosionShape::SmallBall),
                colors,
                fade_colors: Vec::new(),
                has_trail,
                has_twinkle,
            });
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::FireworkStarFade {
            target,
            dye,
            result,
        } => {
            if count < 2 {
                return None;
            }
            let mut target_stack: Option<ItemStack> = None;
            let mut fade_colors = Vec::new();
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if dye.match_item(slot.item)
                    && let Some(d) = slot.get_data_component::<DyeImpl>()
                {
                    fade_colors.push(d.color.firework_color() as i32);
                } else if target.match_item(slot.item) {
                    if target_stack.is_some() {
                        return None;
                    }
                    target_stack = Some(slot);
                } else {
                    return None;
                }
            }
            let target_stack = target_stack?;
            if fade_colors.is_empty() {
                return None;
            }
            let mut out = create_transmuted_stack(result, &target_stack, 0);
            let mut explosion = out
                .get_data_component::<FireworkExplosionImpl>()
                .cloned()
                .unwrap_or_else(|| FireworkExplosionImpl {
                    shape: FireworkExplosionShape::SmallBall,
                    colors: Vec::new(),
                    fade_colors: Vec::new(),
                    has_trail: false,
                    has_twinkle: false,
                });
            explosion.fade_colors = fade_colors;
            out.set_data_component(explosion);
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::FireworkRocket {
            shell,
            fuel,
            star,
            result,
        } => {
            if count < 2 {
                return None;
            }
            let mut has_shell = false;
            let mut fuel_count: i32 = 0;
            let mut explosions = Vec::new();
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if shell.match_item(slot.item) {
                    if has_shell {
                        return None;
                    }
                    has_shell = true;
                } else if fuel.match_item(slot.item) {
                    fuel_count += 1;
                    if fuel_count > 3 {
                        return None;
                    }
                } else if star.match_item(slot.item) {
                    if let Some(exp) = slot.get_data_component::<FireworkExplosionImpl>() {
                        explosions.push(exp.clone());
                    }
                } else {
                    return None;
                }
            }
            if !has_shell || fuel_count < 1 {
                return None;
            }
            let out_item = resolve_result_item(result, &Item::FIREWORK_ROCKET);
            let mut out = ItemStack::new(result.count, out_item);
            out.set_data_component(FireworksImpl {
                flight_duration: fuel_count,
                explosions,
            });
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::MapExtending {
            map,
            material,
            result,
        } => {
            if count != 9 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            for y in 0..3 {
                for x in 0..3 {
                    let slot = inventory.get_stack(y * 3 + x);
                    if slot.is_empty() {
                        return None;
                    }
                    let expected = if x == 1 && y == 1 { map } else { material };
                    if !expected.match_item(slot.item) {
                        return None;
                    }
                }
            }
            let center = inventory.get_stack(4);
            center.get_data_component::<MapIdImpl>()?;
            let mut out = create_transmuted_stack(result, &center, 0);
            out.set_data_component(MapPostProcessingImpl {
                processing: Some(MapPostProcessing::Scale),
            });
            Some(RecipeResult::from_stack(out, None))
        }
        SpecialCraftingRecipe::RepairItem => {
            if count != 2 {
                return None;
            }
            let mut first: Option<ItemStack> = None;
            let mut second: Option<ItemStack> = None;
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if first.is_none() {
                    first = Some(slot);
                } else {
                    second = Some(slot);
                }
            }
            let (first, second) = (first?, second?);
            if first.item.id != second.item.id
                || first.item_count != 1
                || second.item_count != 1
                || !first.has_data_component(DataComponent::MaxDamage)
                || !second.has_data_component(DataComponent::MaxDamage)
                || !first.has_data_component(DataComponent::Damage)
                || !second.has_data_component(DataComponent::Damage)
            {
                return None;
            }
            let max1 = first.get_max_damage()?;
            let max2 = second.get_max_damage()?;
            let durability = max1.max(max2);
            let rem1 = max1 - first.get_damage();
            let rem2 = max2 - second.get_damage();
            let remaining = rem1 + rem2 + durability * 5 / 100;
            let mut out = ItemStack::new(1, first.item);
            out.set_data_component(MaxDamageImpl {
                max_damage: durability,
            });
            out.set_data_component(DamageImpl {
                damage: (durability - remaining).max(0),
            });

            let mut curses: Vec<(&'static pumpkin_data::Enchantment, i32)> = Vec::new();
            for src in [&first, &second] {
                if let Some(ench) = src.get_data_component::<EnchantmentsImpl>() {
                    for (e, lvl) in ench.enchantment.iter() {
                        if e.has_tag(&tag::Enchantment::MINECRAFT_CURSE) {
                            if let Some(existing) = curses.iter_mut().find(|(cur, _)| *cur == *e) {
                                existing.1 = existing.1.max(*lvl);
                            } else {
                                curses.push((*e, *lvl));
                            }
                        }
                    }
                }
            }
            if !curses.is_empty() {
                out.set_data_component(EnchantmentsImpl {
                    enchantment: std::borrow::Cow::Owned(curses),
                });
            }
            Some(RecipeResult::from_stack(out, None))
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
fn recipe_matches(
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
            #[allow(clippy::redundant_closure_for_method_calls)]
            if pattern.len() != input_height
                || pattern.first().map_or(0, |f| f.len()) != input_width
            {
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
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset));
                    if current_key == ' ' {
                        if !slot.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }

                    let Some(ingredient) = key
                        .iter()
                        .find_map(|(k, v)| (*k == current_key).then_some(v))
                    else {
                        matched = false;
                        break 'outer;
                    };

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
                        let Some(current_key) = pattern[y].chars().nth(x) else {
                            matched = false;
                            break 'outer;
                        };
                        let slot = inventory.get_stack(
                            (y + y_offset) * inventory.get_height()
                                + (x_offset + input_width - 1 - x),
                        );
                        if current_key == ' ' {
                            if !slot.is_empty() {
                                matched = false;
                                break 'outer;
                            }
                            continue;
                        }
                        let Some(ingredient) = key
                            .iter()
                            .find_map(|(k, v)| (*k == current_key).then_some(v))
                        else {
                            matched = false;
                            break 'outer;
                        };
                        if !ingredient.match_item(slot.item) {
                            matched = false;
                            break 'outer;
                        }
                    }
                }
            }

            matched.then_some(RecipeResult::simple(
                result.id.to_string(),
                result.count,
                result.components,
            ))
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
                let slot = inventory.get_stack(i);
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
            Some(RecipeResult::simple(
                result.id.to_string(),
                result.count,
                result.components,
            ))
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingTransmute {
            input,
            material,
            material_count,
            add_material_count_to_result,
            result,
            ..
        }) => {
            let (min_mat, max_mat) = *material_count;
            if count < usize::from(min_mat) + 1 || count > usize::from(max_mat) + 1 {
                return None;
            }
            let mut found_input: Option<ItemStack> = None;
            let mut mat_seen: u8 = 0;
            for i in 0..inventory.size() {
                let slot = inventory.get_stack(i);
                if slot.is_empty() {
                    continue;
                }
                if input.match_item(slot.item) {
                    if found_input.is_some() {
                        return None;
                    }
                    found_input = Some(slot);
                } else if material.match_item(slot.item) {
                    mat_seen = mat_seen.saturating_add(1);
                    if mat_seen > max_mat {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            let found_input = found_input?;
            if mat_seen < min_mat {
                return None;
            }
            let extra = if *add_material_count_to_result {
                mat_seen
            } else {
                0
            };
            let out = create_transmuted_stack(result, &found_input, extra);
            if out.is_empty()
                || (out.item_count == 1 && found_input.are_items_and_components_equal(&out))
            {
                return None;
            }
            Some(RecipeResult::from_stack(out, None))
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingDecoratedPot { .. }) => {
            if count != 4 || inventory.get_width() != 3 || inventory.get_height() != 3 {
                return None;
            }
            let mut sherds = std::array::from_fn(|_| ItemStack::EMPTY.clone());
            for (idx, position) in (1..=7).step_by(2).enumerate() {
                let mut slot = inventory.get_stack(position);
                if slot.is_empty()
                    || (!slot
                        .item
                        .has_tag(&tag::Item::MINECRAFT_DECORATED_POT_INGREDIENTS)
                        && slot
                            .get_data_component::<ProvidesPotteryPatternImpl>()
                            .is_none())
                {
                    return None;
                }
                slot.item_count = 1;
                sherds[idx] = slot;
            }
            Some(RecipeResult {
                item_id: "minecraft:decorated_pot".to_string(),
                count: 1,
                pot_decorations: Some(sherds),
                components: &[],
                assembled_stack: None,
                remaining_items: None,
            })
        }
        GenericRecipe::Vanilla(CraftingRecipeTypes::CraftingSpecial(special)) => {
            match_special_recipe(special, count, inventory)
        }
        GenericRecipe::Dynamic(OwnedCraftingRecipe::Shaped {
            pattern,
            key,
            result,
            ..
        }) => {
            #[allow(clippy::redundant_closure_for_method_calls)]
            if pattern.len() != input_height
                || pattern.first().map_or(0, |f| f.len()) != input_width
            {
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
                        .get_stack((y + y_offset) * inventory.get_width() + (x + x_offset));
                    if current_key == ' ' {
                        if !slot.is_empty() {
                            matched = false;
                            break 'outer;
                        }
                        continue;
                    }
                    let Some(ingredient) =
                        key.iter().find(|(k, _)| *k == current_key).map(|(_, v)| v)
                    else {
                        matched = false;
                        break 'outer;
                    };
                    if !ingredient.match_item(slot.item) {
                        matched = false;
                        break 'outer;
                    }
                }
            }
            matched.then_some(RecipeResult::simple(
                result.item_id.clone(),
                result.count,
                &[],
            ))
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
                let slot = inventory.get_stack(i);
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
            Some(RecipeResult::simple(
                result.item_id.clone(),
                result.count,
                &[],
            ))
        }
    }
}

#[must_use]
pub fn match_crafting_recipe(
    inventory: &dyn RecipeInputInventory,
    provider: Option<&dyn RecipeProvider>,
) -> Option<RecipeResult> {
    let mut count: usize = 0;
    let inventory_width = inventory.get_width();
    let mut top_x = 9;
    let mut top_y = 9;
    let mut bottom_x = 0;
    let mut bottom_y = 0;
    for i in 0..inventory.size() {
        let x = i % inventory_width;
        let y = i / inventory_width;
        let slot = inventory.get_stack(i);
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
            inventory,
        ) {
            return Some(result);
        }
    }

    if let Some(provider) = provider {
        let dynamic = provider.get_dynamic_recipes();
        for recipe in &dynamic {
            if let DynamicRecipe::Crafting(crafting) = recipe
                && let Some(result) = recipe_matches(
                    GenericRecipe::Dynamic(crafting),
                    input_height,
                    input_width,
                    top_x,
                    top_y,
                    count,
                    inventory,
                )
            {
                return Some(result);
            }
        }
    }

    None
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
        }
    }

    fn match_recipe(&self) -> Option<RecipeResult> {
        match_crafting_recipe(&*self.inventory, self.recipe_provider.as_deref())
    }

    fn refill_output(&self) -> ItemStack {
        let result = self.match_recipe().map_or_else(
            || ItemStack::EMPTY.clone(),
            |matched| matched.to_item_stack(),
        );
        *self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = result.clone();
        result
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
    fn on_quick_move_crafted(&self, _stack: ItemStack, _stack_prev: ItemStack) {
        self.refill_output();
    }
    fn on_take_item(&self, player: &dyn InventoryPlayer, stack: &ItemStack) {
        player.increment_stat(
            StatisticCategory::Crafted,
            stack.item.id as i32,
            stack.item_count as i32,
        );
        player.on_recipe_crafted(stack);
        let matched_recipe = self.match_recipe();
        for i in 0..self.inventory.size() {
            let ingredient = self.inventory.get_stack(i);
            if ingredient.is_empty() {
                continue;
            }
            let remainder = matched_recipe
                .as_ref()
                .and_then(|m| m.remaining_item_for_slot(i, &ingredient))
                .or_else(|| ingredient.crafting_remainder());
            self.inventory.remove_stack_specific(i, 1);
            let Some(remainder) = remainder else {
                continue;
            };

            let remaining = self.inventory.get_stack(i);
            if remaining.is_empty() {
                self.inventory.set_stack(i, remainder);
            } else if remaining.are_items_and_components_equal(&remainder) {
                let mut merged = remaining;
                merged.increment(remainder.item_count);
                self.inventory.set_stack(i, merged);
            } else {
                let mut leftover = remainder;
                if !player.get_inventory().insert_stack_anywhere(&mut leftover) {
                    player.drop_item(leftover, false);
                }
            }
        }
        self.mark_dirty();
    }
    fn can_insert(&self, _stack: &ItemStack) -> bool {
        false
    }
    fn get_stack(&self) -> ItemStack {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn get_cloned_stack(&self) -> ItemStack {
        self.result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn has_stack(&self) -> bool {
        !self
            .result
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }
    fn set_stack(&self, _stack: ItemStack) {
        self.refill_output();
    }
    fn set_stack_prev(&self, _stack: ItemStack, _previous_stack: ItemStack) {
        self.refill_output();
    }
    fn mark_dirty(&self) {
        self.inventory.mark_dirty();
    }
    fn get_max_item_count(&self) -> u8 {
        let mut count = u8::MAX;
        for i in 0..self.inventory.size() {
            let slot = self.inventory.get_stack(i);
            if !slot.is_empty() {
                count = count.min(slot.item_count);
            }
        }
        count
    }
    fn take_stack(&self, _amount: u8) -> ItemStack {
        if self.has_stack() {
            self.result
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        } else {
            ItemStack::EMPTY.clone()
        }
    }
}

impl ScreenHandlerListener for ResultSlot {
    fn on_slot_update(&self, screen_handler: &ScreenHandlerBehaviour, slot: u8, _stack: ItemStack) {
        if (0..=(self.inventory.get_width() * self.inventory.get_height()))
            .contains(&(slot as usize))
        {
            let result = self.refill_output();
            let next_revision = screen_handler.next_revision();
            if let Some(sync_handler) = screen_handler.sync_handler.as_ref() {
                sync_handler.update_slot(screen_handler, 0, &result, next_revision);
            }
        }
    }
}

pub trait CraftingScreenHandler<I: RecipeInputInventory>:
    RecipeFinderScreenHandler + ScreenHandler
{
    fn add_recipe_slots(
        &mut self,
        crafing_inventory: Arc<dyn RecipeInputInventory>,
        provider: Option<Arc<dyn RecipeProvider>>,
    ) {
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
        self.add_listener(result_slot);
    }
}

pub struct CraftingTableScreenHandler {
    behaviour: ScreenHandlerBehaviour,
    crafting_inventory: Arc<dyn RecipeInputInventory>,
}

impl CraftingTableScreenHandler {
    pub fn new(
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
        crafting_table_handler.add_recipe_slots(crafting_inventory, provider);
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
    fn on_closed(&mut self, player: &dyn InventoryPlayer) {
        self.default_on_closed(player);
        self.drop_inventory(player, self.crafting_inventory.clone());
    }
    fn quick_move(&mut self, player: &dyn InventoryPlayer, slot_index: i32) -> ItemStack {
        let slot = self.get_behaviour().slots[slot_index as usize].clone();
        if slot.has_stack() {
            let mut slot_stack = slot.get_stack();
            let stack_prev = slot_stack.clone();
            if slot_index == 0 {
                if !self.insert_item(&mut slot_stack, 10, 46, true) {
                    return ItemStack::EMPTY.clone();
                }
            } else if (1..=9).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 10, 46, false) {
                    return ItemStack::EMPTY.clone();
                }
            } else if (10..46).contains(&slot_index) {
                if !self.insert_item(&mut slot_stack, 1, 10, false) {
                    if slot_index < 37 {
                        if !self.insert_item(&mut slot_stack, 37, 46, false) {
                            return ItemStack::EMPTY.clone();
                        }
                    } else if !self.insert_item(&mut slot_stack, 10, 37, false) {
                        return ItemStack::EMPTY.clone();
                    }
                }
            } else if !self.insert_item(&mut slot_stack, 10, 46, false) {
                return ItemStack::EMPTY.clone();
            }
            let stack = slot_stack.clone();
            drop(slot_stack);
            if stack.is_empty() {
                slot.set_stack_prev(ItemStack::EMPTY.clone(), stack_prev.clone());
            } else {
                slot.mark_dirty();
            }
            if stack.item_count == stack_prev.item_count {
                return ItemStack::EMPTY.clone();
            }

            let mut taken_stack = stack_prev.clone();
            taken_stack.set_count(stack_prev.item_count - stack.item_count);
            slot.on_take_item(player, &taken_stack);

            if slot_index == 0 {
                slot.on_quick_move_crafted(stack.clone(), stack_prev.clone());
                if !stack.is_empty() {
                    player.drop_item(stack, false);
                }
            }
            return stack_prev;
        }
        ItemStack::EMPTY.clone()
    }
}

impl CraftingScreenHandler<CraftingInventory> for CraftingTableScreenHandler {}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::data_component_impl::EquipmentSlot;
    use pumpkin_data::item::Item;
    use pumpkin_data::sound::Sound;
    use pumpkin_protocol::java::client::play::{
        CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem,
        CSetPlayerInventory, CSetSelectedSlot,
    };
    use pumpkin_protocol::java::server::play::SlotActionType;

    use crate::entity_equipment::EntityEquipment;
    use crate::screen_handler::InventoryPlayer;

    struct TestPlayer {
        inventory: Arc<PlayerInventory>,
        drops: Mutex<Vec<ItemStack>>,
    }

    impl TestPlayer {
        fn new(inventory: Arc<PlayerInventory>) -> Self {
            Self {
                inventory,
                drops: Mutex::new(Vec::new()),
            }
        }
    }

    impl InventoryPlayer for TestPlayer {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn drop_item(&self, item: ItemStack, _retain_ownership: bool) {
            self.drops
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(item);
        }

        fn get_inventory(&self) -> Arc<PlayerInventory> {
            self.inventory.clone()
        }

        fn has_infinite_materials(&self) -> bool {
            false
        }

        fn is_creative(&self) -> bool {
            false
        }

        fn experience_level(&self) -> i32 {
            0
        }

        fn add_experience_levels(&self, _levels: i32) {}

        fn enchantment_seed(&self) -> i32 {
            0
        }

        fn set_enchantment_seed(&self, _seed: i32) {}

        fn enqueue_inventory_packet(
            &self,
            _packet: &CSetContainerContent,
            _window_type: Option<WindowType>,
        ) {
        }

        fn enqueue_slot_packet(
            &self,
            _packet: &CSetContainerSlot,
            _window_type: Option<WindowType>,
            _total_slots: usize,
        ) {
        }

        fn enqueue_cursor_packet(&self, _packet: &CSetCursorItem) {}

        fn enqueue_property_packet(
            &self,
            _packet: &CSetContainerProperty,
            _window_type: Option<WindowType>,
        ) {
        }

        fn enqueue_slot_set_packet(&self, _packet: &CSetPlayerInventory) {}

        fn enqueue_set_held_item_packet(&self, _packet: &CSetSelectedSlot) {}

        fn enqueue_equipment_change(&self, _slot: &EquipmentSlot, _stack: &ItemStack) {}

        fn award_experience(&self, _amount: i32) {}

        fn increment_stat(&self, _category: StatisticCategory, _stat_id: i32, _amount: i32) {}

        fn play_block_sound(&self, _sound: Sound, _pitch: f32) {}
    }

    fn test_handler() -> (CraftingTableScreenHandler, Arc<PlayerInventory>, TestPlayer) {
        let player_inventory = Arc::new(PlayerInventory::new(
            Arc::new(Mutex::new(EntityEquipment::new())),
            Arc::new(rustc_hash::FxHashMap::default()),
        ));
        let player = TestPlayer::new(player_inventory.clone());
        let handler = CraftingTableScreenHandler::new(1, &player_inventory, None);
        (handler, player_inventory, player)
    }

    fn set_slot(handler: &CraftingTableScreenHandler, slot: usize, stack: ItemStack) {
        handler.get_behaviour().slots[slot].set_stack(stack);
    }

    fn grid_stack(handler: &CraftingTableScreenHandler, slot: usize) -> ItemStack {
        handler.get_behaviour().slots[slot].get_stack()
    }

    fn refresh_result(handler: &CraftingTableScreenHandler) {
        handler.get_behaviour().slots[0].set_stack(ItemStack::EMPTY.clone());
    }

    fn place_cake(handler: &CraftingTableScreenHandler) {
        for slot in 1..=3 {
            set_slot(handler, slot, ItemStack::new(1, &Item::MILK_BUCKET));
        }
        set_slot(handler, 4, ItemStack::new(1, &Item::SUGAR));
        set_slot(handler, 5, ItemStack::new(1, &Item::EGG));
        set_slot(handler, 6, ItemStack::new(1, &Item::SUGAR));
        for slot in 7..=9 {
            set_slot(handler, slot, ItemStack::new(1, &Item::WHEAT));
        }
    }

    fn mace_result(grid_size: u8, top_slot: usize) -> Option<String> {
        let inventory = CraftingInventory::new(grid_size, grid_size);
        inventory.set_stack(top_slot, ItemStack::new(1, &Item::HEAVY_CORE));
        inventory.set_stack(
            top_slot + usize::from(grid_size),
            ItemStack::new(1, &Item::BREEZE_ROD),
        );
        match_crafting_recipe(&inventory, None).map(|result| result.item_id)
    }

    #[test]
    fn padded_shaped_recipe_matches_in_any_column() {
        for (grid_size, top_slot) in [(2, 0), (2, 1), (3, 0), (3, 1), (3, 5)] {
            assert_eq!(
                mace_result(grid_size, top_slot).as_deref(),
                Some("minecraft:mace"),
                "grid {grid_size}x{grid_size}, top slot {top_slot}"
            );
        }
    }

    #[test]
    fn taking_cake_returns_empty_buckets_and_consumes_other_ingredients() {
        let (mut handler, _, player) = test_handler();
        place_cake(&handler);
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::CAKE.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        for slot in 1..=3 {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::BUCKET.id);
        }
        for slot in 4..=9 {
            assert!(grid_stack(&handler, slot).is_empty());
        }
    }

    #[test]
    fn taking_honey_block_returns_glass_bottles() {
        let (mut handler, _, player) = test_handler();
        for slot in [1, 2, 4, 5] {
            set_slot(&handler, slot, ItemStack::new(1, &Item::HONEY_BOTTLE));
        }
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::HONEY_BLOCK.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        for slot in [1, 2, 4, 5] {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::GLASS_BOTTLE.id);
        }
    }

    #[test]
    fn taking_sugar_returns_the_honey_bottle_remainder() {
        let (mut handler, _, player) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::HONEY_BOTTLE));
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::SUGAR.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        assert_eq!(grid_stack(&handler, 1).item.id, Item::GLASS_BOTTLE.id);
    }

    #[test]
    fn taking_a_recipe_without_remainders_still_consumes_the_grid() {
        let (mut handler, _, player) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::OAK_LOG));
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::OAK_PLANKS.id);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        assert!(grid_stack(&handler, 1).is_empty());
    }

    #[test]
    fn occupied_input_slot_drops_the_remainder() {
        let (mut handler, player_inventory, player) = test_handler();
        place_cake(&handler);
        set_slot(&handler, 1, ItemStack::new(2, &Item::MILK_BUCKET));
        for slot in 0..player_inventory.size() {
            player_inventory.set_stack(slot, ItemStack::new(64, &Item::DIRT));
        }
        refresh_result(&handler);

        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);

        let milk = grid_stack(&handler, 1);
        assert_eq!(milk.item.id, Item::MILK_BUCKET.id);
        assert_eq!(milk.item_count, 1);
        let drops = player
            .drops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].item.id, Item::BUCKET.id);
    }

    #[test]
    fn shift_clicking_the_result_returns_remainders() {
        let (mut handler, _, player) = test_handler();
        place_cake(&handler);
        refresh_result(&handler);

        handler.on_slot_click(0, 0, SlotActionType::QuickMove, &player);

        for slot in 1..=3 {
            assert_eq!(grid_stack(&handler, slot).item.id, Item::BUCKET.id);
        }
    }

    #[test]
    fn transmute_recipes_preserve_components_and_clone_maps() {
        use pumpkin_data::data_component_impl::{ContainerImpl, CustomNameImpl};
        use pumpkin_util::text::TextComponent;

        let (handler, _, _) = test_handler();
        let mut shulker = ItemStack::new(1, &Item::SHULKER_BOX);
        shulker.set_data_component(CustomNameImpl {
            name: TextComponent::text("Loot Box"),
        });
        shulker.set_data_component(ContainerImpl {
            items: vec![(0, ItemStack::new(5, &Item::DIAMOND))],
        });
        set_slot(&handler, 1, shulker);
        set_slot(&handler, 2, ItemStack::new(1, &Item::RED_DYE));
        refresh_result(&handler);

        let dyed_shulker = grid_stack(&handler, 0);
        assert_eq!(dyed_shulker.item.id, Item::RED_SHULKER_BOX.id);
        assert_eq!(
            dyed_shulker
                .get_data_component::<CustomNameImpl>()
                .map(|c| c.name.clone().get_text()),
            Some("Loot Box".to_string())
        );
        let inside = dyed_shulker.get_data_component::<ContainerImpl>().unwrap();
        assert_eq!(inside.items.len(), 1);
        assert_eq!(inside.items[0].1.item.id, Item::DIAMOND.id);
        assert_eq!(inside.items[0].1.item_count, 5);

        let (handler, _, _) = test_handler();
        let mut filled_map = ItemStack::new(1, &Item::FILLED_MAP);
        filled_map.set_data_component(MapIdImpl { id: 42 });
        set_slot(&handler, 1, filled_map);
        set_slot(&handler, 2, ItemStack::new(1, &Item::MAP));
        set_slot(&handler, 3, ItemStack::new(1, &Item::MAP));
        set_slot(&handler, 4, ItemStack::new(1, &Item::MAP));
        refresh_result(&handler);

        let cloned_maps = grid_stack(&handler, 0);
        assert_eq!(cloned_maps.item.id, Item::FILLED_MAP.id);
        assert_eq!(cloned_maps.item_count, 4);
        assert_eq!(
            cloned_maps.get_data_component::<MapIdImpl>().map(|m| m.id),
            Some(42)
        );
    }

    #[test]
    fn special_crafting_dye_imbue_and_banner() {
        use pumpkin_data::data_component_impl::BannerPatternLayer;

        // 1. Dye leather armor
        let (handler, _, _) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::LEATHER_HELMET));
        set_slot(&handler, 2, ItemStack::new(1, &Item::RED_DYE));
        refresh_result(&handler);
        let helmet = grid_stack(&handler, 0);
        assert_eq!(helmet.item.id, Item::LEATHER_HELMET.id);
        assert!(helmet.get_data_component::<DyedColorImpl>().is_some());

        // 2. Imbue tipped arrows
        let (handler, _, _) = test_handler();
        for slot in [1, 2, 3, 4, 6, 7, 8, 9] {
            set_slot(&handler, slot, ItemStack::new(1, &Item::ARROW));
        }
        let mut lingering = ItemStack::new(1, &Item::LINGERING_POTION);
        lingering.set_data_component(PotionContentsImpl {
            potion_id: Some(7),
            custom_color: None,
            custom_effects: Vec::new(),
            custom_name: None,
        });
        set_slot(&handler, 5, lingering);
        refresh_result(&handler);
        let tipped = grid_stack(&handler, 0);
        assert_eq!(tipped.item.id, Item::TIPPED_ARROW.id);
        assert_eq!(tipped.item_count, 8);
        assert_eq!(
            tipped
                .get_data_component::<PotionContentsImpl>()
                .and_then(|p| p.potion_id),
            Some(7)
        );

        // 3. Shield decoration + Banner duplication with remainder
        let mut patterned_banner = ItemStack::new(1, &Item::ORANGE_BANNER);
        patterned_banner.set_data_component(BannerPatternsImpl {
            layers: vec![BannerPatternLayer {
                pattern: "minecraft:cross".to_string(),
                color: DyeColor::Black,
            }],
        });

        let (handler, _, _) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::SHIELD));
        set_slot(&handler, 2, patterned_banner.clone());
        refresh_result(&handler);
        let dec_shield = grid_stack(&handler, 0);
        assert_eq!(dec_shield.item.id, Item::SHIELD.id);
        assert_eq!(
            dec_shield
                .get_data_component::<BaseColorImpl>()
                .map(|c| c.color.as_str()),
            Some("orange")
        );
        assert_eq!(
            dec_shield
                .get_data_component::<BannerPatternsImpl>()
                .map(|p| p.layers.len()),
            Some(1)
        );

        let (mut handler, _, player) = test_handler();
        set_slot(&handler, 1, patterned_banner);
        set_slot(&handler, 2, ItemStack::new(1, &Item::ORANGE_BANNER));
        refresh_result(&handler);
        assert_eq!(grid_stack(&handler, 0).item.id, Item::ORANGE_BANNER.id);
        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);
        assert_eq!(grid_stack(&handler, 1).item.id, Item::ORANGE_BANNER.id);
        assert_eq!(
            grid_stack(&handler, 1)
                .get_data_component::<BannerPatternsImpl>()
                .map(|p| p.layers.len()),
            Some(1)
        );
        assert!(grid_stack(&handler, 2).is_empty());
    }

    #[test]
    fn special_crafting_book_cloning() {
        use pumpkin_util::text::TextComponent;

        // 4. Book cloning increments generation and leaves source remainder, rejecting gen >= 2
        let (mut handler, _, player) = test_handler();
        let mut book = ItemStack::new(1, &Item::WRITTEN_BOOK);
        book.set_data_component(WrittenBookContentImpl {
            title: "Guide".to_string(),
            author: "Alex".to_string(),
            generation: 0,
            pages: vec![TextComponent::text("Hello")],
            resolved: true,
        });
        set_slot(&handler, 1, book.clone());
        set_slot(&handler, 2, ItemStack::new(1, &Item::WRITABLE_BOOK));
        set_slot(&handler, 3, ItemStack::new(1, &Item::WRITABLE_BOOK));
        refresh_result(&handler);
        let cloned_book = grid_stack(&handler, 0);
        assert_eq!(cloned_book.item.id, Item::WRITTEN_BOOK.id);
        assert_eq!(cloned_book.item_count, 2);
        assert_eq!(
            cloned_book
                .get_data_component::<WrittenBookContentImpl>()
                .map(|b| b.generation),
            Some(1)
        );
        handler.on_slot_click(0, 0, SlotActionType::Pickup, &player);
        assert_eq!(grid_stack(&handler, 1).item.id, Item::WRITTEN_BOOK.id);

        let (handler, _, _) = test_handler();
        book.set_data_component(WrittenBookContentImpl {
            title: "Tattered".to_string(),
            author: "Alex".to_string(),
            generation: 2,
            pages: Vec::new(),
            resolved: true,
        });
        set_slot(&handler, 1, book);
        set_slot(&handler, 2, ItemStack::new(1, &Item::WRITABLE_BOOK));
        refresh_result(&handler);
        assert!(grid_stack(&handler, 0).is_empty());
    }

    #[test]
    fn special_crafting_fireworks_maps_and_repair() {
        use pumpkin_data::Enchantment;

        // 5. Firework star -> fade -> rocket
        let (handler, _, _) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::GUNPOWDER));
        set_slot(&handler, 2, ItemStack::new(1, &Item::RED_DYE));
        set_slot(&handler, 3, ItemStack::new(1, &Item::DIAMOND));
        refresh_result(&handler);
        let star = grid_stack(&handler, 0);
        assert_eq!(star.item.id, Item::FIREWORK_STAR.id);
        assert!(
            star.get_data_component::<FireworkExplosionImpl>()
                .is_some_and(|e| e.has_trail && e.colors.len() == 1)
        );

        let (handler, _, _) = test_handler();
        set_slot(&handler, 1, star);
        set_slot(&handler, 2, ItemStack::new(1, &Item::BLUE_DYE));
        refresh_result(&handler);
        let faded_star = grid_stack(&handler, 0);
        assert!(
            faded_star
                .get_data_component::<FireworkExplosionImpl>()
                .is_some_and(|e| e.fade_colors.len() == 1)
        );

        let (handler, _, _) = test_handler();
        set_slot(&handler, 1, ItemStack::new(1, &Item::PAPER));
        set_slot(&handler, 2, ItemStack::new(1, &Item::GUNPOWDER));
        set_slot(&handler, 3, ItemStack::new(1, &Item::GUNPOWDER));
        set_slot(&handler, 4, faded_star);
        refresh_result(&handler);
        let rocket = grid_stack(&handler, 0);
        assert_eq!(rocket.item.id, Item::FIREWORK_ROCKET.id);
        assert_eq!(rocket.item_count, 3);
        let fw = rocket.get_data_component::<FireworksImpl>().unwrap();
        assert_eq!(fw.flight_duration, 2);
        assert_eq!(fw.explosions.len(), 1);

        // 6. Map extending
        let (handler, _, _) = test_handler();
        for slot in [1, 2, 3, 4, 6, 7, 8, 9] {
            set_slot(&handler, slot, ItemStack::new(1, &Item::PAPER));
        }
        let mut map = ItemStack::new(1, &Item::FILLED_MAP);
        map.set_data_component(MapIdImpl { id: 9 });
        set_slot(&handler, 5, map);
        refresh_result(&handler);
        let ext_map = grid_stack(&handler, 0);
        assert_eq!(ext_map.item.id, Item::FILLED_MAP.id);
        assert_eq!(
            ext_map
                .get_data_component::<MapPostProcessingImpl>()
                .and_then(|m| m.processing),
            Some(MapPostProcessing::Scale)
        );

        // 7. RepairItem: 5% bonus durability, strips sharpness, retains binding_curse
        let (handler, _, _) = test_handler();
        let mut s1 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        s1.set_damage(1000);
        let sharpness = Enchantment::from_name("minecraft:sharpness").unwrap();
        let binding_curse = Enchantment::from_name("minecraft:binding_curse").unwrap();
        s1.set_data_component(EnchantmentsImpl {
            enchantment: std::borrow::Cow::Owned(vec![(sharpness, 5), (binding_curse, 1)]),
        });
        let mut s2 = ItemStack::new(1, &Item::DIAMOND_SWORD);
        s2.set_damage(1000);
        set_slot(&handler, 1, s1);
        set_slot(&handler, 2, s2);
        refresh_result(&handler);
        let repaired = grid_stack(&handler, 0);
        assert_eq!(repaired.item.id, Item::DIAMOND_SWORD.id);
        let max_dmg = repaired.get_max_damage().unwrap();
        let rem = max_dmg - 1000;
        let expected_rem = (rem + rem + max_dmg * 5 / 100).min(max_dmg);
        assert_eq!(repaired.get_damage(), max_dmg - expected_rem);
        assert_eq!(repaired.get_enchantment_level(sharpness), 0);
        assert_eq!(repaired.get_enchantment_level(binding_curse), 1);
    }
}
